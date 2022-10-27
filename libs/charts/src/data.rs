/*<LICENSE>
    This file is part of Memthol.

    Copyright (C) 2020 OCamlPro.

    Memthol is free software: you can redistribute it and/or modify
    it under the terms of the GNU General Public License as published by
    the Free Software Foundation, either version 3 of the License, or
    (at your option) any later version.

    Memthol is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
    GNU General Public License for more details.

    You should have received a copy of the GNU General Public License
    along with Memthol.  If not, see <https://www.gnu.org/licenses/>.
*/

//! Global data about allocations.

prelude! {}

mod watcher;

pub use watcher::Watcher;

/// Factory used when parsing dump-data.
///
/// The role of this factory is to get write-locks over the different factories needed at
/// parse-time. This avoids asking for the lock each time it is needed.
pub struct FullFactory<'a> {
    /// Lock over the allocation-data factories.
    factory: alloc_data::mem::Factory<'a>,
    /// Lock over the `Data` structure storing the whole dump.
    data: sync::RwLockWriteGuard<'a, Data>,
}

impl<'a> std::ops::Deref for FullFactory<'a> {
    type Target = alloc_data::mem::Factory<'a>;
    fn deref(&self) -> &Self::Target {
        &self.factory
    }
}
impl<'a> std::ops::DerefMut for FullFactory<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.factory
    }
}

impl<'a> FullFactory<'a> {
    /// Constructor.
    pub fn new(callstack_is_rev: bool) -> Self {
        Self {
            factory: alloc_data::mem::Factory::new(callstack_is_rev),
            data: get_mut().unwrap(),
        }
    }

    /// Builds a new allocation.
    pub fn build_new(&mut self, alloc: alloc::Builder) -> Res<()> {
        self.data.build_new(alloc)
    }
    /// Registers an allocation.
    pub fn add_new(&mut self, alloc: Alloc) -> Res<()> {
        self.data.add_new(alloc)
    }
    /// Registers the death of an allocation.
    pub fn add_dead(&mut self, timestamp: time::SinceStart, uid: uid::Alloc) -> Res<()> {
        self.data.add_dead(timestamp, uid)
    }

    /// Fills the statistics of the underlying data structure for the whole dump.
    pub fn fill_stats(&mut self) -> Res<()> {
        self.data.fill_stats()
    }

    /// Marks a timestamp.
    pub fn mark_timestamp(&mut self, ts: time::SinceStart) {
        self.data.mark_timestamp(ts)
    }
}

/// Starts global data handling.
///
/// - runs the file watcher daemon.
pub fn start(target: impl AsRef<std::path::Path>) -> Res<()> {
    Watcher::spawn(target, false);
    Ok(())
}

lazy_static! {
    /// Progress indicator, used during loading.
    static ref PROG: sync::RwLock<Option<LoadInfo>> = sync::RwLock::new(Some(LoadInfo::unknown()));
    /// Global state.
    static ref DATA: sync::RwLock<Data> = sync::RwLock::new(Data::new());
    /// Errors.
    static ref ERRORS: sync::RwLock<Vec<String>> = sync::RwLock::new(vec![]);
}

/// Handles progress information.
pub mod progress {
    use super::*;

    /// Read-lock over the global progress data.
    fn read<'a>() -> Res<sync::RwLockReadGuard<'a, Option<LoadInfo>>> {
        PROG.read()
            .map_err(|e| {
                let e: err::Error = e.to_string().into();
                e
            })
            .chain_err(|| "while reading the progress status")
    }
    /// Write-lock over the global progress data.
    fn write<'a>() -> Res<sync::RwLockWriteGuard<'a, Option<LoadInfo>>> {
        PROG.write()
            .map_err(|e| {
                let e: err::Error = e.to_string().into();
                e
            })
            .chain_err(|| "while writing the progress status")
    }

    /// Sets the progress to *unknown*.
    ///
    /// Used by the watcher before it knows how many dumps it needs to parse.
    pub fn set_unknown() -> Res<()> {
        write().map(|mut prog| *prog = Some(LoadInfo::unknown()))
    }

    /// Removes the progress, meaning loading is over.
    pub fn set_done() -> Res<()> {
        *write()? = None;
        Ok(())
    }

