//! Common imports for this crate.

pub use regex::Regex;

pub use base::prelude::{serde::*, *};

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

pub use alloc::Alloc;

/// Imports this crate's prelude.
macro_rules! prelude {
    () => {
        use $crate::prelude::*;
    };
}

base::cfg_item! {
    cfg(server) {
        pub use crate::{data, ChartExt};
    }
}

pub use crate::{
    chart::{self, settings::ChartSettings},
    color::Color,
    filter::{self, Filter, Filters},
    msg,
    point::{self, Point, PointVal, Points},
};

/// Number pretty formatting.
pub mod num_fmt {
    /// Applies an action to a pretty string representation of a number.
    ///
    /// ```rust
    /// # use charts::prelude::num_fmt::*;
    /// let mut s = String::new();
    /// str_do(16_504_670, |pretty| s = pretty);
    /// assert_eq!(&s, "16.50M");
    ///
    /// str_do(670, |pretty| s = pretty);
    /// assert_eq!(&s, "670");
    ///
    /// str_do(1_052_504_670u32, |pretty| s = pretty);
    /// assert_eq!(&s, "1.05G");
    /// ```
    pub fn str_do<Res>(
        stuff: impl std::convert::TryInto<f64> + std::fmt::Display + Clone,
        action: impl FnOnce(String) -> Res,
    ) -> Res {
        use number_prefix::NumberPrefix::{self, *};
        let s = match stuff.clone().try_into().map(NumberPrefix::decimal) {
            Ok(Prefixed(pref, val)) => format!("{:.2}{}", val, pref),
            Err(_) | Ok(Standalone(_)) => stuff.to_string(),
        };
        action(s)
    }

    /// Applies an action to a pretty string representation of a number.
    ///
    /// ```rust
    /// # use charts::prelude::num_fmt::*;
    /// let mut s = String::new();
    /// str_do(16_504_670, |pretty| s = pretty);
    /// assert_eq!(&s, "16.50M");
    ///
    /// str_do(670, |pretty| s = pretty);
    /// assert_eq!(&s, "670");
    ///
    /// str_do(1_052_504_670u32, |pretty| s = pretty);
    /// assert_eq!(&s, "1.05G");
    /// ```
    pub fn bin_str_do<Res>(
        stuff: impl std::convert::TryInto<f64> + std::fmt::Display + Clone,
        action: impl FnOnce(String) -> Res,
    ) -> Res {
        use number_prefix::NumberPrefix::{self, *};
        let s = match stuff.clone().try_into().map(NumberPrefix::binary) {
            Ok(Prefixed(pref, val)) => format!("{:.2}{}", val, pref),
            Err(_) | Ok(Standalone(_)) => stuff.to_string(),
        };
        action(s)
    }
}

/// A set of allocation UIDs.
pub type AllocUidSet = BTSet<uid::Alloc>;

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
