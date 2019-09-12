//! Wrapper around a JS collection of allocations.

use crate::base::*;

// pub mod filter;

pub use filter::Filter;

/// Aggregates all data and the global filters.
pub struct Storage {
    /// Init information.
    init: alloc_data::Init,
    /// Current date.
    current: SinceStart,
    /// History of all the diffs.
    history: Vec<AllocDiff>,
    /// All allocations.
    all_allocs: Map<AllocUid, Alloc>,
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
    filters: Vec<Filter>,
}

impl Storage {
    /// Constructor.
    pub fn new(init: alloc_data::Init, filters: Vec<Filter>) -> Self {
        Self {
            init,
            current: Duration::new(0, 0).into(),
            history: Vec::with_capacity(103),
            all_allocs: Map::new(),
            dead: Map::new(),
            live: Map::new(),
            filters,
        }
    }

    /// Start time.
    pub fn start_time(&self) -> AllocDate {
        self.init.start_time
    }

    /// Current time.
    pub fn current_time(&self) -> AllocDate {
        let mut current_time = self.init.start_time.clone();
        current_time.add(self.current);
        current_time
    }

    /// Current time since start.
    pub fn current_time_since_start(&self) -> &SinceStart {
        &self.current
    }

    /// The most recent diff.
    pub fn last_diff(&self) -> Option<AllocDiff> {
        self.history.last().map(|diff| self.filter_diff(diff))
    }

    /// Allocation data of an allocation UID.
    pub fn get_alloc(&self, uid: &AllocUid) -> &Alloc {
        if let Some(alloc) = self.dead.get(uid) {
            alloc
        } else if let Some(alloc) = self.live.get(uid) {
            alloc
        } else {
            fail!("unknown allocation uid #{}", uid)
        }
    }

    /// Filter accessor.
    pub fn filters(&self) -> &Vec<Filter> {
        &self.filters
    }

    /// Sets the filters.
    pub fn set_filters(&mut self, filters: Vec<Filter>) {
        self.filters = filters
    }

    /// Applies the filters to some allocation.
    pub fn filter(&self, alloc: &Alloc) -> bool {
        for filter in &self.filters {
            if !filter.apply(self, alloc) {
                return false;
            }
        }
        true
    }

    /// Splits a diff based on the filters.
    pub fn filter_diff(&self, diff: &AllocDiff) -> AllocDiff {
        let (start_time, mut new, mut dead) = (diff.time, diff.new.clone(), diff.dead.clone());
        new.retain(|alloc| self.filter(alloc));
        dead.retain(|(uid, tod)| {
            let mut alloc = match self.all_allocs.get(&uid) {
                Some(alloc) => alloc.clone(),
                None => fail!("unknown allocation UID #{}", uid),
            };
            std::mem::replace(&mut alloc.tod, Some(tod.clone()));
            self.filter(&alloc)
        });
        info!(
            "filter_diff:\n- new: {}/{}\n- dead: {}/{}",
            diff.new.len(),
            new.len(),
            diff.dead.len(),
            dead.len()
        );
        AllocDiff::new(start_time, new, dead)
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
        let active = self.filter(&alloc);
        if is_dead {
            let prev = self.dead.insert(uid.clone(), alloc.clone());
            assert_eq! { prev, None }
        } else {
            let prev = self.live.insert(uid.clone(), alloc.clone());
            assert_eq! { prev, None }
        }
        let prev = self.all_allocs.insert(uid.clone(), alloc);
        if prev.is_some() {
            fail!("found multiple allocations for UID #{}", uid)
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
            let prev = self.all_allocs.insert(uid.clone(), alloc.clone());
            if prev.is_none() {
                fail!("allocation #{} died but was not registered as living")
            }
            let active = self.filter(&alloc);
            let prev = self.dead.insert(uid.clone(), alloc);
            assert_eq! { prev, None }
            self.check_invariants();
            active
        } else {
            fail!("unknown live allocation #{}", uid)
        }
    }

    /// Registers a diff.
    ///
    /// Returns `true` if something changed, and that something is not filtered out by the
    /// filter(s).
    pub fn add_diff(&mut self, diff: AllocDiff) -> bool {
        let mut changed = false;
        self.current = diff.time;
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
            if self.filter(alloc) {
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
            if self.filter(alloc) {
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
            let diff = self.filter_diff(diff);
            f(&diff)
        }
    }
}
