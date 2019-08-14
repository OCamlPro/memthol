//! Wrapper around a JS collection of allocations.

use crate::base::*;

pub mod filter;

pub use filter::Filter;

/// Aggregates all data and the global filters.
pub struct Storage {
    /// All data seen so far.
    all: Map<AllocUid, Alloc>,
    /// The global filter.
    filter: Filter,
}

impl Storage {
    /// Constructor.
    pub fn new() -> Self {
        Self {
            all: Map::new(),
            filter: Filter::new(),
        }
    }

    /// Registers some allocation data.
    pub fn add_alloc(&mut self, alloc: Alloc) -> Option<&Alloc> {
        let uid = alloc.uid().clone();
        let entry = self.all.entry(uid);
        debug_assert! {
            if let std::collections::btree_map::Entry::Occupied(_) = &entry {
                false
            } else {
                true
            }
        }
        let alloc = &*entry.or_insert(alloc);
        if self.filter.apply(alloc) {
            Some(alloc)
        } else {
            None
        }
    }

    /// Registers the death of some allocation data.
    pub fn add_death(&mut self, uid: &AllocUid, tod: AllocDate) -> bool {
        if let Some(alloc) = self.all.get_mut(&uid) {
            alloc
                .set_tod(tod)
                .expect("received inconsistent alloc/alloc death information");
            true
        } else {
            false
        }
    }

    /// Applies something to all the data received so far.
    ///
    /// The input function `f` takes an allocation and returns an option of something. It will be
    /// applied to all allocation data until either
    ///
    /// - `f` returns `Some(_)`, at which point `get` stops and returns `Some(_)`, or
    /// - there is no more allocation data.
    pub fn get<F, Out>(&self, mut f: F) -> Option<Out>
    where
        for<'a> F: FnMut(&'a Alloc) -> Option<Out>,
    {
        for (_, alloc) in self.all.iter() {
            if self.filter.apply(alloc) {
                if let Some(res) = f(alloc) {
                    return Some(res);
                }
            }
        }
        None
    }

    /// Applies something to all the data received so far.
    pub fn iter<F>(&self, mut f: F)
    where
        for<'a> F: FnMut(&'a Alloc),
    {
        let none = self.get::<_, ()>(|alloc| {
            f(alloc);
            None
        });
        debug_assert! { none.is_none() }
    }

    // /// Adds a filter.
    // pub fn add_filter(&mut self, filter: )
}
