//! Crate's prelude.

pub use crate::parser::{self, Parseable};
pub use crate::{
    err::{self, bail, Res, ResExt},
    mem::{self, labels::Labels, str::Str, trace::Trace},
    Alloc, AllocKind, BigUint, CLoc, Diff, Init, Loc, Span,
};

pub use base::prelude::{serde::*, *};

/// Imports this crate's prelude.
#[macro_export]
macro_rules! prelude {
    () => {
        use $crate::prelude::*;
    };
}
