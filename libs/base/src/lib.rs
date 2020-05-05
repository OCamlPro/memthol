//! Re-exports, types and helpers for all crates in this project.

pub extern crate rand;

pub use derive_more::*;

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
}
