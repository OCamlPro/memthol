//! Builds memthol's client and copies the right things in the right place.

/// Checks that `cargo-web` is installed.
mod cargo_web {
    use std::process::Command;

    pub fn check() {
        let fail = |msg, err| {
            println!("Error: {}.", msg);
            if let Some(e) = err {
                println!("{}", e);
                println!()
            }
            println!("`cargo-web` is mandatory for the client side of memthol's UI,");
            println!("please install it with");
            println!();
            println!("```");
            println!("cargo install cargo-web");
            println!("```");
            panic!("cargo-web is not installed")
        };
        match Command::new("cargo").arg("web").arg("help").status() {
            Ok(status) => {
                if status.success() {
                    ()
                } else {
                    fail("`cargo-web` is not installed", None)
                }
            }
            Err(e) => fail("could not check for `cargo-web`", Some(e)),
        }
    }
}

/// Static stuff for building the client.
mod client {
    use lazy_static::lazy_static;
    use std::process::Command;

    /// Path to the client's crate.
    const CLIENT_PATH: &str = "libs/client";

    /// Path to the UI's (this crate's) directory.
    const UI_PATH: &str = ".";
    /// Path to the UI's (this crate's) asset files.
    const UI_ASSET_PATH: &str = "static";

    #[cfg(debug_assertions)]
    lazy_static! {
        /// Path to the client's target files. (The result of compiling the client.)
        static ref CLIENT_TARGET_PATH: String = format!(
            "{}{}", CLIENT_PATH,
            "/target/wasm32-unknown-unknown/debug"
        );
    }
    #[cfg(not(debug_assertions))]
    lazy_static! {
        /// Path to the client's target files. (The result of compiling the client.)
        static ref CLIENT_TARGET_PATH: String = format!(
            "{}{}", CLIENT_PATH,
            "/target/wasm32-unknown-unknown/release"
        );
    }

    const CLIENT_JS_FILE: &str = "/client.js";
    const CLIENT_WASM_FILE: &str = "/client.wasm";

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

    /// Command to run to build the client.
    pub static CMD: &str = "cargo";

    /// Options for the command to build the client.
    #[cfg(debug_assertions)]
    pub static OPTIONS: [&str; 2] = ["web", "build"];
    /// Options for the command to build the client.
    #[cfg(not(debug_assertions))]
    pub static OPTIONS: [&str; 3] = ["web", "build", "--release"];

    /// Outputs an error about building the client (includes the command) and exits with status `2`.
    fn error<T>() -> T {
        let mut cmd = CMD.to_string();
        for option in &OPTIONS {
            cmd.push_str(" ");
            cmd.push_str(option)
        }
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

        let _ = Command::new("rm").arg("!(\"Readme.md\")").output();

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
        let mut proc = Command::new(CMD);
        for option in &OPTIONS {
            proc.arg(option);
        }

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
}

fn main() {
    // println!("cargo:rerun-if-changed=\"static\"");
    // cargo_web::check();
    // client::deploy();
    // client::copy_assets()
}
