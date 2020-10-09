//! Re-exports, types and helpers for all crates in this project.

pub extern crate rand;
pub extern crate smallvec;

pub use derive_more::*;
pub use either::Either;
pub use lazy_static::lazy_static;

/// Re-exports from `error_chain`.
pub mod error_chain {
    pub use error_chain::*;
}

pub fn identity<T>(t: T) -> T {
    t
}

pub fn pretty_time(dur: std::time::Duration) -> String {
    format!("{}.{:0>9}", dur.as_secs(), dur.subsec_nanos())
}
pub fn now() -> std::time::Instant {
    std::time::Instant::now()
}

/// Alias type for `SmallVec` of max stack-size 8.
pub type SVec<T> = smallvec::SmallVec<[T; 8]>;
/// Alias type for `SmallVec` of max stack-size 16.
pub type SVec16<T> = smallvec::SmallVec<[T; 16]>;

/// Alias macro for smallvec construction.
#[macro_export]
macro_rules! svec {
    ($($stuff:tt)*) => {
        $crate::smallvec::smallvec!($($stuff)*)
    };
}

#[macro_use]
pub mod macros;

#[macro_use]
pub mod client {
    #[macro_export]
    macro_rules! client_wasm_build_dir_for {
        (release) => {
            "target/client.wasm/release/"
        };
        (debug) => {
            "target/client.wasm/debug/"
        };
        ($profile:tt bins) => {
            vec![
                concat!(
                    $crate::client_wasm_build_dir_for!($profile),
                    "client_bg.wasm"
                ),
                concat!($crate::client_wasm_build_dir_for!($profile), "client.js"),
            ]
        };
    }

    #[macro_export]
    #[cfg(debug_assertions)]
    macro_rules! client_wasm_build_dir {
        () => {
            $crate::client_wasm_build_dir_for!(debug)
        };
    }

    #[macro_export]
    #[cfg(not(debug_assertions))]
    macro_rules! client_wasm_build_dir {
        () => {
            $crate::client_wasm_build_dir_for!(release)
        };
    }
}