    /// Sets the total number of dumps to load.
    ///
    /// Also sets the number of dumps loaded to `0`.
    pub fn set_total(total: usize) -> Res<()> {
        let mut prog = write()?;
        *prog = Some(LoadInfo { total, loaded: 0 });
        Ok(())
    }
    /// Sets the number of dumps loaded.
    pub fn set_loaded(loaded: usize) -> Res<()> {
        let mut prog = write()?;
        if let Some(prog) = prog.as_mut() {
            prog.loaded = loaded;
        }
        Ok(())
    }

    /// Increments the number of dumps loaded.
    pub fn inc_loaded() -> Res<()> {
        if let Some(mut prog) = write()?.as_mut() {
            prog.loaded += 1;
        }
        Ok(())
    }
    /// Adds to the number of dumps loaded.
    pub fn add_loaded(n: usize) -> Res<()> {
        if let Some(mut prog) = write()?.as_mut() {
            prog.loaded += n;
        }
        Ok(())
    }

    /// Retrieves the progress, if any.
    pub fn get() -> Res<Option<LoadInfo>> {
        read().map(|info| info.clone())
    }
}

/// Global data read-accessor.
pub fn get<'a>() -> Res<sync::RwLockReadGuard<'a, Data>> {
    DATA.read()
        .map_err(|e| {
            let e: err::Error = e.to_string().into();
            e
        })
        .chain_err(|| "while reading the global state")
}

/// Total number of allocations.
pub fn alloc_count() -> Res<usize> {
    get().map(|data| data.uid_map.len())
}

/// Global data write-accessor.
fn get_mut<'a>() -> Res<sync::RwLockWriteGuard<'a, Data>> {
    DATA.write()
        .map_err(|e| {
            let e: err::Error = e.to_string().into();
            e
        })
        .chain_err(|| "while reading the global state")
}

/// Structures that aggregates all the information about the allocations so far.
pub struct Data {
    /// Init state.
    init: Option<alloc::Init>,
    /// Map from allocation UIDs to allocation data.
    uid_map: uid::AllocMap<Alloc>,
    /// Map from time-of-death to allocation UIDs.
    tod_map: BTMap<time::SinceStart, BTSet<uid::Alloc>>,
    /// Time of the latest diff.
    current_time: time::SinceStart,
    /// Statistics.
    stats: Option<AllocStats>,
}

impl ops::Index<uid::Alloc> for Data {
    type Output = Alloc;
    fn index(&self, uid: uid::Alloc) -> &Alloc {
        &self.uid_map[uid]
    }
}

impl Data {
    /// Constructor.
    pub fn new() -> Self {
        Self {
            init: None,
            uid_map: uid::AllocMap::new(),
            tod_map: BTMap::new(),
            current_time: time::SinceStart::zero(),
            stats: None,
        }
    }

    /// Reserves space for the `Alloc` vector.
    pub fn reserve(&mut self, capa: usize) {
        self.uid_map.reserve(capa)
    }

    /// Marks a timestamp.
    ///
    /// This sets the current time to the input timestamp.
    pub fn mark_timestamp(&mut self, ts: time::SinceStart) {
        self.current_time = ts
    }

    /// Init accessor.
    pub fn init(&self) -> Option<&alloc::Init> {
        self.init.as_ref()
    }
    /// True if the data is initialized.
    pub fn has_init(&self) -> bool {
        self.init().is_some()
    }

    /// Total number of allocations.
    pub fn alloc_count(&self) -> usize {
        self.uid_map.len()
    }

    /// Allocation statistics stored in the global data.
    pub fn get_stats() -> Res<Option<AllocStats>> {
        get().map(|data| data.stats())
    }

    /// Allocation statistics.
    pub fn stats(&self) -> Option<AllocStats> {
        self.stats.clone()
    }

    /// Current time accessor.
    pub fn current_time(&self) -> &time::SinceStart {
        &self.current_time
    }

    /// Time at which the profiling run started.
    pub fn start_time(&self) -> Res<time::Date> {
        if let Some(init) = self.init.as_ref() {
            Ok(init.start_time.clone())
        } else {
            bail!("cannot access start time")
        }
    }

    /// Alloc accessor.
    ///
    /// Fails if the UID is unknown.
    pub fn get_alloc(&self, uid: uid::Alloc) -> Option<&Alloc> {
        self.uid_map.get(uid)
    }

