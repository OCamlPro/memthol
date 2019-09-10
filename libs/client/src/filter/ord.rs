///! Filter over ordered (number-like) quantities.
use crate::base::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Kind {
    Eq,
    Ge,
    Le,
    In,
}
impl fmt::Display for Kind {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Eq => write!(fmt, "="),
            Self::Ge => write!(fmt, "≥"),
            Self::Le => write!(fmt, "≤"),
            Self::In => write!(fmt, "⋲"),
        }
    }
}
impl Kind {
    pub fn all() -> Vec<Kind> {
        vec![Self::Eq, Self::Ge, Self::Le, Self::In]
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Cmp {
    Eq,
    Ge,
    Le,
}
impl Cmp {
    pub fn apply<Num: PartialEq + PartialOrd>(&self, lhs: Num, rhs: Num) -> bool {
        match self {
            Self::Eq => lhs == rhs,
            Self::Ge => lhs >= rhs,
            Self::Le => lhs <= lhs,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Eq => "=",
            Self::Le => "≤",
            Self::Ge => "≥",
        }
    }
}
impl fmt::Display for Cmp {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{}", self.as_str())
    }
}

#[derive(Debug, Clone)]
pub enum OrdFilter<Num> {
    Cmp { cmp: Cmp, val: Num },
    In { lb: Num, ub: Num },
}

impl<Num> OrdFilter<Num> {
    pub fn between(lb: Num, ub: Num) -> Self {
        Self::In { lb, ub }
    }
    pub fn cmp(cmp: Cmp, val: Num) -> Self {
        Self::Cmp { cmp, val }
    }
    pub fn default_of_cmp(cmp_kind: Kind) -> Self
    where
        Num: Default,
    {
        match cmp_kind {
            Kind::Eq => Self::cmp(Cmp::Eq, Num::default()),
            Kind::Ge => Self::cmp(Cmp::Ge, Num::default()),
            Kind::Le => Self::cmp(Cmp::Le, Num::default()),
            Kind::In => Self::between(Num::default(), Num::default()),
        }
    }

    pub fn cmp_kind(&self) -> Kind {
        match self {
            Self::Cmp { cmp: Cmp::Eq, .. } => Kind::Eq,
            Self::Cmp { cmp: Cmp::Ge, .. } => Kind::Ge,
            Self::Cmp { cmp: Cmp::Le, .. } => Kind::Le,
            Self::In { .. } => Kind::In,
        }
    }

    fn kind_selector<Update>(&self, update: Update) -> Html
    where
        Update: Fn(Option<filter::Filter>) -> Msg + 'static,
        Num: Default,
        Self: Into<filter::Filter>,
    {
        let selected = self.cmp_kind();
        html! {
            <Select<Kind>
                selected=Some(selected)
                options=Kind::all()
                onchange=move |kind| update(Some(Self::default_of_cmp(kind).into()))
            />
        }
    }
}

impl<Num> Default for OrdFilter<Num>
where
    Num: Default,
{
    fn default() -> Self {
        Self::cmp(Cmp::Eq, Num::default())
    }
}

impl<Num> crate::filter::FilterSpec<Num> for OrdFilter<Num>
where
    Num: PartialOrd + PartialEq + fmt::Display + Default,
    Self: Into<filter::Filter>,
{
    fn apply(&self, _: &Storage, data: &Num) -> bool {
        match self {
            Self::Cmp { cmp, val } => cmp.apply(data, val),
            Self::In { lb, ub } => Cmp::Ge.apply(data, lb) && Cmp::Le.apply(data, ub),
        }
    }

    fn render<Update>(&self, update: Update) -> Html
    where
        Update: Fn(Option<filter::Filter>) -> Msg + 'static,
    {
        match self {
            Self::Cmp { val, .. } => html! {
                <>
                    <li class=style::class::filter::line::CELL>
                        <a class=style::class::filter::line::CMP_CELL>
                            { self.kind_selector(update) }
                        </a>
                    </li>
                    <li class=style::class::filter::line::CELL>
                        <a class=style::class::filter::line::VAL_CELL>
                            <input
                                type="text"
                                class=style::class::filter::VALUE
                                value=val.to_string()
                            >
                            </input>
                        </a>
                    </li>
                </>
            },
            Self::In { lb, ub } => html! {
                <>
                    <li class=style::class::filter::line::CELL>
                        <a class=style::class::filter::line::CMP_CELL>
                            { self.kind_selector(update) }
                        </a>
                    </li>
                    <li class=style::class::filter::line::CELL>
                        <a class=style::class::filter::line::VAL_CELL>
                            <code> { "[" } </code>
                            <input
                                type="text"
                                class=style::class::filter::VALUE
                                value=lb.to_string()
                                onchange=|txt| match txt {
                                    yew::html::ChangeData::Value(txt) => Msg::Blah(txt),
                                    _ => unimplemented!(),
                                }
                            >
                            </input>
                            <code> { "," } </code>
                            <input
                                type="text"
                                class=style::class::filter::VALUE
                                value=ub.to_string()
                            >
                            </input>
                            <code> { "]" } </code>
                        </a>
                    </li>
                </>
            },
        }
    }
}
