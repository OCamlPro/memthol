//! Crate's prelude.

pub use std::{
    convert::{TryFrom, TryInto},
    fmt,
};

pub use crate::{
    err::{self, Res, ResExt},
    mem::{self, labels::Labels, str::Str, trace::Trace},
    parser::{self, Parseable},
    time::{self, Lifetime, SinceStart},
    Alloc, AllocKind, BigUint, CLoc, Date, Diff, Duration, Init, Loc, Span, Uid,
};

pub use base::{error_chain, lazy_static};

/// Imports this crate's prelude.
#[macro_export]
macro_rules! prelude {
    () => {
        use $crate::prelude::*;
    };
}
