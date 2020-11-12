//! UID types for charts, filters and subfilters.
//!
//! All UID types implement serialize and deserialize.
//!
//! Types [`Chart`], [`Filter`] and [`SubFilter`] are the straightforward UIDs. This module also has
//! a [`Line`] type which augments [`Filter`] with two additional variants:
//!
//! - the "catch-all filter", which is the filter that catches everything the other filters do not
//!   catch;
//! - the "everything filter", which is the filter that catches **all** allocations, independently
//!   of the user-defined filters.
//!
//! [`Chart`]: struct.Chart.html (The Chart struct)
//! [`Filter`]: struct.Filter.html (The Filter struct)
//! [`Line`]: enum.Line.html (The Line enum)
//! [`SubFilter`]: struct.SubFilter.html (The SubFilter struct)

use std::fmt;

/// Creates UID-related types and a factory for UIDs.
macro_rules! new_uids {
    () => {};
    (
        mod $mod_name:ident {
            $(#[$uid_meta:meta])*
            $uid_type_name:ident
            $(
                ,
                $(#[$map_meta:meta])*
                map: $map_name:ident with iter: $iter_name:ident
            )?
            $(
                ,
                fresh_fn: $fresh_name:ident
            )?
            $(,)?
        }
        $($tail:tt)*
    ) => {
        pub use $mod_name::{
            $uid_type_name,
            $($map_name, $iter_name,)?
        };
        mod $mod_name {
            safe_index::new! {
                $(#[$uid_meta])*
                $uid_type_name,
                $(
                    $(#[$map_meta])*
                    map: $map_name with iter: $iter_name,
                )?
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

            $(
                $crate::prelude::lazy_static! {
                    /// Uid factory.
                    static ref COUNTER: std::sync::Mutex<usize> = std::sync::Mutex::new(0);
                }

                impl $uid_type_name {
                    /// Yields a fresh UID.
                    pub fn $fresh_name() -> $uid_type_name {
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
            )?
        }

        new_uids! { $($tail)* }
    };
}

new_uids! {
    mod alloc_uid {
        /// Allocation UID.
        Alloc,
        /// Map from allocation UIDs to something.
        map: AllocMap with iter: AllocIter,
    }

    mod chart_uid {
        /// Chart UID.
        Chart,
        fresh_fn: fresh,
    }

    mod filter_uid {
        /// Filter UID.
        Filter,
        fresh_fn: fresh,
    }

    mod sub_filter_uid {
        /// Sub-filter UID.
        SubFilter,
        fresh_fn: fresh,
    }
}

implement! {
    impl From for Alloc {
        from u64 => |n| {
            use std::convert::TryFrom;
            usize::try_from(n).unwrap_or_else(
                |e| panic!(
                    "`{}_u64` is not a valid `usize`, cannot construct allocation UID:\n{}", n, e
                )
            ).into()
        }
    }
}

/// A UID for a line in the chart.
///
/// A line in the chart is either an actual filter, or the "catch-all" line, or the "everything"
/// line.
#[derive(Debug, Clone, Copy, PartialOrd, Ord, PartialEq, Eq, Hash)]
pub enum Line {
    /// An actual filter.
    Filter(Filter),
    /// The catch-all filter.
    CatchAll,
    /// The everything filter.
    Everything,
}

impl From<Filter> for Line {
    fn from(uid: Filter) -> Line {
        Self::Filter(uid)
    }
}

impl Line {
    /// The filter UID, if any.
    pub fn filter_uid(self) -> Option<Filter> {
        match self {
            Self::Filter(uid) => Some(uid),
            Self::CatchAll | Self::Everything => None,
        }
    }

    /// True if the filter is the `everything` filter.
    pub fn is_everything(self) -> bool {
        self == Self::Everything
    }
    /// True if the filter is the `catch_all` filter.
    pub fn is_catch_all(self) -> bool {
        self == Self::CatchAll
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

impl fmt::Display for Line {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Filter(uid) => uid.fmt(fmt),
            Self::CatchAll => line_uid::CATCH_ALL_STR.fmt(fmt),
            Self::Everything => line_uid::EVERYTHING_STR.fmt(fmt),
        }
    }
}

mod line_uid {
    use super::*;

    /// String representing the `CatchAll` variant of `Line`.
    pub const CATCH_ALL_STR: &str = "catch_all";
    /// String representing the `Everything` variant of `Line`.
    pub const EVERYTHING_STR: &str = "everything";

    impl serde::Serialize for Line {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            serializer.serialize_str(&self.to_string())
        }
    }
    struct UidVisitor;
    impl<'de> serde::de::Visitor<'de> for UidVisitor {
        type Value = Line;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("a UID (usize), or `catch_all`, or `everything`")
        }

        fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            use std::str::FromStr;
            if value == CATCH_ALL_STR {
                Ok(Line::CatchAll)
            } else if value == EVERYTHING_STR {
                Ok(Line::Everything)
            } else {
                usize::from_str(value)
                    .map(|index| Line::Filter(Filter::from(index)))
                    .map_err(|e| E::custom(e.to_string()))
            }
        }
    }
    impl<'de> serde::Deserialize<'de> for Line {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            deserializer.deserialize_str(UidVisitor)
        }
    }
}
