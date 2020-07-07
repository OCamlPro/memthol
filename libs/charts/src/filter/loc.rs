//! Location filters.

prelude! {}

use filter::{string_like, FilterExt};

/// A location filter.
pub type LocFilter = string_like::StringLikeFilter<LocSpec>;

/// A location list predicate.
pub type LocPred = string_like::Pred;

/// An update for a location filter.
pub type LocUpdate = string_like::Update;

/// A line specification.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LineSpec {
    /// Matches a precise line.
    Value(usize),
    /// Matches a range of lines.
    ///
    /// If `lb.is_none()` and `ub.is_none()`, the range matches any line at all.
    Range {
        /// Lower bound.
        lb: Option<usize>,
        /// Upper bound.
        ub: Option<usize>,
    },
}
impl LineSpec {
    /// Matches any line at all.
    pub fn any() -> Self {
        Self::Range { lb: None, ub: None }
    }
    /// Matches a precise line.
    pub fn line(line: usize) -> Self {
        Self::Value(line)
    }
    /// Matches a range of lines.
    pub fn range(lb: Option<usize>, ub: Option<usize>) -> Self {
        Self::Range { lb, ub }
    }

    /// Constructor, from a string.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use charts::filter::loc::*;
    /// let line_spec = LineSpec::new("_").unwrap();
    /// assert_eq!(line_spec, LineSpec::any());
    ///
    /// let line_spec = LineSpec::new("79").unwrap();
    /// assert_eq!(line_spec, LineSpec::line(79));
    ///
    /// let line_spec = LineSpec::new("[79, 105]").unwrap();
    /// assert_eq!(line_spec, LineSpec::range(Some(79), Some(105)));
    ///
    /// let line_spec = LineSpec::new(" [ 79  , 105  ]  ").unwrap();
    /// assert_eq!(line_spec, LineSpec::range(Some(79), Some(105)));
    ///
    /// let line_spec = LineSpec::new(" [ _  , 105  ]  ").unwrap();
    /// assert_eq!(line_spec, LineSpec::range(None, Some(105)));
    ///
    /// let line_spec = LineSpec::new(" [ 105, _  ]  ").unwrap();
    /// assert_eq!(line_spec, LineSpec::range(Some(105), None));
    /// ```
    pub fn new(str: impl AsRef<str>) -> Res<Self> {
        use std::str::FromStr;
        let mut s = str.as_ref();
        macro_rules! s_do {
            (trim) => {
                s = s.trim()
            };
            (sub $lb:expr) => {
                s = &s[$lb..];
                s_do!(trim)
            };
            (bail) => {
                bail!(s_do!(bail msg))
            };
            (bail if $cond:expr) => {
                if $cond {
                    s_do!(bail)
                }
            };
            (bail msg) => {
                format!("`{}` is not a legal line specification", str.as_ref())
            }
        }

        s_do!(trim);
        if s.is_empty() {
            return Ok(Self::any());
        }
        // `s` is not empty now.

        if &s[0..1] == "[" {
            s_do!(sub 1);
            s_do!(bail if s.is_empty());

            let mut subs = s.split(",");

            let lb = if let Some(mut lb) = subs.next() {
                lb = lb.trim();
                if lb == "_" {
                    None
                } else {
                    let lb = usize::from_str(lb).chain_err(|| s_do!(bail msg))?;
                    Some(lb)
                }
            } else {
                s_do!(bail)
            };

            let ub = if let Some(mut ub) = subs.next() {
                ub = ub.trim();
                s_do!(bail if
                    ub.is_empty() || &ub[ub.len() - 1..ub.len()] != "]"
                );
                ub = ub[0..ub.len() - 1].trim();
                if ub == "_" {
                    None
                } else {
                    let ub = usize::from_str(ub).chain_err(|| s_do!(bail msg))?;
                    Some(ub)
                }
            } else {
                s_do!(bail)
            };

            Ok(Self::range(lb, ub))
        } else if s == "_" {
            Ok(Self::any())
        } else {
            let line = usize::from_str(s).chain_err(|| s_do!(bail msg))?;
            Ok(Self::line(line))
        }
    }

    /// True if the line specification matches anything.
    pub fn matches_anything(&self) -> bool {
        match self {
            Self::Value(_) => false,
            Self::Range { lb: None, ub: None } => true,
            Self::Range { .. } => false,
        }
    }
}
impl FilterExt<usize> for LineSpec {
    fn apply(&self, line: &usize) -> bool {
        match self {
            Self::Value(l) => l == line,
            Self::Range { lb, ub } => {
                lb.map(|lb| lb <= *line).unwrap_or(true) && ub.map(|ub| *line <= ub).unwrap_or(true)
            }
        }
    }
}
impl fmt::Display for LineSpec {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Value(line) => line.fmt(fmt),
            Self::Range { lb: None, ub: None } => "_".fmt(fmt),
            Self::Range { lb, ub } => {
                "[".fmt(fmt)?;
                if let Some(lb) = lb {
                    lb.fmt(fmt)?
                } else {
                    "_".fmt(fmt)?
                }
                write!(fmt, ", ")?;
                if let Some(ub) = ub {
                    ub.fmt(fmt)?
                } else {
                    "_".fmt(fmt)?
                }
                "]".fmt(fmt)
            }
        }
    }
}

