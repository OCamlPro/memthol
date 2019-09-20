//! Macro that generates a `uid` module that can generate UIDs.

// use crate::base::*;

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
                fn deserialize<D>(deserializer: D) -> Result<$uid_type_name, D::Error>
                where
                    D: serde::Deserializer<'de>,
                {
                    deserializer.deserialize_str(UidVisitor)
                }
            }

            /// Private module for the factory.
            lazy_static::lazy_static! {
                /// Uid counter.
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
    mod filter_uid {
        /// Filter index.
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
        /// Sub-filter index.
        ///
        /// Used to refer to [`SubFilter`]s in a [`Filter`].
        ///
        /// [`SubFilter`]: ../filter/sub/struct.SubFilter.html (The SubFilter struct)
        /// [`Filter`]: ../filter/struct.Filter.html (The Filter struct)
        SubFilterUid
    }
}

new_uid! {
    mod chart_uid {
        /// Chart index.
        ///
        /// Used to refer to [`Chart`]s in a [`Charts`].
        ///
        /// [`Chart`]: ../chart/struct.Chart.html (The Chart struct)
        /// [`Charts`]: ../struct.Charts.html (The Charts struct)
        ChartUid
    }
}
