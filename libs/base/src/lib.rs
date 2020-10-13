//! Re-exports, types and helpers for all crates in this project.

pub extern crate chrono;
pub extern crate peg;
pub extern crate rand;
pub extern crate smallvec;

pub use derive_more::*;
pub use either::Either;
pub use lazy_static::lazy_static;

#[macro_use]
pub mod macros;

/// Re-exports from `error_chain`.
pub mod error_chain {
    pub use error_chain::*;
}

/// Errors, handled by `error_chain`.
pub mod err {
    error_chain::error_chain! {
        types {
            Err, ErrKind, ResExt, Res;
        }

        foreign_links {
            Peg(peg::error::ParseError<peg::str::LineCol>)
            /// Parse error from `peg`.
            ;
        }

        links {}
        errors {}
    }

    pub use error_chain::bail;
}

/// Used to convert between integer representations.
#[cfg(any(test, not(release)))]
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

/// Used to convert between integer representations.
#[cfg(not(any(test, not(release))))]
#[inline]
pub fn convert<In, Out>(n: In, from: &'static str) -> Out
where
    In: std::convert::TryInto<Out> + std::fmt::Display + Copy,
    In::Error: std::fmt::Display,
{
    unsafe { std::mem::transmute(n) }
}

/// Returns what it's given.
pub fn identity<T>(t: T) -> T {
    t
}
/// Destroys what it's given.
pub fn destroy<T>(_: T) {}

/// Turns a number of milliseconds into a timpstamp.
pub fn duration_from_millis(ts: u64) -> std::time::Duration {
    let secs = ts / 1_000_000;
    let micros = ts - secs * 1_000_000;
    std::time::Duration::new(secs, (micros as u32) * 1_000)
}

/// Pretty string for a duration.
pub fn pretty_time(duration: std::time::Duration) -> String {
    format!("{}.{:0>9}", duration.as_secs(), duration.subsec_nanos())
}
/// Current instant.
pub fn now() -> std::time::Instant {
    std::time::Instant::now()
}

/// Alias type for `SmallVec` of max stack-size 8.
pub type SVec<T> = smallvec::SmallVec<[T; 8]>;
/// Alias type for `SmallVec` of max stack-size 16.
pub type SVec16<T> = smallvec::SmallVec<[T; 16]>;
/// Alias type for `SmallVec` of max stack-size 16.
pub type SVec32<T> = smallvec::SmallVec<[T; 32]>;

/// Alias macro for smallvec construction.
#[macro_export]
macro_rules! svec {
    ($($stuff:tt)*) => {
        $crate::smallvec::smallvec!($($stuff)*)
    };
}

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
    #[cfg(not(release))]
    macro_rules! client_wasm_build_dir {
        () => {
            $crate::client_wasm_build_dir_for!(debug)
        };
    }

    /// Directory in which the WASM client is being built (release version).
    #[macro_export]
    #[cfg(release)]
    macro_rules! client_wasm_build_dir {
        () => {
            $crate::client_wasm_build_dir_for!(release)
        };
    }
}
