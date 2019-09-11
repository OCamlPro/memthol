//! Error-handling types.

pub use error_chain::bail;

use crate::base::Msg;

error_chain::error_chain! {
    types {
        Err, ErrorKind, ResExt, Res;
    }

    links {
        Data(alloc_data::parser::err::ParseErr, alloc_data::parser::err::ParseErrKind)
        /// Error from the `alloc_data` crate.
        ;
    }

    errors {}
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

/// Turns a result into a message.
///
/// If it's an error, it becomes an error message.
pub fn msg_of_res(res: Res<Msg>) -> Msg {
    match res {
        Ok(msg) => msg,
        Result::Err(e) => Msg::err(e),
    }
}
