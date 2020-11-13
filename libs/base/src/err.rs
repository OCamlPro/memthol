/*<LICENSE>
    This file is part of Memthol.

    Copyright (C) 2020 OCamlPro.

    Memthol is free software: you can redistribute it and/or modify
    it under the terms of the GNU General Public License as published by
    the Free Software Foundation, either version 3 of the License, or
    (at your option) any later version.

    Memthol is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
    GNU General Public License for more details.

    You should have received a copy of the GNU General Public License
    along with Memthol.  If not, see <https://www.gnu.org/licenses/>.
*/

//! Errors for memthol, handled by `error_chain`.
//!
//! This module also features a global list of errors.

pub use error_chain::bail;

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
        let mut s = String::with_capacity(400);

        // Reverse errors.
        let mut errs = crate::SVec16::new();
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

        s.shrink_to_fit();

        s
    }
}

/// Error context, a shallow interface over a global list of errors.
///
/// The point of this type is to provide a memory of the errors from the global list of errors seen
/// so far. There's no point in using this just to register errors, use [`err::register`][reg]
/// instead.
///
/// This type is used by the server and the different client-sockets because they need to report
/// errors.
///
/// # Examples
///
/// ```rust
/// # use base::prelude::{*, err::ErrorCxt};
/// let errors = vec![
///     ("error 0", false),
///     ("error 1", false),
///     ("error 2", true),
///     ("error 3", false),
///     ("error 4", true),
/// ];
/// let mut cnt = 0;
/// macro_rules! check {
///     () => {
///         |err: &str, is_fatal| {
/// #           println!(
/// #               "err: `({}, {})`, expected `({}, {})`",
/// #               err, is_fatal, errors[cnt].0, errors[cnt].1,
/// #           );
///             assert_eq!(err, errors[cnt].0);
///             assert_eq!(is_fatal, errors[cnt].1);
///             cnt += 1
///         }
///     };
/// }
///
/// let mut cxt = ErrorCxt::new();
///
/// cxt.register(errors[0].0, errors[0].1);
/// cxt.register(errors[1].0, errors[1].1);
///
/// // Applies `check` to the two errors registered so far.
/// let (err_count, fatal) = cxt.new_errors_do(check!());
/// assert!(!fatal);
/// assert_eq!(err_count, 2);
/// assert_eq!(cnt, 2);
/// # println!();
///
/// // Applies `check` to nothing, there are no new errors.
/// let (err_count, fatal) = cxt.new_errors_do(check!());
/// assert!(!fatal);
/// assert_eq!(err_count, 0);
/// assert_eq!(cnt, 2);
/// # println!();
///
/// cxt.register(errors[2].0, errors[2].1);
/// cxt.register(errors[3].0, errors[3].1);
///
/// // Applies `check` to the two new errors.
/// let (err_count, fatal) = cxt.new_errors_do(check!());
/// assert!(fatal);
/// assert_eq!(err_count, 2);
/// assert_eq!(cnt, 4);
/// # println!();
///
/// // Registering from the `err` module is fine too.
/// err::register(errors[4].0, errors[4].1);
///
/// // Applies `check` to the one new error.
/// let (err_count, fatal) = cxt.new_errors_do(check!());
/// assert!(fatal);
/// assert_eq!(err_count, 1);
/// assert_eq!(cnt, 5);
/// ```
///
/// [reg]: fn.register.html (The register function)
#[derive(Debug, Clone)]
pub struct ErrorCxt {
    /// Index of the latest error in `ERRORS` we saw.
    idx: std::cell::RefCell<Option<usize>>,
}

pub use list::{
    register, register_fatal, register_non_fatal, unwrap_register, unwrap_register_fatal,
    unwrap_register_non_fatal,
};

/// Stores a global list of errors.
mod list {
    prelude! {}
    use super::ErrorCxt;

    lazy_static::lazy_static! {
        /// Global list of errors with `is_fatal` flags.
        ///
        /// This is **never** popped. Instead server and clients store an `ErrorIdx` that points to
        /// the latest error they have treated using `since_do`.
        static ref ERRORS: sync::RwLock<Vec<(String, bool)>> = sync::RwLock::new(vec![]);
    }

