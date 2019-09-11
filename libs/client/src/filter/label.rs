//! Label filters.

use regex::Regex;

use crate::{base::*, filter::FilterSpec};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Kind {
    Contain,
    Exclude,
}
impl fmt::Display for Kind {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Contain => write!(fmt, "contain"),
            Self::Exclude => write!(fmt, "exclude"),
        }
    }
}
impl Kind {
    pub fn all() -> Vec<Kind> {
        vec![Self::Contain, Self::Exclude]
    }
}

/// Label filter.
#[derive(Debug, Clone)]
pub enum LabelFilter {
    /// Retains label lists that contain a label verifying this spec.
    Contain(Vec<LabelSpec>),
    /// Retains label lists that do not contain a label verifying this spec.
    Exclude(Vec<LabelSpec>),
}

/// Constructors.
impl LabelFilter {
    /// `Contain` constructor.
    pub fn contain(specs: Vec<LabelSpec>) -> Self {
        Self::Contain(specs)
    }
    /// `Exclude` constructor.
    pub fn exclude(specs: Vec<LabelSpec>) -> Self {
        Self::Exclude(specs)
    }

    /// Filter's kind.
    fn kind(&self) -> (Kind, &Vec<LabelSpec>) {
        match self {
            Self::Contain(specs) => (Kind::Contain, specs),
            Self::Exclude(specs) => (Kind::Exclude, specs),
        }
    }

    /// Constructor from a kind.
    fn of_kind(kind: Kind, labels: Vec<LabelSpec>) -> Self {
        match kind {
            Kind::Contain => Self::contain(labels),
            Kind::Exclude => Self::exclude(labels),
        }
    }

    fn kind_selector<Update>(&self, update: Update) -> Html
    where
        Update: Fn(Res<Self>) -> Msg + 'static,
    {
        let (selected, specs) = self.kind();
        let specs = specs.clone();
        html! {
            <Select<Kind>
                selected=Some(selected)
                options=Kind::all()
                onchange=move |kind| update(Ok(Self::of_kind(kind, specs.clone())))
            />
        }
    }

    ///
    pub fn dots<Update>(&self, update: Update, index: usize) -> Html
    where
        Update: Fn(Res<Self>) -> Msg + Clone + 'static,
    {
        let slf = self.clone();
        html! {
            <code
                class=style::class::filter::line::ADD_LABEL
                onclick=move |_| {
                    let mut filter = slf.clone();
                    let specs = filter.specs_mut();
                    specs.insert(
                        index, LabelSpec::default()
                    );
                    update(Ok(filter))
                }
            >{"..."}</code>
        }
    }

    /// Label specifications.
    pub fn specs(&self) -> &Vec<LabelSpec> {
        match self {
            Self::Contain(specs) => specs,
            Self::Exclude(specs) => specs,
        }
    }
    /// Label specifications (mutable).
    pub fn specs_mut(&mut self) -> &mut Vec<LabelSpec> {
        match self {
            Self::Contain(specs) => specs,
            Self::Exclude(specs) => specs,
        }
    }

    /// Inserts a specification.
    ///
    /// If the specification is empty, removes the spec at that index.
    pub fn insert(&mut self, index: usize, spec: LabelSpec) {
        let specs = self.specs_mut();
        if spec.is_empty() {
            specs.remove(index);
            ()
        } else {
            specs[index] = spec
        }
    }
}

impl FilterSpec<[String]> for LabelFilter {
    fn apply(&self, data: &Storage, alloc_data: &[String]) -> bool {
        match self {
            Self::Contain(specs) => Self::check_contain(specs, data, alloc_data),
            Self::Exclude(specs) => !Self::check_contain(specs, data, alloc_data),
        }
    }

