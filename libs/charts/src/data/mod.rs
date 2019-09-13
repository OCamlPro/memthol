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
    pub init: Option<AllocInit>,
    /// Map from allocation UIDs to allocation data.
    pub uid_map: Map<AllocUid, Alloc>,
    /// Errors encountered so far.
    pub errors: Vec<String>,
    /// Time of the latest diff.
    pub current_time: SinceStart,
}

impl Data {
    /// Constructor.
    pub fn new() -> Self {
        Self {
            init: None,
            uid_map: Map::new(),
            errors: vec![],
            current_time: SinceStart::zero(),
        }
    }

    /// Registers a diff.
    pub fn add_diff(&mut self, diff: Diff) -> Res<()> {
        self.current_time = diff.time;

        for alloc in diff.new {
            let uid = alloc.uid.clone();

            let prev = self.uid_map.insert(uid.clone(), alloc);
            if prev.is_some() {
                bail!(
                    "allocation UID collision: two allocations have UID #{}",
                    uid
                )
            }
        }
        for (uid, tod) in diff.dead {
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
