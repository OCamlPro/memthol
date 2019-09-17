//! Data filtering.
//!
//! All types in this module implement `serde`'s `Serialize` and `Deserialize` traits.

use crate::base::*;

pub mod label;
pub mod ord;
mod sub;

pub use label::LabelFilter;
use ord::OrdFilter;
pub use sub::SubFilter;

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
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum FilterKind {
    /// Size filter.
    Size,
    /// Label filter.
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
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Filters {
    /// The actual list.
    filters: Vec<Filter>,
}

impl Filters {
    /// Constructor.
    pub fn new() -> Self {
        Filters { filters: vec![] }
    }

    /// Length of the list of filters.
    pub fn len(&self) -> usize {
        self.filters.len()
    }

    /// Searches for a filter that matches on the input allocation.
    pub fn find_match(&self, alloc: &Alloc) -> Option<index::Filter> {
        for (index, filter) in self.filters.iter().enumerate() {
            if filter.apply(alloc) {
                return Some(index::Filter::new(index));
            }
        }
        None
    }

    /// Applies a filter message.
    pub fn update(&mut self, msg: msg::to_server::FiltersMsg) {
        use msg::to_server::FiltersMsg::*;
        match msg {
            Add { filter } => self.filters.push(filter),
            Rm { index } => {
                self.filters.remove(*index);
                ()
            }
            Filter { index, msg } => self.filters[*index].update(msg),
        }
    }
}

/// A filter that combines `SubFilter`s.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Filter {
    /// Actual list of filters.
    filters: Vec<SubFilter>,
}
impl Filter {
    /// Constructor.
    pub fn new() -> Filter {
        Self { filters: vec![] }
    }

    /// Applies the filters to an allocation.
    pub fn apply(&self, alloc: &Alloc) -> bool {
        for filter in &self.filters {
            if filter.apply(alloc) {
                return true;
            }
        }
        false
    }

    /// Applies a filter message.
    pub fn update(&mut self, msg: msg::to_server::FilterMsg) {
        use msg::to_server::FilterMsg::*;

        match msg {
            Add { filter } => self.filters.push(filter),
            Rm { index } => {
                self.filters.remove(*index);
                ()
            }
            Update { index, filter } => self.filters[*index] = filter,
        }
    }
}
