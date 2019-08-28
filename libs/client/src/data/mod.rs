//! Wrapper around a JS collection of allocations.

use crate::base::*;

pub mod filter;

pub use filter::Filter;

/// Aggregates all data and the global filters.
pub struct Storage {
    /// Init information.
    init: alloc_data::Init,
    /// Current date.
    current: AllocDate,
    /// History of all the diffs.
    history: Vec<AllocDiff>,
    /// All dead data seen so far.
    ///
    /// # Invariants
    ///
    /// - all allocs are such that `alloc.tod().is_some()`.
    dead: Map<AllocUid, Alloc>,
    /// All live data.
    ///
    /// # Invariants
    ///
    /// - all allocs are such that `alloc.tod().is_none()`.
    live: Map<AllocUid, Alloc>,
    /// The global filter.
    filter: Filter,
}

impl Storage {
    /// Constructor.
    pub fn new(init: alloc_data::Init) -> Self {
        let current = init.start_time;
        Self {
            init,
            current,
            history: Vec::with_capacity(103),
            dead: Map::new(),
            live: Map::new(),
            filter: Filter::new(),
        }
    }

    /// Start time.
    pub fn start_time(&self) -> AllocDate {
        self.init.start_time
    }

    /// Current time.
    pub fn current_time(&self) -> &AllocDate {
        &self.current
    }

    /// The most recent diff.
    pub fn last_diff(&self) -> Option<&AllocDiff> {
        self.history.last()
    }

    /// Allocation data of an allocation UID.
    pub fn get_alloc(&self, uid: &AllocUid) -> &Alloc {
        if let Some(alloc) = self.dead.get(uid) {
            alloc
        } else if let Some(alloc) = self.live.get(uid) {
            alloc
        } else {
            panic!("unknown allocation uid #{}", uid)
        }
    }

    /// Invariant.
    #[cfg(debug_assertions)]
    pub fn check_invariants(&self) {
        for (_, alloc) in &self.dead {
            assert! { alloc.tod().is_some() }
        }
        for (_, alloc) in &self.live {
            assert! { alloc.tod().is_none() }
        }
    }

    /// Invariant.
    #[cfg(not(debug_assertions))]
    #[inline]
    pub fn check_invariants(&self) {}

    /// Registers some allocation data.
    pub fn add_alloc(&mut self, alloc: Alloc) -> bool {
        let uid = alloc.uid().clone();
        let is_dead = alloc.tod().is_some();
        let active = self.filter.apply(&alloc);
        if is_dead {
            let prev = self.dead.insert(uid.clone(), alloc);
            assert_eq! { prev, None }
        } else {
            let prev = self.live.insert(uid.clone(), alloc);
            assert_eq! { prev, None }
        }
        self.check_invariants();
        active
    }

    /// Registers the death of some allocation data.
    ///
    /// Returns `true` if the allocation is included by the filter(s).
    pub fn add_dead(&mut self, uid: &AllocUid, tod: SinceStart) -> bool {
        if let Some(mut alloc) = self.live.remove(uid) {
            alloc
                .set_tod(tod)
                .expect("received inconsistent alloc/alloc death information");
            let active = self.filter.apply(&alloc);
            let prev = self.dead.insert(uid.clone(), alloc);
            assert_eq! { prev, None }
            self.check_invariants();
            active
        } else {
            panic!("unknown live allocation #{}", uid)
        }
    }

    /// Registers a diff.
    ///
    /// Returns `true` if something changed, and that something is not filtered out by the
    /// filter(s).
    pub fn add_diff(&mut self, diff: AllocDiff) -> bool {
        let mut changed = false;
        self.current = self.start_time();
        self.current.add(diff.time.clone());
        for alloc in &diff.new {
            let new_stuff = self.add_alloc(alloc.clone());
            changed = changed || new_stuff
        }
        for (uid, tod) in &diff.dead {
            let new_stuff = self.add_dead(uid, tod.clone());
            changed = changed || new_stuff
        }
        self.history.push(diff);
        changed
    }

    /// Iterator over dead data.
    ///
    /// Skips allocations that the filter(s) say we should ignore.
    pub fn dead_iter<F>(&self, mut f: F)
    where
        for<'a> F: FnMut(&'a Alloc),
    {
        for (_, alloc) in &self.dead {
            if self.filter.apply(alloc) {
                f(alloc)
            }
        }
    }

    /// Iterator over live data.
    ///
    /// Skips allocations that the filter(s) say we should ignore.
    pub fn live_iter<F>(&self, mut f: F)
    where
        for<'a> F: FnMut(&'a Alloc),
    {
        for (_, alloc) in &self.live {
            if self.filter.apply(alloc) {
                f(alloc)
            }
        }
    }

    /// Iterator over all data.
    ///
    /// Skips allocations that the filter(s) say we should ignore.
    pub fn iter<F>(&self, mut f: F)
    where
        for<'a> F: FnMut(bool, &'a Alloc),
    {
        self.dead_iter(|alloc| f(false, alloc));
        self.live_iter(|alloc| f(true, alloc))
    }

    /// Iterates over all the diffs.
    pub fn diff_iter<F>(&self, mut f: F)
    where
        for<'a> F: FnMut(&'a AllocDiff),
    {
        for diff in &self.history {
            f(diff)
        }
    }

    // /// Adds a filter.
    // pub fn add_filter(&mut self, filter: )
}
