//! Common imports for this crate.

pub use std::{
    collections::{BTreeMap as Map, BTreeSet as Set},
    convert::{TryFrom, TryInto},
    fmt,
    ops::{Deref, DerefMut},
    sync::Arc,
};

pub use regex::Regex;
pub use serde_derive::{Deserialize, Serialize};

pub use base::{
    error_chain::{self, bail},
    lazy_static,
};

/// Re-exports from the `alloc_data` crate.
pub mod alloc {
    pub use alloc_data::prelude::*;
}

pub use alloc::{
    time, Alloc, Date, Diff as AllocDiff, Duration, Init as AllocInit, Uid as AllocUid,
};

/// Imports this crate's prelude.
macro_rules! prelude {
    () => {
        use $crate::prelude::*;
    };
}

pub use crate::{
    chart,
    color::Color,
    data, err,
    err::{Res, ResExt},
    filter,
    filter::{Filter, Filters},
    msg, point,
    point::{Point, PointVal, Points},
    uid, ChartExt,
};

pub mod num_fmt {
    static LOCALE: num_format::Locale = num_format::Locale::en;

    pub fn str_do<Res>(
        stuff: &impl num_format::ToFormattedStr,
        action: impl Fn(&str) -> Res,
    ) -> Res {
        let mut buf = num_format::Buffer::default();
        buf.write_formatted(stuff, &LOCALE);
        action(buf.as_str())
    }
}

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
