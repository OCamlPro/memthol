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

    fn kind(&self) -> (Kind, &Vec<LabelSpec>) {
        match self {
            Self::Contain(specs) => (Kind::Contain, specs),
            Self::Exclude(specs) => (Kind::Exclude, specs),
        }
    }

    fn of_kind(kind: Kind, labels: Vec<LabelSpec>) -> Self {
        match kind {
            Kind::Contain => Self::contain(labels),
            Kind::Exclude => Self::exclude(labels),
        }
    }

    fn kind_selector<Update>(&self, update: Update) -> Html
    where
        Update: Fn(Option<filter::Filter>) -> Msg + 'static,
    {
        let (selected, specs) = self.kind();
        let specs = specs.clone();
        html! {
            <Select<Kind>
                selected=Some(selected)
                options=Kind::all()
                onchange=move |kind| update(Some(Self::of_kind(kind, specs.clone()).into()))
            />
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
        Update: Fn(Option<filter::Filter>) -> Msg + Copy + 'static,
    {
        let specs = match self {
            Self::Contain(specs) => specs,
            Self::Exclude(specs) => specs,
        };

        html! {
            <>
                <li class=style::class::filter::line::CELL>
                    <a class=style::class::filter::line::CMP_CELL>
                        { self.kind_selector(update) }
                    </a>
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
                                                html!(<code>{"..."}</code>)
                                            } else {
                                                html!(<code>{"..."}</code>)
                                            }
                                        }
                                        { spec.render(update) }
                                    </>
                                }
                            )
                        }
                        <code> {"..."} </code>
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

    fn render<Update>(&self, _update: Update) -> Html
    where
        Update: Fn(Option<filter::Filter>) -> Msg,
    {
        let value = match self {
            LabelSpec::Value(value) => format!("\"{}\"", value),
            LabelSpec::Regex(regex) => format!("#\"{}\"#", regex),
        };
        html! {
            <input
                type="text"
                class=style::class::filter::VALUE
                value=value
            />
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
