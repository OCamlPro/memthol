//! Handles the internals of label sharing.

use base::SVec32;

use crate::prelude::Str;

// Macro defined in `crate::mem`.
new! {
    mod mem for SVec32<super::Str>, uid: Labels
}

pub use mem::{AsRead, AsWrite, Labels};

/// Registers a list of labels and returns its UID.
pub fn add(labels: SVec32<Str>) -> Labels {
    let mut mem = mem::write();
    mem.get_uid(labels)
}

/// Retrieves a list of labels from its UID.
pub fn get(uid: Labels) -> std::sync::Arc<SVec32<Str>> {
    let mem = mem::read();
    mem.get_elm(uid)
}
