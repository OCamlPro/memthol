//! Memthol's server errors.

prelude! {}

error_chain::error_chain! {
    types {
        Err, ErrorKind, ResExt, Res;
    }

    foreign_links {
        Io(std::io::Error)
        /// I/O error.
        ;
        DeJson(serde_json::error::Error)
        /// Json (de)serialization error.
        ;

    }

    links {
        Data(alloc_data::err::Err, alloc_data::err::ErrKind)
        /// Error from the `alloc_data` crate.
        ;
    }

    errors {
    }
}

impl Err {
    /// Multi-line representation of a trace of errors.
    ///
    /// See the [module-level documentation] for more.
    ///
    /// [module-level documentation]: index.html (module-level documentation)
    pub fn pretty(&self) -> String {
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

#[macro_export]
macro_rules! unwrap {
    ($e:expr) => {
        match $e {
            Ok(res) => res,
            Err(e) => {
                println!("|===| Error:");
                for e in e.iter() {
                    for line in format!("{}", e).lines() {
                        println!("| {}", line)
                    }
                }
                println!("|===|");
                std::process::exit(2)
            }
        }
    };
}
