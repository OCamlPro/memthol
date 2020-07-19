//! UID types for charts, filters and subfilters.
//!
//! All UID types implement serialize and deserialize.
//!
//! Types [`ChartUid`], [`FilterUid`] and [`SubFilterUid`] are the straightforward UIDs. This module
//! also has a [`LineUid`] type which augments [`FilterUid`] with two additional variants:
//!
//! - the "catch-all filter", which is the filter that catches everything the other filters do not
//!     catch;
//! - the "everything filter", which is the filter that catches **all** allocations, independently
//!     of the user-defined filters.
//!
//! [`ChartUid`]: struct.ChartUid.html (The ChartUid struct)
//! [`FilterUid`]: struct.FilterUid.html (The FilterUid struct)
//! [`LineUid`]: enum.LineUid.html (The LineUid enum)
//! [`SubFilterUid`]: struct.SubFilterUid.html (The SubFilterUid struct)

use std::fmt;

/// Creates UID-related types and a factory for UIDs.
macro_rules! new_uid {
    (
        mod $mod_name:ident {
            $(#[$uid_meta:meta])*
            $uid_type_name:ident
        }
    ) => {
        pub use $mod_name::$uid_type_name;
        mod $mod_name {
            use std::sync::Mutex;

            safe_index::new! {
                $(#[$uid_meta])*
                $uid_type_name,
            }

            impl serde::Serialize for $uid_type_name {
                fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                where
                    S: serde::Serializer,
                {
                    serializer.serialize_str(&self.to_string())
                }
            }
            struct UidVisitor;
            impl<'de> serde::de::Visitor<'de> for UidVisitor {
                type Value = $uid_type_name;

                fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                    formatter.write_str("a UID (usize)")
                }

                fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
                where E: serde::de::Error {
                    use std::str::FromStr;
                    usize::from_str(value).map(|index| $uid_type_name::from(index)).map_err(
                        |e| E::custom(e.to_string())
                    )
                }
            }
            impl<'de> serde::Deserialize<'de> for $uid_type_name {
                fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
                where
                    D: serde::Deserializer<'de>,
                {
                    deserializer.deserialize_str(UidVisitor)
                }
            }

            $crate::prelude::lazy_static! {
                /// Uid factory.
                static ref COUNTER: Mutex<usize> = Mutex::new(0);
            }

            impl $uid_type_name {
                /// Yields a fresh UID.
                pub fn fresh() -> $uid_type_name {
                    let mut factory = COUNTER.lock().unwrap_or_else(|e| {
                        panic!(
                            "[sync] unable to access UID factory for `{}`: {}",
                            stringify!($uid_type_name),
                            e
                        )
                    });
                    let uid = *factory;
                    *factory += 1;
                    uid.into()
                }
            }
        }
    };
}

new_uid! {
    mod chart_uid {
        /// Chart UID.
        ///
        /// Used to refer to [`Chart`]s in a [`Charts`].
        ///
        /// [`Chart`]: ../chart/struct.Chart.html (The Chart struct)
        /// [`Charts`]: ../struct.Charts.html (The Charts struct)
        ChartUid
    }
}

new_uid! {
    mod filter_uid {
        /// Filter UID.
        ///
        /// Used to refer to [`Filter`]s in a [`Filters`] (filter collection) and/or in a
        /// [`Charts`].
        ///
        /// [`Filter`]: ../filter/sub/struct.Filter.html (The Filter struct)
        /// [`Filters`]: ../filter/struct.Filters.html (The Filters struct)
        /// [`Charts`]: ../struct.Charts.html (The Charts struct)
        FilterUid
    }
}

new_uid! {
    mod sub_filter_uid {
        /// Sub-filter UID.
        ///
        /// Used to refer to [`SubFilter`]s in a [`Filter`].
        ///
        /// [`SubFilter`]: ../filter/sub/struct.SubFilter.html (The SubFilter struct)
        /// [`Filter`]: ../filter/struct.Filter.html (The Filter struct)
        SubFilterUid
    }
}

/// A UID for a line in the chart.
///
/// A line in the chart is either an actual filter, or the "catch-all" line, or the "everything"
/// line.
#[derive(Debug, Clone, Copy, PartialOrd, Ord, PartialEq, Eq)]
pub enum LineUid {
    /// An actual filter.
    Filter(FilterUid),
    /// The catch-all filter.
    CatchAll,
    /// The everything filter.
    Everything,
}

impl From<FilterUid> for LineUid {
    fn from(uid: FilterUid) -> LineUid {
        Self::Filter(uid)
    }
}

impl LineUid {
    /// The filter UID, if any.
    pub fn filter_uid(self) -> Option<FilterUid> {
        match self {
            Self::Filter(uid) => Some(uid),
            Self::CatchAll | Self::Everything => None,
        }
    }

    /// Y-axis key representation.
    pub fn y_axis_key(self) -> String {
        match self {
            Self::Filter(uid) => format!("y_{}", uid),
            Self::CatchAll => "y_catch_all".into(),
            Self::Everything => "y".into(),
        }
    }
}

/// String representing the `CatchAll` variant of `LineUid`.
const CATCH_ALL_STR: &str = "catch_all";
/// String representing the `Everything` variant of `LineUid`.
const EVERYTHING_STR: &str = "everything";

impl fmt::Display for LineUid {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Filter(uid) => uid.fmt(fmt),
            Self::CatchAll => CATCH_ALL_STR.fmt(fmt),
            Self::Everything => EVERYTHING_STR.fmt(fmt),
        }
    }
}

impl serde::Serialize for LineUid {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}
struct UidVisitor;
impl<'de> serde::de::Visitor<'de> for UidVisitor {
    type Value = LineUid;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a UID (usize), or `catch_all`, or `everything`")
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        use std::str::FromStr;
        if value == CATCH_ALL_STR {
            Ok(LineUid::CatchAll)
        } else if value == EVERYTHING_STR {
            Ok(LineUid::Everything)
        } else {
            usize::from_str(value)
                .map(|index| LineUid::Filter(FilterUid::from(index)))
                .map_err(|e| E::custom(e.to_string()))
        }
    }
}
impl<'de> serde::Deserialize<'de> for LineUid {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_str(UidVisitor)
    }
}