    fn render<Update>(&self, update: Update) -> Html
    where
        Update: Fn(Res<Self>) -> Msg + Clone + 'static,
    {
        let specs = match self {
            Self::Contain(specs) => specs,
            Self::Exclude(specs) => specs,
        };

        html! {
            <>
                <li class=style::class::filter::line::CELL>
                    <a class=style::class::filter::line::CMP_CELL>
                        { self.kind_selector(update.clone()) }
                    </a>
                </li>
                <li class=style::class::filter::line::CELL>
                    <a class=style::class::filter::line::VAL_CELL>
                        <code> { "[" } </code>
                        {
                            for specs.iter().enumerate().map(
                                |(index, spec)| {
                                    html! {
                                        // Attach to nothing, will become kid of the `<div>` above.
                                        <>
                                            { self.dots(update.clone(), index) }
                                            {
                                                let slf = self.clone();
                                                let update = update.clone();
                                                spec.render(
                                                    move |spec| update(
                                                        spec.map(
                                                            |spec| {
                                                                let mut filter = slf.clone();
                                                                filter.insert(index, spec);
                                                                filter
                                                            }
                                                        )
                                                    )
                                                )
                                            }
                                        </>
                                    }
                                }
                            )
                        }
                        { self.dots(update.clone(), specs.len()) }
                        <code> { "]" } </code>
                    </a>
                </li>
            </>
        }
    }
}
impl Default for LabelFilter {
    fn default() -> Self {
        Self::Contain(vec![])
    }
}

impl LabelFilter {
    /// Helper that returns true if some labels verify the input specs.
    fn check_contain(specs: &[LabelSpec], data: &Storage, labels: &[String]) -> bool {
        let mut labels = labels.iter();
        'next_spec: for spec in specs.iter() {
            'find_match: while let Some(label) = labels.next() {
                if spec.apply(data, label) {
                    continue 'next_spec;
                } else {
                    continue 'find_match;
                }
            }
            // Only reachable if there are no more labels.
            return false;
        }
        // Only reachable if there are no more specs and all succeeded.
        true
    }
}

/// Label specification.
#[derive(Debug, Clone)]
pub enum LabelSpec {
    /// An actualy label value.
    Value(String),
    /// A regular expression.
    Regex(Regex),
}
impl FilterSpec<str> for LabelSpec {
    fn apply(&self, _: &Storage, label: &str) -> bool {
        match self {
            LabelSpec::Value(value) => label == value,
            LabelSpec::Regex(regex) => regex.is_match(label),
        }
    }

    fn render<Update>(&self, update: Update) -> Html
    where
        Update: Fn(Res<Self>) -> Msg + 'static,
    {
        let value = match self {
            LabelSpec::Value(value) => format!("{}", value),
            LabelSpec::Regex(regex) => format!("#\"{}\"#", regex),
        };
        html! {
            <input
                type="text"
                class=style::class::filter::VALUE
                value=value
                onchange=|data| update(
                    data.text_value()
                        .and_then(LabelSpec::new)
                        .chain_err(|| "while parsing label")
                )
            />
        }
    }
}
impl Default for LabelSpec {
    fn default() -> LabelSpec {
        LabelSpec::Value("my label".into())
    }
}

impl LabelSpec {
    /// Constructor.
    pub fn new<S>(label: S) -> Res<Self>
    where
        S: Into<String>,
    {
        let label = label.into();
        macro_rules! illegal {
            () => {{
                let err: err::Err = format!("illegal regex `{}`", label).into();
                err
            }};
        }
        if label.len() > 2 && &label[0..2] == "#\"" {
            if &label[label.len() - 2..label.len()] != "\"#" {
                bail!(illegal!().chain_err(|| "a regex must end with `\"#`"))
            }

            let regex = Regex::new(&label[2..label.len() - 2])
                .map_err(|e| illegal!().chain_err(|| format!("{}", e)))?;
            Ok(regex.into())
        } else {
            Ok(label.into())
        }
    }

    /// True if the spec is an empty label.
    pub fn is_empty(&self) -> bool {
        match self {
            LabelSpec::Value(s) => s == "",
            LabelSpec::Regex(_) => false,
        }
    }
}

impl From<String> for LabelSpec {
    fn from(s: String) -> Self {
        Self::Value(s)
    }
}
impl<'a> From<&'a str> for LabelSpec {
    fn from(s: &'a str) -> Self {
        Self::Value(s.into())
    }
}
impl From<Regex> for LabelSpec {
    fn from(re: Regex) -> Self {
        Self::Regex(re)
    }
}
