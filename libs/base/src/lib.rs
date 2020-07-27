//! Re-exports, types and helpers for all crates in this project.

pub extern crate rand;

pub use derive_more::*;
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
    /// Environment variable indicating the path to the (wasm) client's build dir.
    #[macro_export]
    macro_rules! build_dir_env_var {
        () => {
            "WASM_CLIENT_DIR"
        };
    }
    /// Retrieves the path to the client's build dir.
    ///
    /// Compile-time error if the environment variable for `BUILD_DIR_ENV_VAR` is not set.
    #[macro_export]
    macro_rules! client_get_build_dir {
        () => {
            env!($crate::build_dir_env_var!())
        };
    }

    pub const WASM_TARGET_DIR: &str = "target/client.wasm";
}
