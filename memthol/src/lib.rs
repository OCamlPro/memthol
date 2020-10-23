//! Memthol's UI.
//!
//! Memthol's UI is decomposed in two parts: the *client* and the server. The client is the part
//! users interact with in their browser. It is compiled to webassembly.
//!
//! The server (this crate) is compiled normally and is responsible for
//!
//! - monitoring the files in the user-provided dump directory;
//! - organising the data from the diffs;
//! - answer browser's queries by sending them the client;
//! - maintain one session per client that performs whatever treatment the user requests.
//!
//! The documentation for the server is the present document. The client's crate is in the
//! `./libs/client` from the root of the repository.
//!
//! # Common Crates
//!
//! The client and the server have quite a lot of code in common: types for diffs, allocations,
//! charts, filters... Everything related to "raw data" is in the [`alloc_data` crate]: diff and
//! allocation types, but also parsing and file monitoring. This crate is in the `./libs/data`
//! directory from the root of the repo.
//!
//! The [`charts` crate] deals with chart representation, and how charts handle the raw data. It
//! also defines the messages that the server and the client can exchange.
//!
//! [`alloc_data` crate]: ../alloc_data/index.html (Memthol's alloc_data crate)
//! [`charts` crate]: ../charts/index.html (Memthol's charts crate)

#[macro_use]
pub mod prelude;

pub mod assets;
pub mod msg;
pub mod router;
pub mod socket;

use prelude::*;

/// Top-level error handler.
pub struct ErrorHandler {
    /// Error context.
    cxt: err::ErrorCxt,
}
impl ErrorHandler {
    /// Constructor.
    pub fn new() -> Self {
        Self {
            cxt: err::ErrorCxt::new(),
        }
    }

    /// Handles new errors.
    ///
    /// This function `std::process::exit(2)`s on fatal errors.
    pub fn handle_new_errors(&mut self) {
        let mut line_count = 0;
        let (err_count, fatal) = self.cxt.new_errors_do(|err, fatal| {
            for (idx, line) in err.lines().enumerate() {
                line_count += 1;
                if idx == 0 {
                    if fatal {
                        log::error!("|===[fatal] {}", line)
                    } else {
                        log::warn!("|===| {}", line)
                    }
                } else {
                    if fatal {
                        log::error!("| {}", line)
                    } else {
                        log::warn!("| {}", line)
                    }
                }
            }
        });
        if err_count > 0 && line_count > 1 {
            if fatal {
                log::error!("|===|")
            } else {
                log::warn!("|===|")
            }
        }
        if fatal {
            println!();
            log::error!("exiting due to fatal error(s)");
            std::process::exit(2)
        }
    }

    /// Loops, watching for errors.
    ///
    /// This function `std::process::exit(2)`s on fatal errors.
    pub fn error_watch_loop(&mut self) {
        loop {
            self.handle_new_errors();
            std::thread::sleep(time::Duration::from_millis(200))
        }
    }
}

/// CLAP-related actions.
pub mod clap {
    use crate::prelude::*;

    /// Handles filter-generation-related CLAs.
    ///
    /// When `args.trim() == "help"`, this function displays an help message for filter generation
    /// and `std::process::exit(0)`s.
    pub fn filter_gen(args: &str) {
        let mut exit_code = None;
        let args = args.trim();

        if args == "help" {
            exit_code = Some(0)
        }

        if let Err(e) =
            charts::filter::gen::set_from_cla(args).chain_err(charts::filter::gen::FilterGen::help)
        {
            err::register_fatal(e)
        }

        if let Some(code) = exit_code {
            println!("{}", charts::filter::gen::FilterGen::help().trim());
            std::process::exit(code)
        }
    }
}
