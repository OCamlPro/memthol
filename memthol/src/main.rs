//! Memthol's UI.

#[macro_use]
extern crate clap;

use base::log;

/// Default clap values.
mod default {
    /// Default filter gen parameter.
    pub const FILTER_GEN: &str = "alloc_site";

    /// Default address.
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
        Err(format!("expected integer (usize), found `{}`", s))
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
    let mut error_handler = memthol::ErrorHandler::new();

    let matches = clap_app!(memthol =>
        (author: crate_authors!())
        (version: crate_version!())
        (about: "Memthol's UI.")

        // Basic stuff.

        (@arg VERB:
            -v --verbose !required
            ...
            "activates verbose output"
        )
        (@arg OPEN:
            --open !required
            "opens the memthol browser right away"
        )
        (@arg LOG:
            -l --log !required
            "activates (separate) socket logging"
        )

        // Filter-gen stuff.
        (@arg FILTER_GEN:
            --filter_gen +takes_value !required
            default_value(default::FILTER_GEN)
            "filter generation heuristic, get help with `--filter_gen help`"
        )

        // Server-related stuff.

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

        // Directory or CTF file.

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
    let open = matches.occurrences_of("OPEN") > 0;

    let verb = matches.occurrences_of("VERB");
    init_logger(verb);

    let target = matches.value_of("INPUT").expect("argument with default");

    let filter_gen_args = matches
        .value_of("FILTER_GEN")
        .expect("argument with default");
    memthol::clap::filter_gen(filter_gen_args);

    let path = format!("{}:{}", addr, port);
    println!("|===| Starting");
    println!("| url: http://{}", path);
    println!("| target: `{}`", target);
    println!("|===|");
    println!();

    error_handler.handle_new_errors();

    let router = memthol::router::new();

    log::info!("starting data monitoring");
    base::unwrap_or! {
        charts::data::start(target), exit
    }

    error_handler.handle_new_errors();

    log::info!("starting socket listeners");
    base::unwrap_or! {
        memthol::socket::spawn_server(addr, port + 1, log), exit
    }

    error_handler.handle_new_errors();

    if open {
        open_in_background(&path)
    }

    log::info!("starting gotham server");
    std::thread::spawn(move || gotham::start(path, router));

    error_handler.error_watch_loop()
}

fn open_in_background(path: &str) {
    let path = format!("http://{}", path);
    std::thread::spawn(move || match open::that(&path) {
        Ok(status) => {
            if !status.success() {
                log::error!("while opening page {}", path);
                log::error!(
                    "got a non-success exit code: {}",
                    status
                        .code()
                        .map(|n| n.to_string())
                        .unwrap_or_else(|| "??".into())
                )
            }
        }
        Err(e) => {
            log::error!("while opening page {}", path);
            log::error!("{}", e)
        }
    });
}
