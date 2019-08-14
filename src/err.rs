//! Memthol's server errors.

error_chain::error_chain! {
    types {
        Err, ErrorKind, ResultExt, Res;
    }

    foreign_links {
        Io(::std::io::Error)
        /// I/O error.
        ;
    }

    errors {
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
