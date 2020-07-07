//! Crate's prelude.

pub use std::convert::{TryFrom, TryInto};

pub use crate::{
    err::{self, Res, ResExt},
    labels, locs, mem,
    parser::{self, Parseable},
    Alloc, AllocKind, BigUint, CLoc, Date, Diff, Duration, Init, Loc, SinceStart, Span, Uid,
};

#[macro_export]
macro_rules! prelude {
    () => {
        use $crate::prelude::*;
    };
}
