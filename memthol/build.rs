//! Builds memthol's client and copies the right things in the right place.

fn main() {
    let changed = client::copy_client_and_libs();
    if changed {
        client::build();
    }
    emit_env_var();
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
            crate::error()
        })
    );
}

/// Sets the environment variable indicating the path to the client build dir.
pub fn emit_env_var() {
    println!(
        "cargo:rustc-env={}={}",
        base::build_dir_env_var!(),
        *paths::CLIENT_BUILD,
    )
}

mod paths {
    lazy_static::lazy_static! {
        /// Output directory.
        pub static ref BUILD: String = unwrap! {
            std::env::var("OUT_DIR"),
            "while retrieving cargo build directory"
        };

        /// Path to the client's crate.
        pub static ref CLIENT_CRATE: &'static str = "../libs/client";
        /// Client build directory.
        pub static ref CLIENT_TARGET: String = format!("{}/client", *BUILD);
        /// Client target directory.
        pub static ref CLIENT_BUILD: String = format!("{}/target", *BUILD);

        /// Path to the `base` crate.
        pub static ref BASE_CRATE: &'static str = "../libs/base";
        pub static ref BASE_TARGET: String = format!("{}/base", *BUILD);
        /// Path to the `alloc_data` crate.
        pub static ref ALLOC_DATA_CRATE: &'static str = "../libs/alloc_data";
        pub static ref ALLOC_DATA_TARGET: String = format!("{}/alloc_data", *BUILD);
        /// Path to the `charts` crate.
        pub static ref CHARTS_CRATE: &'static str = "../libs/charts";
        pub static ref CHARTS_TARGET: String = format!("{}/charts", *BUILD);
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

    pub fn copy_client_and_libs() -> bool {
        let to_copy = [
            (&*paths::CLIENT_CRATE, &*paths::CLIENT_TARGET),
            (&*paths::BASE_CRATE, &*paths::BASE_TARGET),
            (&*paths::ALLOC_DATA_CRATE, &*paths::ALLOC_DATA_TARGET),
            (&*paths::CHARTS_CRATE, &*paths::CHARTS_TARGET),
        ];
        let mut changed = false;
        for (src, tgt) in &to_copy {
            let new_changed = copy_if_modified(src, tgt);
            changed = changed || new_changed;
        }
        changed
    }

    fn copy_if_modified<P1, P2>(src: P1, tgt: P2) -> bool
    where
        P1: AsRef<std::path::Path>,
        P2: AsRef<std::path::Path>,
    {
        let (src, tgt) = (src.as_ref(), tgt.as_ref());
        if more_recently_modified(src, tgt) {
            println!("copying {} to {}", src.display(), tgt.display());
            unwrap! {
                fs_extra::dir::copy(src, tgt, &fs_extra::dir::CopyOptions {
                    overwrite: true,
                    skip_exist: false,
                    buffer_size: 64_000,
                    copy_inside: true,
                    depth: 0,
                }),
                "while copying the client's sources from {} to {}",
                &*paths::CLIENT_CRATE,
                &*paths::CLIENT_BUILD,
            };
            true
        } else {
            false
        }
    }

    /// False if `reference` was modified more recently than **all** files/dirs in `dir`.
    ///
    /// **NB:** returns `true` if `reference` does not exist.
    ///
    /// # Panics
    ///
    /// - on date-of-last-mod retrieval error on any file/dir
    /// - on sys-time/duration conversion error
    fn more_recently_modified<P1, P2>(dir: P1, reference: P2) -> bool
    where
        P1: AsRef<std::path::Path>,
        P2: AsRef<std::path::Path>,
    {
        println!(
            "more_recently_modified({}, {})",
            dir.as_ref().display(),
            reference.as_ref().display()
        );

        let reference = reference.as_ref();
        if !reference.exists() {
            return true;
        }

        let reference_last_mod = last_mod(reference);

        println!(
            "|===| {:>10} ({})",
            display_time(reference_last_mod),
            reference.display()
        );

        for entry in walkdir::WalkDir::new(dir)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let entry = entry.path();
            let last_mod = last_mod(entry);
            println!(
                "    | {:>10} ({}, {})",
                display_time(last_mod),
                entry.display(),
                last_mod > reference_last_mod
            );
            if last_mod > reference_last_mod {
                println!(
                    "{:?} was modified more recently than {}",
                    entry.display(),
                    reference.display(),
                );
                return true;
            }
        }

        false
    }

    /// Retrieves the date of last modification as a duration since EPOCH.
    fn last_mod<P>(file: P) -> std::time::Duration
    where
        P: AsRef<std::path::Path>,
    {
        let file = file.as_ref();
        let epoch = std::time::UNIX_EPOCH;
        let sys_time = unwrap! {
            std::fs::metadata(file).and_then(|meta| meta.modified()),
            "while retrieving the date of last modification for `{}`", file.display()
        };
        unwrap! {
            sys_time.duration_since(epoch),
            "during conversion from system time to duration since epoch"
        }
    }

    fn display_time(d: std::time::Duration) -> String {
        let mut s = format!("{}", d.as_secs());
        if s.len() > 0 {
            let mut curr = s.len() - 1;
            let mut dec = 2;
            while curr > 3 {
                curr -= dec;
                s.insert(curr, '.');
                dec = 3;
            }
        }
        s
    }
}

/// Contains helpers related to `wasm-pack`.
///
/// - `cmd` and `string_cmd` generate the command to call `wasm-pack` with;
/// - `check` will produce an error if `wasm-pack` is not installed.
mod wasm_pack {
    use std::process::Command;

    use super::paths;

    lazy_static::lazy_static! {
        static ref CMD: &'static str = "wasm-pack";

        static ref OPTIONS: Vec<&'static str> = vec![
            // Options for `wasm-pack`.
            "build",
            "--target", "web",
            "--out-name", "client",
            "--out-dir", &*paths::CLIENT_BUILD,
            &*paths::CLIENT_TARGET,
        ];

        // #[cfg(release)]
        // static ref RLS_OPTIONS: Vec<String> = vec!["--release"];
    }

    fn inner_cmd() -> Command {
        let mut cmd = Command::new(*CMD);
        cmd.args(&*OPTIONS);
        cmd
    }
    #[cfg(release)]
    pub fn cmd() -> Command {
        let mut cmd = inner_cmd();
        cmd.args(&*RLS_OPTIONS);
        cmd
    }
    #[cfg(not(release))]
    pub fn cmd() -> Command {
        inner_cmd()
    }

    fn inner_string_cmd() -> String {
        let mut res = CMD.to_string();
        for opt in &*OPTIONS {
            res.push(' ');
            res.push_str(opt);
        }
        res
    }
    #[cfg(release)]
    pub fn string_cmd() -> String {
        let mut res = inner_string_cmd();
        for opt in *RLS_OPTIONS {
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
        match Command::new(*CMD).arg("help").output() {
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
