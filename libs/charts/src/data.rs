//! Global data about allocations.

prelude! {}

use std::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};

mod watcher;

pub use watcher::Watcher;

/// Starts global data handling.
///
/// - runs the file watcher daemon.
pub fn start(target: impl AsRef<std::path::Path>) -> Res<()> {
    Watcher::spawn(target, false);
    Ok(())
}

lazy_static! {
    /// Progress indicator, used during loading.
    static ref PROG: RwLock<Option<LoadInfo>> = RwLock::new(Some(LoadInfo::unknown()));
    /// Global state.
    static ref DATA: RwLock<Data> = RwLock::new(Data::new());
}

/// Handles progress information.
pub mod progress {
    use super::*;

    fn read<'a>() -> Res<RwLockReadGuard<'a, Option<LoadInfo>>> {
        PROG.read()
            .map_err(|e| {
                let e: err::Err = e.to_string().into();
                e
            })
            .chain_err(|| "while reading the progress status")
    }
    fn write<'a>() -> Res<RwLockWriteGuard<'a, Option<LoadInfo>>> {
        PROG.write()
            .map_err(|e| {
                let e: err::Err = e.to_string().into();
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

    /// Retrieves the progress, if any.
    pub fn get() -> Res<Option<LoadInfo>> {
        read().map(|info| info.clone())
    }
}

/// Global state accessor.
pub fn get<'a>() -> Res<RwLockReadGuard<'a, Data>> {
    DATA.read()
        .map_err(|e| {
            let e: err::Err = e.to_string().into();
            e
        })
        .chain_err(|| "while reading the global state")
}

/// Total number of allocations.
pub fn alloc_count() -> Res<usize> {
    get().map(|data| data.uid_map.len())
}

/// Global state mutable accessor.
fn get_mut<'a>() -> Res<RwLockWriteGuard<'a, Data>> {
    DATA.write()
        .map_err(|e| {
            let e: err::Err = e.to_string().into();
            e
        })
        .chain_err(|| "while reading the global state")
}

/// Structures that aggregates all the information about the allocations so far.
pub struct Data {
    /// Init state.
    init: Option<AllocInit>,
    /// Map from allocation UIDs to allocation data.
    uid_map: Map<AllocUid, Alloc>,
    /// Map from time-of-death to allocation UIDs.
    tod_map: Map<time::SinceStart, AllocUidSet>,
    /// Errors encountered so far.
    errors: Vec<String>,
    /// Time of the latest diff.
    current_time: time::SinceStart,
    /// Statistics.
    stats: Option<AllocStats>,
}

impl Data {
    /// Constructor.
    pub fn new() -> Self {
        Self {
            init: None,
            uid_map: Map::new(),
            tod_map: Map::new(),
            errors: vec![],
            current_time: time::SinceStart::zero(),
            stats: None,
        }
    }

    /// Init accessor.
    pub fn init(&self) -> Option<&AllocInit> {
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
    pub fn start_time(&self) -> Res<Date> {
        if let Some(init) = self.init.as_ref() {
            Ok(init.start_time.clone())
        } else {
            bail!("cannot access start time")
        }
    }

    /// Alloc accessor.
    ///
    /// Fails if the UID is unknown.
    pub fn get_alloc(&self, uid: &AllocUid) -> Res<&Alloc> {
        self.uid_map
            .get(uid)
            .ok_or_else(|| format!("unknown allocation UID #{}", uid).into())
    }

    /// Runs some functions on new allocations and allocation deaths since some time in history.
    ///
    /// - new allocations that have a time-of-death **will also be** in `iter_dead_since`;
    /// - allocations will appear in reverse time-of-creation chronological order.
    pub fn iter_new_since(
        &self,
        time: &time::SinceStart,
        mut new_alloc: impl FnMut(&Alloc) -> Res<()>,
    ) -> Res<()> {
        // Reverse iter allocations.
        for (_, alloc) in self.uid_map.iter().rev() {
            if &alloc.toc <= time {
                break;
            } else {
                new_alloc(alloc)?
            }
        }

        Ok(())
    }

    /// Iterator over all the allocations.
    ///
    /// - allocations will appear in time-of-creation chronological order.
    pub fn iter_all(&self) -> impl Iterator<Item = &Alloc> {
        self.uid_map.values()
    }

    /// Runs some functions on new allocations and allocation deaths since some time in history.
    ///
    /// - new allocations that have a time-of-death **will also appear** in `iter_new_since`;
    /// - allocation deaths will appear in reverse time-of-death chronological order.
    pub fn iter_dead_since(
        &self,
        time: &time::SinceStart,
        mut new_death: impl FnMut(&AllocUidSet, &time::SinceStart) -> Res<()>,
    ) -> Res<()> {
        // Reverse iter death.
        for (tod, uid) in self.tod_map.iter().rev() {
            if tod <= time {
                break;
            } else {
                new_death(uid, tod)?
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

    /// Resets the data.
    ///
    /// Called when the init file of a run has changed.
    pub fn reset(&mut self, dump_dir: impl Into<std::path::PathBuf>, init: AllocInit) {
        self.stats = Some(AllocStats::new(dump_dir, init.start_time));
        self.init = Some(init);
        self.uid_map.clear();
        self.tod_map.clear();
        self.current_time = time::SinceStart::zero();
    }

    /// Registers a diff.
    pub fn add_diff(&mut self, diff: AllocDiff) -> Res<()> {
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

        for mut alloc in diff.new {
            // Force the allocation to have toc/tod map the diff's time.
            alloc.toc = diff.time;
            if let Some(tod) = alloc.tod.as_mut() {
                *tod = diff.time
            }
            let uid = alloc.uid.clone();

            if let Some(tod) = alloc.tod.clone() {
                let is_new = self.tod_map_get_mut(tod).insert(uid.clone());
                if !is_new {
                    bail!(
                        "allocation UID collision (1): two allocations have UID #{}",
                        uid
                    )
                }
            }

            let prev = self.uid_map.insert(uid.clone(), alloc);
            if prev.is_some() {
                bail!(
                    "allocation UID collision (2): two allocations have UID #{}",
                    uid
                )
            }
        }
        for (uid, _tod) in diff.dead {
            // Force TOD to be diff's time.
            let tod = diff.time;
            let is_new = self.tod_map_get_mut(tod).insert(uid.clone());
            if !is_new {
                bail!(
                    "allocation UID collision (3): two allocations have UID #{}",
                    uid
                )
            }

            match self.uid_map.get_mut(&uid) {
                Some(alloc) => alloc.set_tod(tod)?,
                None => bail!("cannot register death for unknown allocation UID #{}", uid),
            }
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

    /// Retrieves the global errors.
    pub fn get_errors() -> Res<Option<Vec<String>>> {
        get_mut().map(|mut data| data.errors())
    }

    /// Retrieves the errors.
    pub fn errors(&mut self) -> Option<Vec<String>> {
        if self.errors.is_empty() {
            None
        } else {
            Some(std::mem::replace(&mut self.errors, vec![]))
        }
    }
}

/// Adds an error.
pub fn add_err(err: impl Into<String>) {
    let err = err.into();
    println!("[data] Error:");
    for line in err.lines() {
        println!("[data] | {}", line)
    }
    get_mut()
        .chain_err(|| format!("while adding error:\n{}", err))
        .expect("failed to retrieve global state")
        .errors
        .push(err.into())
}

/// Registers a diff.
pub fn add_diff(diff: AllocDiff) -> Res<()> {
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
        for (_, alloc) in uid_map.iter() {
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
