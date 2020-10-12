//! Common imports for this crate.

pub use std::{
    collections::{BTreeMap as Map, BTreeSet as Set},
    convert::{TryFrom, TryInto},
    fmt,
    ops::{Deref, DerefMut},
    sync::Arc,
};

pub use regex::Regex;

pub use base::{
    debug_do,
    error_chain::{self, bail},
    impl_display, lazy_static, Either,
};

/// Re-exports from the `alloc_data` crate.
pub mod alloc {
    pub use alloc_data::prelude::*;
}

/// Re-exports from `plotters`'s `coord` module.
pub mod coord {
    pub use plotters::coord::{
        cartesian::Cartesian2d,
        ranged1d::{AsRangedCoord, Ranged, ValueFormatter},
        types::{RangedCoordf32, RangedCoordu32, RangedDuration},
    };
}

pub use alloc::serderive::*;

pub use alloc::{
    time, Alloc, Date, Diff as AllocDiff, Duration, Init as AllocInit, Uid as AllocUid,
};

/// Imports this crate's prelude.
macro_rules! prelude {
    () => {
        use $crate::prelude::*;
    };
}

#[cfg(any(test, feature = "server"))]
pub use crate::data;

pub use crate::{
    chart::{self, settings::ChartSettings},
    color::Color,
    err,
    err::{Res, ResExt},
    filter::{self, Filter, Filters},
    msg, point,
    point::{Point, PointVal, Points},
    uid, ChartExt,
};

pub mod num_fmt {
    pub fn str_do<Res>(
        stuff: impl std::convert::TryInto<f64> + std::fmt::Display + Clone,
        action: impl Fn(String) -> Res,
    ) -> Res {
        use number_prefix::NumberPrefix::{self, *};
        let s = match stuff.clone().try_into().map(NumberPrefix::decimal) {
            Ok(Prefixed(pref, val)) => format!("{:.2}{}", val, pref),
            Err(_) | Ok(Standalone(_)) => stuff.to_string(),
        };
        action(s)
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

/// Dump-loading information.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LoadInfo {
    /// Number of dumps loaded so far.
    pub loaded: usize,
    /// Total number of dumps.
    ///
    /// Can be `0`, in which case the progress is considered to be `0` as well.
    pub total: usize,
}
impl LoadInfo {
    /// Unknown info, `loaded` and `total` are set to `0`.
    pub fn unknown() -> Self {
        Self {
            loaded: 0,
            total: 0,
        }
    }
    /// Percent version of the progress.
    pub fn percent(&self) -> f64 {
        if self.total == 0 {
            0.
        } else {
            (self.loaded as f64) * 100. / (self.total as f64)
        }
    }
}

/// Retrieve data errors.
#[cfg(any(test, feature = "server"))]
pub fn get_errors() -> Res<Option<Vec<String>>> {
    data::Data::get_errors()
}

/// Allocation statistics.
///
/// Sent to the client so that it can display basic informations (run date, allocation count...).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AllocStats {
    /// Dump directory.
    pub dump_dir: std::path::PathBuf,
    /// Total number of allocations.
    pub alloc_count: usize,
    /// Date at which the run started.
    pub start_date: time::Date,
    /// Duration of the run.
    pub duration: time::SinceStart,
}
#[cfg(any(test, feature = "server"))]
impl AllocStats {
    /// Constructor.
    pub fn new(dump_dir: impl Into<std::path::PathBuf>, start_date: time::Date) -> Self {
        let dump_dir = dump_dir.into();
        // let dump_dir = dump_dir.canonicalize().unwrap_or(dump_dir);
        Self {
            dump_dir,
            alloc_count: 0,
            start_date,
            duration: time::SinceStart::zero(),
        }
    }

    /// Allocation statistics accessor for the global data server-side.
    pub fn get() -> Res<Option<AllocStats>> {
        data::Data::get_stats()
    }
}
