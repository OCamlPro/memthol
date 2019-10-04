//! Handles the internals of location sharing.

use crate::Trace;

// Macro defined in `crate::mem`.
new! {
    mod mem for crate::Trace
}

/// A location UID.
pub struct Uid {
    uid: mem::Uid,
}

/// Registers a list of locations and returns its UID.
pub fn add(trace: Trace) -> Uid {
    let mut mem = mem::write();
    let uid = mem.get_uid(trace);
    Uid { uid }
}

/// Retrieves a list of locations from its UID.
pub fn get(uid: Uid) -> std::sync::Arc<Trace> {
    let mem = mem::read();
    mem.get_elm(uid.uid)
}
