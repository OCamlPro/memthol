//! Label filters.

use regex::Regex;

use crate::{base::*, filter::FilterSpec};

/// Label filter.
#[derive(Debug, Clone)]
pub enum LabelFilter {
    /// Retains label lists that contain a label verifying this spec.
    Contains(Vec<LabelSpec>),
    /// Retains label lists that do not contain a label verifying this spec.
    Excludes(Vec<LabelSpec>),
}
impl FilterSpec<[String]> for LabelFilter {
    fn apply(&self, data: &Storage, alloc_data: &[String]) -> bool {
        match self {
            Self::Contains(specs) => Self::check_contains(specs, data, alloc_data),
            Self::Excludes(specs) => !Self::check_contains(specs, data, alloc_data),
        }
    }

    fn render(&self) -> Html {
        let (op, specs) = match self {
            Self::Contains(specs) => ("contain", specs),
            Self::Excludes(specs) => ("exclude", specs),
        };

        html! {
            <>
                <li class=style::class::filter::line::CELL>
                    <a class=style::class::filter::line::CMP_CELL> { op } </a>
                </li>
                <li class=style::class::filter::line::CELL>
                    <a class=style::class::filter::line::VAL_CELL>
                        <code> { "[" } </code>
                        {
                            for specs.iter().enumerate().map(
                                |(index, spec)| html! {
                                    // Attach to nothing, will become kid of the `<div>` above.
                                    <>
                                        {
                                            if index > 0 {
                                                html!({"..."})
                                            } else {
                                                html!({"..."})
                                            }
                                        }
                                        { spec.render() }
                                    </>
                                }
                            )
                        }
                        {"..."}
                        <code> { "]" } </code>
                    </a>
                </li>
            </>
        }
    }
}
impl Default for LabelFilter {
    fn default() -> Self {
        Self::Contains(vec![])
    }
}

/// Constructors.
impl LabelFilter {
    /// `Contains` constructor.
    pub fn contains(specs: Vec<LabelSpec>) -> Self {
        Self::Contains(specs)
    }
    /// `Excludes` constructor.
    pub fn excludes(specs: Vec<LabelSpec>) -> Self {
        Self::Excludes(specs)
    }
}

impl LabelFilter {
    /// Helper that returns true if some labels verify the input specs.
    fn check_contains(specs: &[LabelSpec], data: &Storage, labels: &[String]) -> bool {
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

    fn render(&self) -> Html {
        html! {
            <code class=style::class::filter::LABEL> {
                match self {
                    LabelSpec::Value(value) => format!("\"{}\"", value),
                    LabelSpec::Regex(regex) => format!("re(\"{}\")", regex),
                }
            } </code>
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
