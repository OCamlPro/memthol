//! Basic types and helpers for this crate.

pub use std::{
    collections::{BTreeMap as Map, BTreeSet as Set},
    fmt,
    ops::{Deref, DerefMut},
};

pub use error_chain::bail;

// pub use serde::{Deserialize, Serialize};
pub use serde_derive::{Deserialize, Serialize};

pub use alloc_data::{Alloc, Date, Diff, Init as AllocInit, SinceStart, Uid as AllocUid};

pub use crate::{
    chart, data, err,
    err::{Res, ResExt},
    filter,
    filter::{Filter, Filters},
    index, msg, point,
    point::{Point, PointVal, Points},
    uid, ChartExt,
};

/// A set of allocation UIDs.
pub type AllocUidSet = Set<AllocUid>;

/// Trait for types that can be (de)serialized in JSON format.
pub trait Json: Sized {
    /// Json serialization.
    fn as_json(&self) -> Res<String>;
    /// Json serialization, pretty version.
    fn as_pretty_json(&self) -> Res<String>;
    /// Json deserialization.
    fn from_json(text: &str) -> Res<Self>;
    /// Json deserialization (bytes).
    fn from_json_bytes(bytes: &[u8]) -> Res<Self>;
}
impl<T> Json for T
where
    T: Sized + serde::Serialize + for<'a> serde::Deserialize<'a>,
{
    fn as_json(&self) -> Res<String> {
        let tml = serde_json::to_string(self)?;
        Ok(tml)
    }
    fn as_pretty_json(&self) -> Res<String> {
        let tml = serde_json::to_string_pretty(self)?;
        Ok(tml)
    }
    fn from_json(text: &str) -> Res<Self> {
        let slf = serde_json::from_str(text.as_ref())?;
        Ok(slf)
    }
    fn from_json_bytes(bytes: &[u8]) -> Res<Self> {
        let slf = serde_json::from_slice(bytes)?;
        Ok(slf)
    }
}
