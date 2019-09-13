//! Basic types and helpers for this crate.

pub use std::{
    collections::{BTreeMap as Map, BTreeSet as Set},
    fmt,
};

pub use error_chain::bail;

pub use alloc_data::{Alloc, Date, Diff, Init as AllocInit, SinceStart, Uid as AllocUid};

pub use crate::{
    data, err,
    err::{Res, ResExt},
    filter,
    filter::Filter,
    time,
};

/// A set of allocation UIDs.
pub type AllocUidSet = Set<AllocUid>;

/// A point.
///
/// A point is a `key`, which is the x-value of the point. Then, there is one value for each filter
/// and a value for `rest`, the catch-all line of the graph.
pub struct Point<Key, Val> {
    /// X-value.
    pub key: Key,
    /// Values for filter lines.
    pub filtered: Vec<Val>,
    /// Catch-all value.
    pub rest: Val,
}

/// A point for a time chart.
pub type TimePoint<Val> = Point<Date, Val>;
