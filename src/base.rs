//! Basic types and helpers used by the whole crate.

pub use std::{
    collections::BTreeSet as Set,
    net::{TcpListener, TcpStream},
};

pub use error_chain::bail;

pub use crate::err::{ErrorKind, Res, ResultExt};
