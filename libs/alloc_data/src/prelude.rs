//! Crate's prelude.

pub use std::{
    collections::BTreeMap as Map,
    convert::{TryFrom, TryInto},
    fmt,
};

pub use crate::parser::{self, Parseable};
pub use crate::{
    err::{self, bail, Res, ResExt},
    mem::{self, labels::Labels, str::Str, trace::Trace},
    time::{self, Lifetime, SinceStart},
    Alloc, AllocKind, BigUint, CLoc, Date, Diff, Duration, Init, Loc, Span, Uid,
};

pub use base::{chrono, convert, error_chain, lazy_static, SVec16, SVec32};

/// Re-exports of the serde traits for auto-implementations.
pub mod serderive {
    pub use serde_derive::{Deserialize, Serialize};
}
pub use serderive::*;

/// Imports this crate's prelude.
#[macro_export]
macro_rules! prelude {
    () => {
        use $crate::prelude::*;
    };
}
