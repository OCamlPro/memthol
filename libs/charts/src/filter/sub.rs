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

    /// Changes the filter kind of a sub-filter.
    ///
    /// Returns `true` iff the filter actually changed.
    pub fn change_kind(&mut self, kind: FilterKind) -> bool {
        if self.kind() == kind {
            return false;
        }

        *self = Self::of_kind(kind);
        true
    }
}

impl fmt::Display for SubFilter {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Size(filter) => write!(fmt, "size {}", filter),
            Self::Label(filter) => write!(fmt, "labels {}", filter),
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

/// Sub-filter update.
pub enum Update {
    /// Size filter update.
    Size(ord::SizeUpdate),
    /// Label filter update.
    Label(label::Update),
}
impl fmt::Display for Update {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Size(update) => update.fmt(fmt),
            Self::Label(update) => update.fmt(fmt),
        }
    }
}

impl SubFilter {
    /// Updates a sub-filter.
    pub fn update(&mut self, update: Update) -> Res<bool> {
        match self {
            Self::Size(filter) => match update {
                Update::Size(update) => filter.update(update),
                update => bail!("cannot apply update `{}` to filter `{}`", update, filter),
            },
            Self::Label(filter) => match update {
                Update::Label(update) => filter.update(update),
                update => bail!("cannot apply update `{}` to filter `{}`", update, filter),
            },
        }
    }
}
