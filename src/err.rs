//! Memthol's server errors.

pub use charts::err::*;

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
