//! Handles the internals of trace sharing.

use crate::CLoc;

// Macro defined in `crate::mem`.
new! {
    mod mem for Vec<super::CLoc>, uid: Trace
}

pub use mem::{AsRead, AsWrite, Trace};

/// Registers a list of locations and returns its UID.
pub fn add(trace: Vec<CLoc>) -> Trace {
    let mut mem = mem::write();
    mem.get_uid(trace)
}

/// Registers some lists of locations and returns its UID.
pub fn add_all(capa: usize, mut get_loc: impl FnMut() -> Option<Vec<CLoc>>) -> Vec<Trace> {
    let mut mem = mem::write();
    let mut res = Vec::with_capacity(capa);
    while let Some(locs) = get_loc() {
        res.push(mem.get_uid(locs))
    }
    res
}

/// Retrieves a list of locations from its UID.
pub fn get(uid: Trace) -> std::sync::Arc<Vec<CLoc>> {
    let mem = mem::read();
    mem.get_elm(uid)
}
