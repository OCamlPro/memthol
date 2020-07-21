//! Handles the internals of label sharing.

use crate::prelude::Str;

// Macro defined in `crate::mem`.
new! {
    mod mem for Vec<super::Str>, uid: Labels
}

pub use mem::{AsRead, AsWrite, Labels};

/// Registers a list of labels and returns its UID.
pub fn add(labels: Vec<Str>) -> Labels {
    let mut mem = mem::write();
    mem.get_uid(labels)
}

/// Retrieves a list of labels from its UID.
pub fn get(uid: Labels) -> std::sync::Arc<Vec<Str>> {
    let mem = mem::read();
    mem.get_elm(uid)
}
