use std::env;
use std::process::{self, Command};

struct BuildConfig {
    profile: &'static str,
    client_wasm_path: &'static str,
}

#[cfg(debug_assertions)]
fn config() -> BuildConfig {
    BuildConfig {
        profile: "--dev",
        client_wasm_path: "target/client.wasm/debug",
    }
}

#[cfg(not(debug_assertions))]
fn config() -> BuildConfig {
    BuildConfig {
        profile: "--release",
        client_wasm_path: "target/client.wasm/release",
    }
}

fn main() {
    let config = config();
    let memthol_dir = env::current_dir().unwrap();
    let status = Command::new("wasm-pack")
        .args(&[
            "build",
            config.profile,
            "--target",
            "web",
            "--out-name",
            "client",
            "--out-dir",
            config.client_wasm_path,
            "../libs/client",
        ]).current_dir(memthol_dir)
        .status()
        .unwrap();
    if !status.success() {
        process::exit(1); 
    }
    println!("cargo:rerun-if-changed=src/*");
}
 