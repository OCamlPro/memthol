//! Sub filters.
//!
//! A sub filter is what [`Filter`]s are made of.
//!
//! [`Filter`]: ../struct.Filter.html (The Filter struct).

use crate::base::filter::*;

/// An allocation filter.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SubFilter {
    /// Filter over allocation sizes.
    Size(SizeFilter),
    /// Filter over labels.
    Label(LabelFilter),
}
impl SubFilter {
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
            SubFilter::Size(filter) => filter.apply(&alloc.size),
            SubFilter::Label(filter) => filter.apply(&alloc.labels),
        }
    }
}

impl From<SizeFilter> for SubFilter {
    fn from(filter: SizeFilter) -> Self {
        Self::Size(filter)
    }
}
impl From<LabelFilter> for SubFilter {
    fn from(filter: LabelFilter) -> Self {
        Self::Label(filter)
    }
}
impl Default for SubFilter {
    fn default() -> Self {
        SizeFilter::default().into()
    }
}