    /// Destroys a unit result, registering the error if any.
    pub fn unwrap_register<E>(res: Result<(), E>, fatal: bool)
    where
        E: Into<err::Error>,
    {
        match res {
            Ok(()) => (),
            Err(e) => {
                register(e, fatal);
                ()
            }
        }
    }
    /// Destroys a unit result, registering the error as fatal if any.
    pub fn unwrap_register_non_fatal<E>(res: Result<(), E>)
    where
        E: Into<err::Error>,
    {
        unwrap_register(res, false)
    }
    /// Destroys a unit result, registering the error as fatal if any.
    pub fn unwrap_register_fatal<E>(res: Result<(), E>)
    where
        E: Into<err::Error>,
    {
        unwrap_register(res, true)
    }

    /// Registers an error in the global list of errors.
    pub fn register(e: impl Into<err::Error>, fatal: bool) {
        let mut errors = ERRORS.write().expect("global error list was poisoned");
        errors.push((e.into().to_pretty(), fatal))
    }
    /// Registers a non-fatal error in the global list of errors.
    pub fn register_non_fatal(e: impl Into<err::Error>) {
        register(e, false)
    }
    /// Registers a fatal error in the global list of errors.
    pub fn register_fatal(e: impl Into<err::Error>) {
        register(e, true)
    }

    impl ErrorCxt {
        /// Constructor.
        pub fn new() -> Self {
            Self {
                idx: std::cell::RefCell::new(None),
            }
        }

        /// Registers a non-fatal error in the global list of errors.
        ///
        /// See the [type-level documentation][ty doc] for examples.
        ///
        /// [ty doc]: struct.ErrorCxt.html (Type-level documentation)
        pub fn register_non_fatal(&self, e: impl Into<err::Error>) {
            register_non_fatal(e)
        }
        /// Registers a fatal error in the global list of errors.
        ///
        /// See the [type-level documentation][ty doc] for examples.
        ///
        /// [ty doc]: struct.ErrorCxt.html (Type-level documentation)
        pub fn register_fatal(&self, e: impl Into<err::Error>) {
            register_fatal(e)
        }
        /// Registers an error in the global list of errors.
        ///
        /// See the [type-level documentation][ty doc] for examples.
        ///
        /// [ty doc]: struct.ErrorCxt.html (Type-level documentation)
        pub fn register(&self, e: impl Into<err::Error>, fatal: bool) {
            register(e, fatal)
        }

        /// Applies an action to the new errors in the global list of errors.
        ///
        /// Returns the number of new errors to which `action` was applied.
        ///
        /// **NB**: errors only appear once *across calls*. So when `action(e_i)` runs, future calls
        /// to this function will never run `action(e_i)` again.
        ///
        /// See the [type-level documentation][ty doc] for examples.
        ///
        /// [ty doc]: struct.ErrorCxt.html (Type-level documentation)
        pub fn new_errors_do(&mut self, mut action: impl FnMut(&str, bool)) -> (usize, bool) {
            match self.new_errors_try(|err, is_fatal| {
                action(err, is_fatal);
                Ok(()) as Result<(), Inhabited>
            }) {
                Ok(count) => count,
                Err(e) => match e {},
            }
        }

        /// Applies an action that can fail to the new errors in the global list of errors.
        ///
        /// Returns the number of new errors to which `action` was applied.
        ///
        /// **NB**: errors only appear once *across calls*. So when `action(e_i)` runs, future calls
        /// to this function will never run `action(e_i)` again.
        ///
        /// See the [type-level documentation][ty doc] for examples.
        ///
        /// [ty doc]: struct.ErrorCxt.html (Type-level documentation)
        pub fn new_errors_try<E>(
            &mut self,
            mut action: impl FnMut(&str, bool) -> Result<(), E>,
        ) -> Result<(usize, bool), E> {
            let errors = ERRORS.read().expect("global error list was poisoned");

            let mut idx = self.idx.borrow_mut();

            if errors.is_empty() {
                return Ok((0, false));
            }

            let lb = if let Some(idx) = *idx {
                if idx + 1 == errors.len() {
                    return Ok((0, false));
                }
                idx + 1
            } else {
                0
            };

            let new_errors = &errors[lb..];
            let count = new_errors.len();
            let mut fatal = false;

            for (offset, (e, is_fatal)) in new_errors.iter().enumerate() {
                *idx = Some(lb + offset);
                action(e, *is_fatal)?;
                if *is_fatal {
                    fatal = true
                }
            }

            Ok((count, fatal))
        }
    }
}
