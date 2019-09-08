//! Data filtering.

use crate::base::*;

pub mod label;
pub mod ord;

pub use label::LabelFilter;
use ord::NumFilter;

/// A filter over allocation sizes.
pub type SizeFilter = NumFilter<usize>;

/// A filter over lifetimes.
pub type LifetimeFilter = NumFilter<Duration>;

/// Function(s) a filter must implement.
pub trait FilterSpec<Data>
where
    Data: ?Sized,
{
    /// Applies the filter to some allocation data.
    fn apply(&self, data: &Storage, alloc_data: &Data) -> bool;

    /// Renders the filter.
    fn render(&self) -> Html;
}

/// An allocation filter.
#[derive(Debug, Clone)]
pub enum Filter {
    /// Filter over allocation sizes.
    Size(NumFilter<usize>),
    /// Filter over allocation lifetimes.
    Lifetime(NumFilter<Duration>),
    /// Filter over labels.
    Label(LabelFilter),
}
impl Filter {
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

    pub fn render(&self) -> Html {
        // let actions = html! {
        //     <li class="filter_li">
        //         <input type="checkbox" class="filter_actions"/>
        //     </li>
        // };
        match self {
            Filter::Size(filter) => html! {
                <ul class="filter_ul">
                    <li class="filter_li">
                        <a class="filter_prop">{ "size" }</a>
                    </li>
                    { filter.render() }
                </ul>
            },
            Filter::Lifetime(filter) => html! {
                <ul class="filter_ul">
                    <li class="filter_li">
                        <input type="checkbox" class="filter_tick"/>
                    </li>
                    <li class="filter_li">
                        <a class="filter_prop">{ "lifetime" }</a>
                    </li>
                    { filter.render() }
                </ul>
            },
            Filter::Label(filter) => html! {
                <ul class="filter_ul">
                    // { actions }
                    <li class="filter_li">
                        <a class="filter_prop">
                            { "labels" }
                        </a>
                    </li>
                    { filter.render() }
                </ul>
            },
        }
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
impl From<FilterKind> for Filter {
    fn from(kind: FilterKind) -> Self {
        match kind {
            FilterKind::Size => SizeFilter::default().into(),
            FilterKind::Lifetime => LifetimeFilter::default().into(),
            FilterKind::Label => LabelFilter::default().into(),
        }
    }
}

/// Filter kind.
#[derive(Debug, Clone)]
pub enum FilterKind {
    /// Size filter.
    Size,
    /// Lifetime filter.
    Lifetime,
    /// Label filter.
    Label,
}
