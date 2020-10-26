//! Re-exports, types and helpers for all crates in this project.

#![deny(missing_docs)]

pub extern crate bincode;
pub extern crate chrono;
pub extern crate log;
pub extern crate peg;
pub extern crate rand;
pub use either::Either;

#[macro_use]
mod macros;

pub mod time;
pub mod time_stats;
pub mod uid;

pub mod prelude;

/// Re-exports from `error_chain`.
pub mod error_chain {
    pub use error_chain::*;
}

pub mod err;

/// Used to convert between integer representations.
#[inline]
pub fn convert<In, Out>(n: In, from: &'static str) -> Out
where
    In: std::convert::TryInto<Out> + std::fmt::Display + Copy,
    In::Error: std::fmt::Display,
{
    match n.try_into() {
        Ok(res) => res,
        Err(e) => panic!("[fatal] while converting {} ({}): {}", n, from, e),
    }
}

/// Returns what it's given.
pub fn identity<T>(t: T) -> T {
    t
}
/// Destroys what it's given.
pub fn destroy<T>(_: T) {}

/// Alias type for `SmallVec` of max stack-size 8.
pub type SVec8<T> = smallvec::SmallVec<[T; 8]>;
/// Alias type for `SmallVec` of max stack-size 16.
pub type SVec16<T> = smallvec::SmallVec<[T; 16]>;
/// Alias type for `SmallVec` of max stack-size 32.
pub type SVec32<T> = smallvec::SmallVec<[T; 32]>;
/// Alias type for `SmallVec` of max stack-size 64.
pub type SVec64<T> = smallvec::SmallVec<[T; 64]>;

/// Contains compilation directives for the WASM client.
#[macro_use]
pub mod client {
    /// Directory in which the WASM client is being built.
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

    /// Directory in which the WASM client is being built (not release version).
    #[macro_export]
    #[cfg(debug_assertions)]
    macro_rules! client_wasm_build_dir {
        () => {
            $crate::client_wasm_build_dir_for!(debug)
        };
    }

    /// Directory in which the WASM client is being built (release version).
    #[macro_export]
    #[cfg(not(debug_assertions))]
    macro_rules! client_wasm_build_dir {
        () => {
            $crate::client_wasm_build_dir_for!(release)
        };
    }
}
