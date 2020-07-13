//! Builds memthol's client and copies the right things in the right place.

fn main() {
    let in_release = {
        let profile = std::env::var("PROFILE")
            .expect("failed to retrieve cargo profile info from environment");
        match &profile as &str {
            "release" => true,
            "debug" => false,
            _ => panic!("unexpected cargo profile {:?}", profile),
        }
    };

    paths::show();

    if !in_release {
        client::build();
    } else {
        let target = std::path::Path::new(&*paths::WASM_TARGET);
        if !target.exists() {
            eprintln!(
                "\
                    Could not find WASM client `{}`.\n\
                    It seems you are trying to build memthol-ui in release mode \
                    using cargo directly.\n\
                    This won't work, use `rsc/scripts/release.sh` instead.
                ",
                &*paths::WASM_TARGET,
            );
            panic!("compilation failed")
        }
    }

    emit_env_var();
    emit_rerun();
}

/// Outputs an error about building the client (includes the command) and exits with status `2`.
fn error<T>() -> T {
    println!("|===| while building memthol UI's client with");
    println!("| {:?}", wasm_pack::cmd());
    println!("|===|");
    panic!("a fatal error occured")
}

macro_rules! unwrap {
    ($e:expr, $($blah:tt)*) => (
        $e.unwrap_or_else(|e| {
            println!("|===| Error");
            println!("| {}", e);
            println!("| {}", format!($($blah)*));
            crate::error()
        })
    );
}

/// Sets the environment variable indicating the path to the client build dir.
pub fn emit_env_var() {
    println!(
        "cargo:rustc-env={}={}",
        base::build_dir_env_var!(),
        *paths::BUILD,
    )
}

/// Emits the re-run instructions for cargo.
pub fn emit_rerun() {
    for entry in walkdir::WalkDir::new(*paths::CLIENT_CRATE)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        println!("cargo:rerun-if-changed={}", entry.path().display())
    }
}

mod paths {
    lazy_static::lazy_static! {
        /// Output directory.
        pub static ref BUILD: String = {
            let parent = unwrap! {
                std::fs::canonicalize(".."),
                "during the extraction of the canonical version of path `..`",
            };
            format!("{}/{}", parent.display(), base::client::WASM_TARGET_DIR)
        };

        /// WASM target file.
        pub static ref WASM_TARGET: String = format!("{}/client_bg.wasm", &*BUILD);

        /// Path to the client's crate.
        pub static ref CLIENT_CRATE: &'static str = "../libs/client";
    }

    pub fn show() {
        macro_rules! dups {
            ($($id:ident),* $(,)?) => (
                vec![$((stringify!($id), $id.to_string())),*]
            );
        }
        println!("paths:");
        for (id, val) in dups![BUILD, CLIENT_CRATE,] {
            println!("| {:<30} | {}", id, val)
        }
        println!()
    }
}

/// Static stuff for building the client.
mod client {
    use super::*;

    pub fn build() {
        let start = std::time::Instant::now();
        wasm_pack::check();

        let mut build_cmd = wasm_pack::cmd();

        println!("cmd: {:?}", build_cmd);

        let output = unwrap! {
            build_cmd.output(),
            "while running {:?}", build_cmd
        };

        if !output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);
            if !stdout.is_empty() {
                println!("|===| stdout");
                for line in stdout.lines() {
                    println!("| {}", line)
                }
            }
            if !stderr.is_empty() {
                println!("|===| stderr");
                for line in stderr.lines() {
                    println!("| {}", line)
                }
            }
            error()
        }
        let duration = std::time::Instant::now() - start;
        println!(
            "done building in {}.{}",
            duration.as_secs(),
            duration.subsec_millis()
        );
    }
}

/// Contains helpers related to `wasm-pack`.
///
/// - `cmd` and `string_cmd` generate the command to call `wasm-pack` with;
/// - `check` will produce an error if `wasm-pack` is not installed.
mod wasm_pack {
    use std::process::Command;

    use super::paths;

    const CMD: &str = "wasm-pack";
    const MODE: &str = "build";

    lazy_static::lazy_static! {
        static ref OPTIONS: Vec<&'static str> = vec![
            // Options for `wasm-pack`.
            "--target", "web",
            "--out-name", "client",
            "--out-dir", &*paths::BUILD,
            &*paths::CLIENT_CRATE,
        ];
    }

    pub fn cmd() -> Command {
        let mut cmd = Command::new(CMD);
        cmd.arg(MODE);
        cmd.args(&*OPTIONS);
        cmd
    }

    pub fn check() {
        let fail = |msg, err| {
            println!("Error: {}.", msg);
            if let Some(e) = err {
                println!("{}", e);
                println!()
            }
            println!("`wasm-pack` is mandatory for the client side of memthol's UI,");
            println!("please install it from https://rustwasm.github.io/wasm-pack/installer");
            println!();
            panic!("wasm-pack is not installed")
        };
        match Command::new(CMD).arg("help").output() {
            Ok(output) => {
                if output.status.success() {
                    ()
                } else {
                    fail("`wasm-pack` is not installed", None)
                }
            }
            Err(e) => fail("could not check for `wasm-pack`", Some(e)),
        }
    }
}
