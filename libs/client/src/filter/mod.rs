//! Data filtering.

use crate::base::*;

pub mod label;
pub mod ord;

pub use label::LabelFilter;
use ord::OrdFilter;

/// A filter over allocation sizes.
pub type SizeFilter = OrdFilter<usize>;

/// A filter over lifetimes.
pub type LifetimeFilter = OrdFilter<SinceStart>;

/// Function(s) a filter must implement.
pub trait FilterSpec<Data>: Sized
where
    Data: ?Sized,
{
    /// Applies the filter to some allocation data.
    fn apply(&self, data: &Storage, alloc_data: &Data) -> bool;

    /// Renders the filter.
    fn render<Update>(&self, update: Update) -> Html
    where
        Update: Fn(Res<Self>) -> Msg + Clone + 'static;
}

/// Filter kind.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum FilterKind {
    Size,
    Lifetime,
    Label,
}
impl fmt::Display for FilterKind {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Size => write!(fmt, "size"),
            Self::Lifetime => write!(fmt, "lifetime"),
            Self::Label => write!(fmt, "labels"),
        }
    }
}

impl FilterKind {
    pub fn all() -> Vec<FilterKind> {
        vec![FilterKind::Size, FilterKind::Lifetime, FilterKind::Label]
    }
}

/// An allocation filter.
#[derive(Debug, Clone)]
pub enum Filter {
    /// Filter over allocation sizes.
    Size(SizeFilter),
    /// Filter over allocation lifetimes.
    Lifetime(LifetimeFilter),
    /// Filter over labels.
    Label(LabelFilter),
}
impl Filter {
    /// Default filter for some filter kind.
    pub fn of_kind(kind: FilterKind) -> Self {
        match kind {
            FilterKind::Size => SizeFilter::default().into(),
            FilterKind::Lifetime => LifetimeFilter::default().into(),
            FilterKind::Label => LabelFilter::default().into(),
        }
    }

    /// Filter kind of a filter.
    pub fn kind(&self) -> FilterKind {
        match self {
            Self::Size(_) => FilterKind::Size,
            Self::Lifetime(_) => FilterKind::Lifetime,
            Self::Label(_) => FilterKind::Label,
        }
    }

    /// Applies the filter to an allocation.
    pub fn apply(&self, data: &Storage, alloc: &Alloc) -> bool {
        match self {
            Filter::Size(filter) => filter.apply(data, &alloc.size),
            Filter::Lifetime(filter) => {
                let tod = alloc
                    .tod
                    .as_ref()
                    .unwrap_or_else(|| data.current_time_since_start());
                let lifetime = *tod - alloc.toc;
                filter.apply(data, &lifetime)
            }
            Filter::Label(filter) => filter.apply(data, &alloc.labels),
        }
    }

    fn prop_selector<Update>(&self, update: Update) -> Html
    where
        Update: Fn(Res<Filter>) -> Msg + 'static,
    {
        let selected = self.kind();
        html! {
            <Select<FilterKind>
                selected=Some(selected)
                options=FilterKind::all()
                onchange=move |kind| update(Ok(Self::of_kind(kind)))
            />
        }
    }

    /// Renders the filter.
    pub fn render<Update>(&self, update: Update) -> Html
    where
        Update: Fn(Res<Filter>) -> Msg + Clone + 'static,
    {
        html! {
            <>
                {
                    match self {
                        Filter::Size(filter) => html! {
                            <>
                                <li class=style::class::filter::line::CELL>
                                    <a class=style::class::filter::line::PROP_CELL>
                                    //     { "size" }
                                        { self.prop_selector(update.clone()) }
                                    </a>
                                </li>
                                {
                                    filter.render(
                                        move |filter| update(
                                            filter.map(|filter| filter.into())
                                        )
                                    )
                                }
                            </>
                        },
                        Filter::Lifetime(filter) => html! {
                            <>
                                <li class=style::class::filter::line::CELL>
                                    <a class=style::class::filter::line::PROP_CELL>
                                    //     { "lifetime" }
                                        { self.prop_selector(update.clone()) }
                                    </a>
                                </li>
                                {
                                    filter.render(
                                        move |filter| update(
                                            filter.map(|filter| filter.into())
                                        )
                                    )
                                }
                            </>
                        },
                        Filter::Label(filter) => html! {
                            <>
                                <li class=style::class::filter::line::CELL>
                                    <a class=style::class::filter::line::PROP_CELL>
                                    //     { "labels" }
                                        { self.prop_selector(update.clone()) }
                                    </a>
                                </li>
                                {
                                    filter.render(
                                        move |filter| update(
                                            filter.map(|filter| filter.into())
                                        )
                                    )
                                }
                            </>
                        },
                    }
                }
            </>
        }
    }

    /// Renders the filter in edition mode.
    pub fn edit_render<F>(&self) -> Html
    where
        F: Fn(Filter) -> Msg + 'static,
    {
        unimplemented!()
    }
}

impl From<SizeFilter> for Filter {
    fn from(filter: SizeFilter) -> Self {
        Self::Size(filter)
    }
}
impl From<LifetimeFilter> for Filter {
    fn from(filter: LifetimeFilter) -> Self {
        Self::Lifetime(filter)
    }
}
impl From<LabelFilter> for Filter {
    fn from(filter: LabelFilter) -> Self {
        Self::Label(filter)
    }
}
impl Default for Filter {
    fn default() -> Self {
        SizeFilter::default().into()
    }
}
