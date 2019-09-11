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

impl<Num> OrdFilter<Num>
where
    Num: PartialEq + PartialOrd + fmt::Display,
{
    pub fn between(lb: Num, ub: Num) -> Res<Self> {
        if lb <= ub {
            Ok(Self::In { lb, ub })
        } else {
            bail!("illegal interval [{}, {}]", lb, ub)
        }
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
            Kind::In => Self::between(Num::default(), Num::default()).unwrap(),
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
        Update: Fn(Res<Self>) -> Msg + 'static,
        Num: Default,
    {
        let selected = self.cmp_kind();
        html! {
            <Select<Kind>
                selected=Some(selected)
                options=Kind::all()
                onchange=move |kind| update(Ok(Self::default_of_cmp(kind)))
            />
        }
    }
}

impl<Num> Default for OrdFilter<Num>
where
    Num: Default + PartialEq + PartialOrd + fmt::Display,
{
    fn default() -> Self {
        Self::cmp(Cmp::Eq, Num::default())
    }
}

impl<Num> OrdFilter<Num>
where
    Num: alloc_data::Parseable + fmt::Display + Clone + PartialOrd + 'static,
    Self: Into<filter::Filter>,
{
    fn parse_text_data(data: ChangeData) -> Res<Num>
    where
        Cmp: fmt::Display,
    {
        data.text_value()
            .and_then(|text| Num::parse(text).map_err(|e| e.into()))
    }

    fn render_values<Update>(&self, update: Update) -> Html
    where
        Update: Fn(Res<Self>) -> Msg + Clone + 'static,
    {
        match self {
            Self::Cmp { cmp, val } => {
                let cmp = *cmp;
                html! {
                    <li class=style::class::filter::line::CELL>
                        <a class=style::class::filter::line::VAL_CELL>
                            <input
                                type="text"
                                class=style::class::filter::VALUE
                                value=val.to_string()
                                onchange=|data| update(
                                    Self::parse_text_data(data)
                                        .chain_err(||
                                            format!(
                                                "while parsing value for filter operator `{}`",
                                                cmp
                                            )
                                        )
                                        .map(|val|
                                            Self::Cmp { cmp, val }
                                        )
                                )
                            >
                            </input>
                        </a>
                    </li>
                }
            }
            Self::In { lb, ub } => {
                let (lb_clone, ub_clone) = (lb.clone(), ub.clone());
                let other_update = update.clone();
                html! {
                    <li class=style::class::filter::line::CELL>
                        <a class=style::class::filter::line::VAL_CELL>
                            <code> { "[" } </code>
                            <input
                                type="text"
                                class=style::class::filter::VALUE
                                value=lb.to_string()
                                onchange=|data| {
                                    let ub = ub_clone.clone();
                                    other_update(
                                        Self::parse_text_data(data)
                                            .and_then(|lb| Self::between(lb, ub))
                                            .chain_err(||
                                                format!(
                                                    "while parsing lower bound \
                                                    for filter operator `{}`",
                                                    Kind::In
                                                )
                                            )
                                    )
                                }
                            >
                            </input>
                            <code> { "," } </code>
                            <input
                                type="text"
                                class=style::class::filter::VALUE
                                value=ub.to_string()
                                onchange=|data| {
                                    let lb = lb_clone.clone();
                                    update(
                                        Self::parse_text_data(data)
                                            .and_then(|ub| Self::between(lb, ub))
                                            .chain_err(||
                                                format!(
                                                    "while parsing upper bound \
                                                    for filter operator `{}`",
                                                    Kind::In
                                                )
                                            )
                                    )
                                }
                            >
                            </input>
                            <code> { "]" } </code>
                        </a>
                    </li>
                }
            }
        }
    }
}

impl<Num> crate::filter::FilterSpec<Num> for OrdFilter<Num>
where
    Num: PartialOrd + PartialEq + fmt::Display + Default + alloc_data::Parseable + Clone + 'static,
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
        Update: Fn(Res<Self>) -> Msg + Clone + 'static,
    {
        html! {
            <>
                <li class=style::class::filter::line::CELL>
                    <a class=style::class::filter::line::CMP_CELL>
                        { self.kind_selector(update.clone()) }
                    </a>
                </li>
                { self.render_values(update) }
            </>
        }
    }
}
