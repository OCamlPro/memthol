//! Errors, handled by `error_chain`.

error_chain::error_chain! {
    types {
        Error, ErrorKind, ResExt, Res;
    }

    foreign_links {
        Peg(peg::error::ParseError<peg::str::LineCol>)
        /// Parse error from `peg`.
        ;
        ParseInt(std::num::ParseIntError)
        /// Integer parse error from `std`.
        ;
        Io(std::io::Error)
        /// I/O error.
        ;
        Serde(bincode::Error)
        /// (De)serialization error.
        ;
    }

    links {}
    errors {}
}

impl Error {
    /// Multi-line representation of a trace of errors.
    ///
    /// See the [module-level documentation] for more.
    ///
    /// [module-level documentation]: index.html (module-level documentation)
    pub fn to_pretty(&self) -> String {
        let mut s = "error: ".to_string();

        // Reverse errors.
        let mut errs = vec![];
        for e in self.iter() {
            errs.push(e)
        }

        let mut is_first = true;
        for e in errs.into_iter().rev() {
            if is_first {
                is_first = false
            } else {
                s.push_str("\n")
            }
            s.push_str(&e.to_string())
        }

        s
    }
}

pub use error_chain::bail;
