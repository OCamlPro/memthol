//! Handles the internals of trace sharing.

use base::SVec32;

use crate::CLoc;

// Macro defined in `crate::mem`.
new! {
    mod mem for SVec32<super::CLoc>, uid: Trace
}

pub use mem::{AsRead, AsWrite, Trace};

/// Registers a list of locations and returns its UID.
pub fn add(trace: SVec32<CLoc>) -> Trace {
    let mut mem = mem::write();
    mem.get_uid(trace)
}

/// Registers some lists of locations and returns its UID.
pub fn add_all(capa: usize, mut get_loc: impl FnMut() -> Option<SVec32<CLoc>>) -> Vec<Trace> {
    let mut mem = mem::write();
    let mut res = Vec::with_capacity(capa);
    while let Some(locs) = get_loc() {
        res.push(mem.get_uid(locs))
    }
    res
}

/// Retrieves a list of locations from its UID.
pub fn get(uid: Trace) -> std::sync::Arc<SVec32<CLoc>> {
    let mem = mem::read();
    mem.get_elm(uid)
}
