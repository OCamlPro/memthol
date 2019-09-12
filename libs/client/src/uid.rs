//! Macro that generates a `uid` module that can generate UIDs.

/// Creates UID-related types and a factory for UIDs.
macro_rules! new_uid {
    (
        mod $mod_name:ident {
            uid: $uid_type_name:ident,
            set: $uid_set_type_name:ident,
            map: $uid_map_type_name:ident
            $(,)*
        }
    ) => {
        mod $mod_name {
            use std::sync::Mutex;

            safe_index::new! {
                /// Type of UIDs.
                $uid_type_name,
                /// Map from UIDs to something.
                btree map: $uid_map_type_name,
                /// Set of UIDs.
                btree set: $uid_set_type_name,
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
                        use crate::base::*;
                        fail!(
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
