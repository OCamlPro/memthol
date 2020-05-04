//! Builds memthol's client and copies the right things in the right place.

fn main() {
    if client::is_outdated() {
        wasm_pack::check();
        client::deploy();
        client::copy_assets()
    }
}

/// Static stuff for building the client.
mod client {
    use lazy_static::lazy_static;

    use super::*;

    /// True if the version of the client in this crate's asset directory is outdated.
    pub fn is_outdated() -> bool {
        more_recently_modified(CLIENT_TARGET_JS_PATH.as_str(), CLIENT_SRC_PATH)
            || more_recently_modified(CLIENT_TARGET_JS_PATH.as_str(), CLIENT_STATIC_PATH)
    }

    /// Path to the client's crate.
    pub const CLIENT_PATH: &str = "../libs/client";
    /// Path to the client's sources.
    pub const CLIENT_SRC_PATH: &str = "../libs/client/src";
    /// Path to the client's static files.
    pub const CLIENT_STATIC_PATH: &str = "../libs/client/static";

    /// Path to the UI's (this crate's) directory.
    const UI_PATH: &str = ".";
    /// Path to the UI's (this crate's) asset files.
    const UI_ASSET_PATH: &str = "static";

    lazy_static! {
        /// Path to the client's target files. (The result of compiling the client.)
        static ref CLIENT_TARGET_PATH: String = format!("{}{}", CLIENT_PATH, "/pkg");
    }

    const CLIENT_JS_FILE: &str = "/client.js";
    const CLIENT_WASM_FILE: &str = "/client_bg.wasm";

    lazy_static! {
        /// Path to the client's asset files.
        static ref CLIENT_ASSET_PATH: String = format!("{}{}", CLIENT_PATH, "/static");

        /// Path to the client's JS target files.
        static ref CLIENT_TARGET_JS_PATH: String = format!(
            "{}{}", *CLIENT_TARGET_PATH, CLIENT_JS_FILE
        );
        /// Path to the client's web assembly target files.
        static ref CLIENT_TARGET_WASM_PATH: String = format!(
            "{}{}", *CLIENT_TARGET_PATH, CLIENT_WASM_FILE
        );
    }

    /// Outputs an error about building the client (includes the command) and exits with status `2`.
    fn error<T>() -> T {
        let cmd = wasm_pack::string_cmd();
        println!("|===| while building memthol UI's client with");
        println!("| {}", cmd);
        println!("|===|");
        std::process::exit(2)
    }

    macro_rules! unwrap {
        ($e:expr, $($blah:tt)*) => (
            $e.unwrap_or_else(|e| {
                println!("|===| Error");
                println!("| {}", e);
                println!("| {}", format!($($blah)*));
                error()
            })
        );
    }

    /// Deploys the client's code and static assets in `CLIENT_PATH/DEPLOY_PATH`.
    pub fn deploy() {
        let curr_dir = unwrap! {
            std::env::current_dir(),
            "could not retrieve current direcory"
        };
        unwrap! {
            std::env::set_current_dir(CLIENT_PATH),
            "failed to access memthol's client's build directory `{}`",
            CLIENT_PATH
        }

        // let _ = Command::new("rm").arg("!(\"Readme.md\")").output();

        // Build the client.
        let res = build();

        // Get back into the original directory.
        unwrap! {
            std::env::set_current_dir(curr_dir).map(|_| ()),
            "failed to access memthol's UI's build directory"
        }

        res
    }

    /// This function is meant to run in `CLIENT_PATH`. Do not use directly.
    fn build() {
        let mut proc = wasm_pack::cmd();

        let output = match proc.output() {
            Ok(out) => out,
            Err(e) => {
                println!("|===| Error");
                println!("| {}", e);
                error()
            }
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
    }

    pub fn copy_assets() {
        println!("current dir: {:?}", std::env::current_dir().unwrap());

        // Copy target files.
        let dir_copy_option = fs_extra::dir::CopyOptions {
            overwrite: true,
            skip_exist: true,
            buffer_size: 64000,
            copy_inside: true,
            depth: 0,
        };
        let file_copy_option = fs_extra::file::CopyOptions {
            overwrite: true,
            skip_exist: true,
            buffer_size: 64000,
        };
        unwrap! {
            fs_extra::dir::copy(&*CLIENT_ASSET_PATH, &*UI_PATH, &dir_copy_option).map(|_| ()),
            "while copying `{}` to `{}`", &*CLIENT_ASSET_PATH, UI_PATH
        }
        let ui_js_target = format!("{}{}", &*UI_ASSET_PATH, CLIENT_JS_FILE);
        unwrap! {
            fs_extra::file::copy(
                &*CLIENT_TARGET_JS_PATH,
                &ui_js_target,
                &file_copy_option,
            ).map(|_| ()),
            "while copying `{}` to `{}`", &*CLIENT_TARGET_JS_PATH, ui_js_target
        }
        let ui_wasm_target = format!("{}{}", &*UI_ASSET_PATH, CLIENT_WASM_FILE);
        unwrap! {
            fs_extra::file::copy(
                &*CLIENT_TARGET_WASM_PATH,
                &ui_wasm_target,
                &file_copy_option,
            ).map(|_| ()),
            "while copying `{}` to `{}`", &*CLIENT_TARGET_WASM_PATH, ui_wasm_target
        }
    }

    /// False if `reference` was modified more recently than **all** files in `dir`.
    ///
    /// **NB:** returns `true` if the date of last modification for `reference` or any file in `dir`
    /// could not be retrieved.
    fn more_recently_modified<P1, P2>(dir: P1, reference: P2) -> bool
    where
        P1: AsRef<std::path::Path>,
        P2: AsRef<std::path::Path>,
    {
        let reference_last_mod =
            if let Ok(last_mod) = std::fs::metadata(reference).and_then(|meta| meta.modified()) {
                last_mod
            } else {
                panic!("1")
                // return true;
            };

        for entry in walkdir::WalkDir::new(dir)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            if let Some(last_mod) = entry.metadata().ok().and_then(|meta| meta.modified().ok()) {
                if last_mod > reference_last_mod {
                    return true;
                }
            } else {
                panic!("2")
                // return true;
            }
        }

        false
    }
}

/// Contains helpers related to `wasm-pack`.
///
/// - `cmd` and `string_cmd` generate the command to call `wasm-pack` with;
/// - `check` will produce an error if `wasm-pack` is not installed.
mod wasm_pack {
    use std::process::Command;

    const CMD: &str = "wasm-pack";

    const OPTIONS: [&str; 5] = ["build", "--target", "web", "--out-name", "client"];
    #[cfg(release)]
    const RLS_OPTIONS: [&str; 1] = ["--release"];

    fn inner_cmd() -> Command {
        let mut cmd = Command::new(CMD);
        cmd.args(&OPTIONS);
        cmd
    }
    #[cfg(release)]
    pub fn cmd() -> Command {
        let mut cmd = inner_cmd();
        cmd.args(&RLS_OPTIONS);
        cmd
    }
    #[cfg(not(release))]
    pub fn cmd() -> Command {
        inner_cmd()
    }

    fn inner_string_cmd() -> String {
        let mut res = CMD.to_string();
        for opt in &OPTIONS {
            res.push(' ');
            res.push_str(opt);
        }
        res
    }
    #[cfg(release)]
    pub fn string_cmd() -> String {
        let mut res = inner_string_cmd();
        for opt in &RLS_OPTIONS {
            res.push(' ');
            res.push_str(opt)
        }
        res
    }
    #[cfg(not(release))]
    pub fn string_cmd() -> String {
        inner_string_cmd()
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
