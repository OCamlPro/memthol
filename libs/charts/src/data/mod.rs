//! Global data about allocations.

use std::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};

use lazy_static::lazy_static;

use crate::base::*;

mod watcher;

pub use watcher::Watcher;

/// Starts global data handling.
///
/// - runs the file watcher daemon.
pub fn start<S>(dir: S)
where
    S: Into<String>,
{
    Watcher::spawn(dir)
}

lazy_static! {
    /// Global state.
    static ref DATA: RwLock<Data> = RwLock::new(Data::new());
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
    tod_map: Map<SinceStart, AllocUidSet>,
    /// Errors encountered so far.
    errors: Vec<String>,
    /// Time of the latest diff.
    current_time: SinceStart,
}

impl Data {
    /// Constructor.
    pub fn new() -> Self {
        Self {
            init: None,
            uid_map: Map::new(),
            tod_map: Map::new(),
            errors: vec![],
            current_time: SinceStart::zero(),
        }
    }

    /// Current time accessor.
    pub fn current_time(&self) -> &SinceStart {
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
    pub fn iter_new_since<AllocDo>(&self, time: &SinceStart, mut new_alloc: AllocDo) -> Res<()>
    where
        AllocDo: for<'a> FnMut(&'a Alloc) -> Res<()>,
    {
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

    /// Runs some functions on new allocations and allocation deaths since some time in history.
    ///
    /// - new allocations that have a time-of-death **will also appear** in `iter_new_since`;
    /// - allocation deaths will appear in reverse time-of-death chronological order.
    pub fn iter_dead_since<DeathDo>(&self, time: &SinceStart, mut new_death: DeathDo) -> Res<()>
    where
        DeathDo: for<'a> FnMut(&'a AllocUidSet, &'a SinceStart) -> Res<()>,
    {
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
    fn tod_map_get_mut(&mut self, time: SinceStart) -> &mut AllocUidSet {
        self.tod_map.entry(time).or_insert_with(AllocUidSet::new)
    }

    /// Resets the data.
    ///
    /// Called when the init file of a run has changed.
    pub fn reset(&mut self, init: AllocInit) {
        self.init = Some(init);
        self.uid_map.clear();
        self.tod_map.clear();
        self.current_time = SinceStart::zero()
    }

    /// Registers a diff.
    pub fn add_diff(&mut self, diff: Diff) -> Res<()> {
        self.current_time = diff.time;

        for alloc in diff.new {
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
        for (uid, tod) in diff.dead {
            let is_new = self.tod_map_get_mut(tod).insert(uid.clone());
            if !is_new {
                bail!(
                    "allocation UID collision (3): two allocations have UID #{}",
                    uid
                )
            }

            match self.uid_map.get_mut(&uid) {
                Some(alloc) => alloc.set_tod(tod).map_err(|e| {
                    let e: err::Err = e.into();
                    e
                })?,
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
}

/// Adds an error.
pub fn add_err<S>(err: S)
where
    S: Into<String>,
{
    let err = err.into();
    println!("Error:");
    for line in err.lines() {
        println!("| {}", line)
    }
    get_mut()
        .chain_err(|| format!("while adding error:\n{}", err))
        .expect("failed to retrieve global state")
        .errors
        .push(err.into())
}

/// Registers a diff.
pub fn add_diff(diff: Diff) -> Res<()> {
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
