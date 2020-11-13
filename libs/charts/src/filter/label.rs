/*<LICENSE>
    This file is part of Memthol.

    Copyright (C) 2020 OCamlPro.

    Memthol is free software: you can redistribute it and/or modify
    it under the terms of the GNU General Public License as published by
    the Free Software Foundation, either version 3 of the License, or
    (at your option) any later version.

    Memthol is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
    GNU General Public License for more details.

    You should have received a copy of the GNU General Public License
    along with Memthol.  If not, see <https://www.gnu.org/licenses/>.
*/

//! Label filters.

prelude! {}

use filter::{string_like, FilterExt};

/// A label filter.
pub type LabelFilter = string_like::StringLikeFilter<LabelSpec>;

/// A label list predicate.
pub type LabelPred = string_like::Pred;

/// An update for a label filter.
pub type LabelUpdate = string_like::Update;

/// Label specification.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LabelSpec {
    /// Matches a sequence of arbitrary labels.
    Anything,
    /// An actualy label value.
    Value(String),
    /// A regular expression.
    #[serde(with = "serde_regex")]
    Regex(Regex),
}
impl std::cmp::PartialEq for LabelSpec {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Anything, Self::Anything) => true,
            (Self::Value(lft), Self::Value(rgt)) => lft == rgt,
            (Self::Regex(lft), Self::Regex(rgt)) => lft.as_str() == rgt.as_str(),
            (Self::Anything, _) | (Self::Value(_), _) | (Self::Regex(_), _) => false,
        }
    }
}
impl std::cmp::Eq for LabelSpec {}

impl FilterExt<str> for LabelSpec {
    fn apply(&self, label: &str) -> bool {
        match self {
            LabelSpec::Value(value) => label == value,
            LabelSpec::Regex(regex) => regex.is_match(label),
            LabelSpec::Anything => true,
        }
    }
}

impl string_like::SpecExt for LabelSpec {
    type Data = alloc::Str;
    const DATA_DESC: &'static str = "label";

    fn from_string(s: impl Into<String>) -> Res<Self> {
        Self::new(s)
    }

    /// True if the spec is an empty label.
    fn is_empty(&self) -> bool {
        match self {
            LabelSpec::Value(s) => s == "",
            LabelSpec::Regex(_) => false,
            LabelSpec::Anything => false,
        }
    }

    fn data_of_alloc(alloc: &Alloc) -> Arc<Vec<Self::Data>> {
        alloc.labels()
    }

    /// True if the input data is a match for this specification.
    fn matches(&self, data: &Self::Data) -> bool {
        match self {
            LabelSpec::Value(value) => data == value,
            LabelSpec::Regex(regex) => data.str_do(|s| regex.is_match(s)),
            LabelSpec::Anything => true,
        }
    }

    /// True if the specification matches a repetition of anything.
    fn matches_anything(&self) -> bool {
        match self {
            Self::Anything => true,
            Self::Value(_) => false,
            Self::Regex(_) => false,
        }
    }
}

impl fmt::Display for LabelSpec {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Value(label) => label.fmt(fmt),
            Self::Regex(regex) => write!(fmt, "#\"{}\"#", regex),
            Self::Anything => write!(fmt, "**"),
        }
    }
}

impl Default for LabelSpec {
    fn default() -> LabelSpec {
        LabelSpec::Value("my label".into())
    }
}

impl LabelSpec {
    /// Constructor from strings.
    pub fn new(s: impl Into<String>) -> Res<Self> {
        let label = s.into();
        macro_rules! illegal {
            () => {{
                let err: err::Error = format!("illegal regex `{}`", label).into();
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

    /// True if the spec matches anything.
    pub fn matches_anything(&self) -> bool {
        match self {
            Self::Anything => true,
            Self::Value(_) => false,
            Self::Regex(_) => false,
        }
    }
}

impl From<String> for LabelSpec {
    fn from(s: String) -> Self {
        if &s == "**" {
            Self::Anything
        } else {
            Self::Value(s)
        }
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

impl Default for string_like::StringLikeFilter<LabelSpec> {
    fn default() -> Self {
        Self::new(
            string_like::Pred::Contain,
            vec![
                LabelSpec::Anything,
                LabelSpec::default(),
                LabelSpec::Anything,
            ],
        )
    }
}
