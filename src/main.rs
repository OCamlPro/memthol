//! An example of serving static assets with Gotham.

#[macro_use]
extern crate clap;

mod assets;
mod base;
#[macro_use]
pub mod conf;
mod err;
mod router;
mod socket;

/// Default clap values.
mod default {
    /// Default aadress.
    pub const ADDR: &str = "localhost";
    /// Default port.
    pub const PORT: &str = "7878";
    /// Default directory.
    pub const DIR: &str = ".";
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

pub fn main() {
    let matches = clap_app!(memthol =>
            (author: crate_authors!("\n"))
            (version: crate_version!())
            (about: "Memthol's UI.")
            (@arg VERB:
                -v --verbose
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
            (@arg DIR:
                !required
                default_value(default::DIR)
                "path to the directory containing memthol's dump files"
            )
    )
    .get_matches();

    let addr = matches.value_of("ADDR").expect("argument with default");
    let port = {
        use std::str::FromStr;
        let port = matches.value_of("PORT").expect("argument with default");
        usize::from_str(port).expect("argument with validator")
    };

    let verb = matches.occurrences_of("VERB") > 0;
    conf::set_verb(verb);

    let dump_dir = matches.value_of("DIR").expect("argument with default");

    let path = format!("{}:{}", addr, port);
    println!("|===| Config");
    println!("| url: http://{}", path);
    println!("| dump directory: `{}`", dump_dir);
    println!("|===|");
    println!();

    let router = router::new();

    println!("initializing assets...");
    unwrap! {
        assets::init(&addr, port)
    }

    println!("starting server...");
    unwrap! {
        socket::spawn_server(addr, port + 1, dump_dir)
    }

    gotham::start(path, router)
}
