//! Crate's prelude.

pub use std::convert::{TryFrom, TryInto};

pub use crate::{
    err::{self, Res, ResExt},
    labels, locs, mem,
    parser::{self, Parseable},
    time, Alloc, AllocKind, BigUint, CLoc, Date, Diff, Duration, Init, Loc, SinceStart, Span, Uid,
};

pub use base::{error_chain, lazy_static};

/// Imports this crate's prelude.
macro_rules! prelude {
    () => {
        use $crate::prelude::*;
    };
}
