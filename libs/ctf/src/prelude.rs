//! Common imports for the modules in this crate.

pub use base::prelude::*;

pub use crate::{parse::CanParse, *};

/// A duration since the start of the run as microseconds.
pub type Clock = u64;
/// A difference between two [`Clock`] values.
///
/// [`Clock`]: type.Clock.html (Clock type)
pub type DeltaClock = u64;

/// Type of allocation UIDs.
pub type AllocUid = u64;

/// Type of PIDs.
pub type Pid = u64;