/// Loc specification.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LocSpec {
    /// Matches a sequence of arbitrary locations.
    Anything,
    /// An actualy location value.
    Value {
        /// Location string.
        value: String,
        /// Location line.
        line: LineSpec,
    },
    /// A regular expression.
    Regex {
        /// Location regex.
        #[serde(with = "serde_regex")]
        regex: Regex,
        /// Location line.
        line: LineSpec,
    },
}
impl FilterExt<alloc::CLoc> for LocSpec {
    fn apply(&self, alloc::CLoc { loc, .. }: &alloc::CLoc) -> bool {
        match self {
            LocSpec::Anything => true,
            LocSpec::Value { value, line } => &loc.file == value && line.apply(&loc.line),
            LocSpec::Regex { regex, line } => regex.is_match(&loc.file) && line.apply(&loc.line),
        }
    }
}

impl string_like::SpecExt for LocSpec {
    type Data = alloc::CLoc;
    const DATA_DESC: &'static str = "label";

    fn from_string(s: impl Into<String>) -> Res<Self> {
        Self::new(s)
    }

    /// True if the spec is an empty label.
    fn is_empty(&self) -> bool {
        match self {
            LocSpec::Value { value, line } => value == "" && line.matches_anything(),
            LocSpec::Regex { .. } => false,
            LocSpec::Anything => false,
        }
    }

    fn data_of_alloc(alloc: &Alloc) -> Arc<Vec<Self::Data>> {
        alloc.trace()
    }

    /// True if the input data is a match for this specification.
    fn matches(&self, data: &Self::Data) -> bool {
        self.apply(data)
    }

    /// True if the specification matches a repetition of anything.
    fn matches_anything(&self) -> bool {
        match self {
            Self::Anything => true,
            Self::Value { .. } => false,
            Self::Regex { .. } => false,
        }
    }
}

impl fmt::Display for LocSpec {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Value { value, line } => {
                write!(fmt, "{}", value)?;
                if !line.matches_anything() {
                    write!(fmt, " : {}", line)?
                }
                Ok(())
            }
            Self::Regex { regex, line } => {
                write!(fmt, "#\"{}\"#", regex)?;
                if !line.matches_anything() {
                    write!(fmt, " : {}", line)?
                }
                Ok(())
            }
            Self::Anything => write!(fmt, "**"),
        }
    }
}

impl Default for LocSpec {
    fn default() -> LocSpec {
        Self::new("path/to/my_file.ml:[55, _]").unwrap()
    }
}

impl LocSpec {
    /// Constructor from strings.
    pub fn new(s: impl Into<String>) -> Res<Self> {
        let loc = s.into();
        macro_rules! illegal {
            () => {{
                let err: err::Err = format!("illegal regex `{}`", loc).into();
                err
            }};
        }

        if loc == "**" {
            return Ok(Self::Anything);
        }

        let (file, line) = {
            let mut subs = loc.split(':');
            if let Some(file) = subs.next() {
                let file = file.trim();
                if let Some(line) = subs.next() {
                    if subs.next().is_some() {
                        bail!(
                            illegal!().chain_err(|| "found more than one `:` file/line separators")
                        )
                    }
                    (file, Some(line.trim()))
                } else {
                    (file, None)
                }
            } else {
                // Unreachable.
                bail!(illegal!().chain_err(|| "entered unreachable code"))
            }
        };

        let line = line
            .map(|s| LineSpec::new(s))
            .unwrap_or_else(|| Ok(LineSpec::any()));

        if file.len() > 2 && &file[0..2] == "#\"" {
            if &file[file.len() - 2..file.len()] != "\"#" {
                bail!(illegal!().chain_err(|| "a regex must end with `\"#`"))
            }

            let regex = Regex::new(&file[2..file.len() - 2])
                .map_err(|e| illegal!().chain_err(|| format!("{}", e)))?;
            Ok(Self::Regex { regex, line: line? })
        } else {
            Ok(Self::Value {
                value: file.into(),
                line: line?,
            })
        }
    }

    /// True if the spec matches anything.
    pub fn matches_anything(&self) -> bool {
        match self {
            Self::Anything => true,
            Self::Value { .. } => false,
            Self::Regex { .. } => false,
        }
    }
}

impl Default for string_like::StringLikeFilter<LocSpec> {
    fn default() -> Self {
        Self::new(
            string_like::Pred::Contain,
            vec![LocSpec::Anything, LocSpec::default(), LocSpec::Anything],
        )
    }
}
