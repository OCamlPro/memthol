//! Handles the internals of location sharing.

pub use serde_derive::{Deserialize, Serialize};

use crate::CLoc;

// Macro defined in `crate::mem`.
new! {
    mod mem for Vec<crate::CLoc>
}

/// A location UID.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Uid {
    uid: mem::Uid,
}

/// Registers a list of locations and returns its UID.
pub fn add(trace: Vec<CLoc>) -> Uid {
    let mut mem = mem::write();
    let uid = mem.get_uid(trace);
    Uid { uid }
}

/// Retrieves a list of locations from its UID.
pub fn get(uid: Uid) -> std::sync::Arc<Vec<CLoc>> {
    let mem = mem::read();
    mem.get_elm(uid.uid)
}
