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
                #[derive(serde_derive::Serialize, serde_derive::Deserialize)]
                $uid_type_name,
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
        Filter
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
        SubFilter
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
        Chart
    }
}