    /// Iterates over all the allocations.
    pub fn iter_allocs(&self) -> impl Iterator<Item = &Alloc> {
        self.uid_map.iter()
    }

    /// True if there are any new events since some timestamp.
    pub fn has_new_stuff_since(&self, time: Option<(uid::Alloc, time::SinceStart)>) -> bool {
        if let Some((uid, tod)) = time {
            self.uid_map[uid..].is_empty() || self.tod_map.keys().rev().next() != Some(&tod)
        } else {
            !self.uid_map.is_empty()
        }
    }

    /// Yields the last events at the current time.
    pub fn last_events(&self) -> Option<(uid::Alloc, time::SinceStart)> {
        self.uid_map.last().map(|alloc| {
            (
                alloc.0,
                self.tod_map
                    .keys()
                    .cloned()
                    .last()
                    .unwrap_or_else(time::SinceStart::zero),
            )
        })
    }

    /// Iterates over the new (de)allocation events in chronological order.
    ///
    /// Argument `since` is an optional pair containing an allocation UID, and a time-of-death
    /// (TOD). Input `action` will run on all new allocations since the UID (exclusive), and all the
    /// deallocations since the TOD (exclusive).
    ///
    /// Input function `action` returns a boolean indicating whether the iteration should continue.
    pub fn iter_new_events<'me>(
        &'me self,
        since: Option<(uid::Alloc, time::SinceStart)>,
        mut action: impl FnMut(Either<&'me Alloc, (time::SinceStart, &'me Alloc)>) -> Res<bool>,
    ) -> Res<()> {
        let (mut new_iter, mut dead_iter) = if let Some((last_alloc, last_time)) = since {
            let mut alloc_iter = self.uid_map[last_alloc..].iter();
            // First element is `last_alloc`, skipping it.
            let _alloc = alloc_iter.next();
            debug_assert!(_alloc.unwrap().uid == last_alloc);

            let last_time = last_time + time::SinceStart::from_nano_timestamp(0, 1);

            (alloc_iter, self.tod_map.range(last_time..))
        } else {
            (
                self.uid_map.iter(),
                self.tod_map.range(time::SinceStart::zero()..),
            )
        };

        let (mut next_new, mut next_dead) = (new_iter.next(), dead_iter.next());
        let mut keep_going = true;

        macro_rules! work {
            (new: $alloc:expr) => {{
                let cont = action(Either::Left($alloc))?;
                if !cont {
                    keep_going = false
                }
                next_new = new_iter.next();
            }};
            (dead: $tod:expr, $uids:expr) => {{
                for uid in $uids {
                    let alloc = &self.uid_map[*uid];
                    let cont = action(Either::Right(($tod, alloc)))?;
                    if !cont {
                        keep_going = false
                    }
                }
                next_dead = dead_iter.next();
            }};
        }

        while keep_going {
            match (next_new, next_dead) {
                (Some(alloc), None) => {
                    work!(new: alloc);
                    next_dead = None;
                }
                (None, Some((tod, uids))) => {
                    work!(dead: *tod, uids);
                    next_new = None;
                }
                (Some(alloc), Some((tod, uids))) => {
                    if &alloc.toc <= tod {
                        work!(new: alloc);
                        next_dead = Some((tod, uids));
                    } else {
                        work!(dead: *tod, uids);
                        next_new = Some(alloc);
                    }
                }
                (None, None) => break,
            }
        }

        Ok(())
    }
}

/// # Mutable Functions
impl Data {
    /// Mutable reference to `self.tod_map[tod]`.
    fn tod_map_get_mut(&mut self, time: time::SinceStart) -> &mut AllocUidSet {
        self.tod_map.entry(time).or_insert_with(AllocUidSet::new)
    }

    /// Applies some action on the allocation statistics.
    pub fn stats_do(&mut self, action: impl FnOnce(&mut AllocStats)) {
        if let Some(stats) = self.stats.as_mut() {
            action(stats)
        }
    }

    /// Fills the allocation statistics.
    pub fn fill_stats(&mut self) -> Res<()> {
        let stats = self
            .stats
            .as_mut()
            .ok_or_else(|| "[charts data] trying to fill stats of uninitialized data")?;
        stats.alloc_count = self.uid_map.len();
        stats.duration = self.current_time;
        Ok(())
    }

