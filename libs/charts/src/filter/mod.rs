//! Data filtering.

use crate::base::*;

pub mod label;
pub mod ord;

pub use label::LabelFilter;
use ord::OrdFilter;

/// A filter over allocation sizes.
pub type SizeFilter = OrdFilter<usize>;

/// Function(s) a filter must implement.
pub trait FilterSpec<Data>: Sized
where
    Data: ?Sized,
{
    /// Applies the filter to some allocation data.
    fn apply(&self, alloc_data: &Data) -> bool;
}

/// Filter kind.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum FilterKind {
    Size,
    Label,
}
impl fmt::Display for FilterKind {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Size => write!(fmt, "size"),
            Self::Label => write!(fmt, "labels"),
        }
    }
}

impl FilterKind {
    pub fn all() -> Vec<FilterKind> {
        vec![FilterKind::Size, FilterKind::Label]
    }
}

/// A list of filters.
pub type Filters = Vec<Filter>;

/// An allocation filter.
#[derive(Debug, Clone)]
pub enum Filter {
    /// Filter over allocation sizes.
    Size(SizeFilter),
    /// Filter over labels.
    Label(LabelFilter),
}
impl Filter {
    /// Default filter for some filter kind.
    pub fn of_kind(kind: FilterKind) -> Self {
        match kind {
            FilterKind::Size => SizeFilter::default().into(),
            FilterKind::Label => LabelFilter::default().into(),
        }
    }

    /// Filter kind of a filter.
    pub fn kind(&self) -> FilterKind {
        match self {
            Self::Size(_) => FilterKind::Size,
            Self::Label(_) => FilterKind::Label,
        }
    }

    /// Applies the filter to an allocation.
    pub fn apply(&self, alloc: &Alloc) -> bool {
        match self {
            Filter::Size(filter) => filter.apply(&alloc.size),
            Filter::Label(filter) => filter.apply(&alloc.labels),
        }
    }
}

impl From<SizeFilter> for Filter {
    fn from(filter: SizeFilter) -> Self {
        Self::Size(filter)
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
