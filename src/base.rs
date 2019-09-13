//! Basic types and helpers used by the whole crate.

pub use std::{
    collections::BTreeMap as Map,
    collections::BTreeSet as Set,
    net::{TcpListener, TcpStream},
};

pub use alloc_data::{Alloc, Diff, Init as AllocInit, SinceStart, Uid as AllocUid};

pub use error_chain::bail;

pub use crate::{
    err,
    err::{Res, ResultExt},
};

/// A set of allocation UIDs.
pub type AllocUidSet = Set<AllocUid>;
