//! A filter over allocation data.

use crate::base::*;

new_uid! {
    mod filter_uid {
        uid: FilterUid,
        set: FilterUidSet,
        map: FilterUidMap,
    }
}
pub use filter_uid::*;

/// A single function that filters data.
pub trait AllocFilter {
    /// Filters the data.
    fn filter<'a>(&self, alloc: &'a Alloc) -> bool;

    /// Uid of the filter.
    fn uid(&self) -> FilterUid;
}

/// A single filter.
///
/// Combined by the `Filter` type.
pub type SingleFilter = Box<dyn AllocFilter + 'static>;

/// Actual allocation filter struct.
///
/// - stores a bunch of filters as an heterogeneous list;
/// - can apply these filters to allocation data.
///
/// The names of the filters **must be unique**. This is checked in `debug` but not in `release`.
/// The reason is that filter names are used as keys when removing specific filters.
pub struct Filter {
    /// The filters.
    filters: Vec<SingleFilter>,
}
impl Filter {
    /// Creates an empty filter.
    pub fn new() -> Self {
        Self { filters: vec![] }
    }

    /// Adds a new filter.
    pub fn add<F: AllocFilter + 'static>(&mut self, filter: F) {
        debug_assert! {
            self.filters.iter().all(
                |f| f.uid() != filter.uid()
            )
        }
        self.filters.push(Box::new(filter))
    }

    /// Removes a filter.
    pub fn rm(&mut self, uid: FilterUid) -> Option<SingleFilter> {
        debug_assert! {
            self.filters.iter().any(|f| f.uid() == uid)
        }
        match self.filters.pop() {
            Some(f) => {
                if f.uid() == uid {
                    return Some(f);
                }

                for filter in &mut self.filters {
                    if f.uid() == uid {
                        let filter = std::mem::replace(filter, f);
                        return Some(filter);
                    }
                }
                None
            }
            None => None,
        }
    }

    /// Applies the filter.
    pub fn apply<'a>(&self, alloc: &'a Alloc) -> bool {
        for filter in &self.filters {
            if !filter.filter(alloc) {
                return false;
            }
        }
        true
    }
}

/// A numerical quantity.
///
/// Used in interval filters.
pub trait Quantity: PartialOrd + PartialEq + fmt::Display + Sized + Clone {}
impl Quantity for usize {}

/// Filters allocations for which one a quantity is in an interval.
#[derive(Clone)]
pub struct IntervalFilter<Q, F> {
    lb: Option<Q>,
    ub: Option<Q>,
    f: F,
    uid: FilterUid,
    name: String,
}
impl<Q, F> IntervalFilter<Q, F>
where
    Q: Quantity,
    for<'a> F: Fn(&'a Alloc) -> Q,
{
    /// Constructor.
    pub fn new(f: F, lb: Option<Q>, ub: Option<Q>) -> Self {
        let uid = FilterUid::fresh();
        let name = format!("memthol_filter_{}", uid);
        Self {
            lb,
            ub,
            f,
            uid,
            name,
        }
    }

    /// Sets the lower bound.
    pub fn set_lb(&mut self, q: Option<Q>) {
        self.lb = q
    }
    /// Sets the upper bound.
    pub fn set_ub(&mut self, q: Option<Q>) {
        self.ub = q
    }
}
impl<Q, F> AllocFilter for IntervalFilter<Q, F>
where
    Q: Quantity,
    for<'a> F: Fn(&'a Alloc) -> Q,
{
    fn filter<'a>(&self, alloc: &'a Alloc) -> bool {
        let q = (&self.f)(alloc);
        if let Some(lb) = self.lb.as_ref() {
            if !(lb <= &q) {
                return false;
            }
        }
        if let Some(ub) = self.ub.as_ref() {
            if !(&q <= ub) {
                return false;
            }
        }
        true
    }

    fn uid(&self) -> FilterUid {
        self.uid
    }
}

/// Filters about the size of an alloc.
pub mod size {
    use super::*;

    /// Interval filter over the size of an alloc.
    pub fn interval<S>(lb: Option<usize>, ub: Option<usize>) -> impl AllocFilter
    where
        S: Into<String>,
    {
        IntervalFilter::new(|alloc| alloc.size(), lb, ub)
    }
}