    /// Resets the data.
    ///
    /// Called when the init file of a run has changed.
    pub fn reset(&mut self, dump_dir: impl Into<std::path::PathBuf>, init: alloc::Init) {
        self.stats = Some(AllocStats::new(dump_dir, init.start_time));
        self.init = Some(init);
        self.uid_map.clear();
        self.tod_map.clear();
        self.current_time = time::SinceStart::zero();
    }

    /// Builds a new allocation.
    pub fn build_new(&mut self, alloc: alloc::Builder) -> Res<()> {
        if self.current_time != alloc.toc {
            self.current_time = alloc.toc.clone()
        }
        let uid = self.uid_map.next_index();
        let alloc = alloc.build(
            &self
                .init
                .as_ref()
                .ok_or_else(|| "trying to build allocation without initialization")?
                .sample_rate,
            uid,
        )?;

        self.add_new(alloc)
    }

    /// Registers a new allocation.
    pub fn add_new(&mut self, alloc: Alloc) -> Res<()> {
        self.stats
            .as_mut()
            .ok_or_else(|| "trying to add allocation before initialization")?
            .total_size += alloc.real_size as u64;
        self.current_time = alloc.toc;
        let uid = self.uid_map.next_index();
        if uid != alloc.uid {
            bail!(
                "unexpected allocation index {}, expected {}",
                alloc.uid,
                uid
            )
        }

        if let Some(tod) = alloc.tod.clone() {
            self.add_dead(tod, uid.clone())?
        }

        let uid_check = self.uid_map.push(alloc);
        debug_assert!(uid == uid_check);

        Ok(())
    }

    /// Registers an allocation's death.
    pub fn add_dead(&mut self, timestamp: time::SinceStart, uid: uid::Alloc) -> Res<()> {
        self.uid_map[uid].set_tod(timestamp)?;
        self.current_time = timestamp;
        let is_new = self.tod_map_get_mut(timestamp).insert(uid.clone());
        if !is_new {
            bail!(
                "allocation UID collision (1): two allocations have UID #{}",
                uid
            )
        }
        Ok(())
    }

    /// Registers a diff.
    pub fn add_diff(&mut self, diff: alloc::Diff) -> Res<()> {
        self.current_time = diff.time;

        if let Some(stats) = self.stats.as_mut() {
            stats.alloc_count += diff.new.len();
            stats.duration = diff.time;
        } else {
            if self.init.is_some() {
                bail!("inconsistent state, adding diff to data with init but no statistics")
            } else {
                bail!("inconsistent state, adding diff to data with no init")
            }
        }

        for alloc in diff.new {
            self.build_new(alloc)?
        }
        for (uid, tod) in diff.dead {
            self.add_dead(tod, uid)?
        }
        self.check_invariants().chain_err(|| "after adding diff")?;
        Ok(())
    }

    /// Checks that all data invariants hold.
    ///
    /// - only active in `debug`, does nothing in `release`.
    #[cfg(not(debug_assertions))]
    #[inline(always)]
    fn check_invariants(&self) -> Res<()> {
        Ok(())
    }

    /// Checks that all data invariants hold.
    ///
    /// - only active in `debug`, does nothing in `release`.
    #[cfg(debug_assertions)]
    fn check_invariants(&self) -> Res<()> {
        invariants::uid_order_is_toc_order(self)?;
        Ok(())
    }
}

/// Registers a diff.
pub fn add_diff(diff: alloc::Diff) -> Res<()> {
    let mut data = get_mut().chain_err(|| "while registering a diff")?;
    data.add_diff(diff)?;
    Ok(())
}

/// Data invariants.
pub mod invariants {
    use super::*;

    /// Map from alloc UIDs to alloc info is ordered by time-of-creation.
    pub fn uid_order_is_toc_order(data: &Data) -> Res<()> {
        let uid_map = &data.uid_map;
        let mut prev_toc = None;
        for alloc in uid_map.iter() {
            if let Some(prev_toc) = prev_toc {
                if prev_toc > &alloc.toc {
                    bail!("[data::invariants::uid_order_is_toc_order] invariant does not hold")
                }
            }

            prev_toc = Some(&alloc.toc)
        }
        Ok(())
    }
}
