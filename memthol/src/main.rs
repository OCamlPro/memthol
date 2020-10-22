//! Memthol's UI.

#[macro_use]
extern crate clap;

use base::log;

/// Default clap values.
mod default {
    /// Default aadress.
    pub const ADDR: &str = "localhost";
    /// Default port.
    pub const PORT: &str = "7878";
    /// Default directory.
    pub const INPUT: &str = ".";
}

/// Fails if the input string is not a `usize`.
fn usize_validator(s: String) -> Result<(), String> {
    use std::str::FromStr;
    if usize::from_str(&s).is_err() {
        Err(format!("expected integer, found `{}`", s))
    } else {
        Ok(())
    }
}

/// Initializes the logger.
fn init_logger(verb: u64) {
    let mut builder = pretty_env_logger::formatted_timed_builder();

    let level = match verb {
        0 => log::LevelFilter::Warn,
        1 => log::LevelFilter::Info,
        2 => log::LevelFilter::Debug,
        _ => log::LevelFilter::Trace,
    };

    builder.filter_module("memthol", level);
    builder.filter_module("ctf", level);
    builder.filter_module("charts", level);
    builder.init();
}

pub fn main() {
    let matches = clap_app!(memthol =>
        (author: crate_authors!())
        (version: crate_version!())
        (about: "Memthol's UI.")
        (@arg VERB:
            -v --verbose
            ...
            "activates verbose output"
        )
        (@arg ADDR:
            -a --addr +takes_value !required
            default_value(default::ADDR)
            "the address to serve the UI at"
        )
        (@arg PORT:
            -p --port +takes_value !required
            default_value(default::PORT)
            { usize_validator }
            "the port to serve the UI at"
        )
        (@arg LOG:
            -l --log !required
            "activates (separate) socket logging"
        )
        (@arg INPUT:
            !required
            default_value(default::INPUT)
            "path to either a directory containing memthol's dump files, or a memtrace CTF file"
        )
    )
    .get_matches();

    let addr = matches.value_of("ADDR").expect("argument with default");
    let port = {
        use std::str::FromStr;
        let port = matches.value_of("PORT").expect("argument with default");
        usize::from_str(port).expect("argument with validator")
    };
    let log = matches.occurrences_of("LOG") > 0;

    let verb = matches.occurrences_of("VERB");
    init_logger(verb);

    let target = matches.value_of("INPUT").expect("argument with default");

    let path = format!("{}:{}", addr, port);
    println!("|===| Starting");
    println!("| url: http://{}", path);
    println!("| target: `{}`", target);
    println!("|===|");
    println!();

    let router = memthol::router::new();

    log::info!("starting data monitoring");
    base::unwrap_or! {
        charts::data::start(target), exit
    }

    log::info!("starting socket listeners");
    base::unwrap_or! {
        memthol::socket::spawn_server(addr, port + 1, log), exit
    }

    log::info!("starting gotham server");
    gotham::start(path, router)
}
