//! Re-exports, types and helpers for all crates in this project.

#![deny(missing_docs)]

pub extern crate bincode;
pub extern crate chrono;
pub extern crate conv;
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

use prelude::serde::*;

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

/// Represents a sample rate.
///
/// Contains the original sample rate (`f64`), as well as the integer factor corresponding to
/// dividing by the sample rate.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SampleRate {
    /// Actual sample rate.
    pub sample_rate: f64,
    /// Factor version of the sample rate.
    pub factor: usize,
    /// True if `factor` is an approximation of `1 / sample_rate`.
    pub factor_is_approx: bool,
}
impl SampleRate {
    /// Constructor.
    pub fn new(sample_rate: f64) -> Self {
        use conv::*;
        let inv = 1. / sample_rate;
        let factor = inv.trunc();
        let factor_is_approx = factor == inv;
        let factor = factor.approx().expect("error while handling sample rate");
        Self {
            sample_rate,
            factor,
            factor_is_approx,
        }
    }
}
implement! {
    impl SampleRate {
        From {
            from f64 => |sample_rate| Self::new(sample_rate)
        }
    }
}

cfg_item! {
    cfg(client) {
        /// Stores a current and reference version of something.
        #[derive(Debug, Clone)]
        pub struct Memory<T> {
            /// Current version.
            current: T,
            /// Reference version.
            reference: T,
        }
    } {
        impl<T> Memory<T> {
            /// Constructor.
            pub fn new(reference: T) -> Self
            where
                T: Clone,
            {
                Self {
                    current: reference.clone(),
                    reference,
                }
            }

            /// Current version.
            pub fn get(&self) -> &T {
                &self.current
            }
            /// Sets the current version.
            pub fn set(&mut self, current: T) {
                self.current = current
            }
            /// Current version (mutable).
            pub fn get_mut(&mut self) -> &mut T {
                &mut self.current
            }

            /// Reference version.
            pub fn reference(&self) -> &T {
                &self.reference
            }

            /// True if the current and reference versions are the same.
            pub fn has_changed(&self) -> bool
            where
                T: PartialEq,
            {
                self.current != self.reference
            }

            /// Overwrites the reference and current version.
            pub fn set_both(&mut self, reference: T)
            where
                T: Clone,
            {
                self.current = reference.clone();
                self.reference = reference;
            }

            /// Overwrites the current version to be the reference version.
            pub fn reset(&mut self)
            where
                T: Clone,
            {
                self.current = self.reference.clone()
            }
        }
    }
}

/// A range, inclusive on both ends.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize, Serialize)]
pub struct Range<T> {
    /// Lower bound.
    pub lbound: T,
    /// Upper bound.
    pub ubound: T,
}
impl<T> Range<T> {
    /// Constructor.
    pub const fn new(lbound: T, ubound: T) -> Self {
        Self { lbound, ubound }
    }

    /// Applies an action to the range's bounds.
    pub fn map<U>(self, mut action: impl FnMut(T) -> U) -> Range<U> {
        Range::new(action(self.lbound), action(self.ubound))
    }

    /// Reference version of the range bounds.
    pub fn as_ref(&self) -> Range<&T> {
        Range::new(&self.lbound, &self.ubound)
    }
}
impl<T> Range<T>
where
    T: PartialOrd,
{
    /// True if the range contains some value.
    pub fn contains(&self, val: T) -> bool {
        self.lbound <= val && val <= self.ubound
    }
    /// True if the range contains some value.
    pub fn contains_ref(&self, val: &impl AsRef<T>) -> bool {
        let val = val.as_ref();
        &self.lbound <= val && val <= &self.ubound
    }
}

impl<T> Range<Option<T>> {
    /// Unwraps optional bounds.
    pub fn unwrap(self) -> Range<T> {
        Range::new(
            self.lbound.expect("while unwrapping range lower bound"),
            self.ubound.expect("while unwrapping range upper bound"),
        )
    }

    /// Unwraps optional bounds with a default.
    pub fn unwrap_or(self, lbound: T, ubound: T) -> Range<T> {
        Range::new(self.lbound.unwrap_or(lbound), self.ubound.unwrap_or(ubound))
    }

    /// Unwraps optional bounds with a lazy default.
    pub fn unwrap_or_else(
        self,
        lbound: impl FnOnce() -> T,
        ubound: impl FnOnce() -> T,
    ) -> Range<T> {
        Range::new(
            self.lbound.unwrap_or_else(lbound),
            self.ubound.unwrap_or_else(ubound),
        )
    }
}

implement! {
    impl Range<T>, with (T: PartialOrd) {
        From {
            from (T, T) => |(lbound, ubound)| Self::new(lbound, ubound)
        }
    }

    impl Range<T>, with (T: std::fmt::Display) {
        Display {
            |&self, fmt| write!(fmt, "[{}, {}]", self.lbound, self.ubound),
        }
    }
}
