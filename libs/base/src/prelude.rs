//! Common imports.

pub use std::{
    borrow::Borrow,
    collections::{BTreeMap as BTMap, BTreeSet as BTSet, HashMap as HMap, HashSet as HSet},
    convert::{TryFrom, TryInto},
    fmt, ops,
    str::FromStr,
    sync::{self, Arc},
};

pub use either::Either;
pub use lazy_static::lazy_static;

/// Log macros re-exports.
pub mod log {
    pub use log::{debug, error, info, trace, warn};
}

pub use crate::{
    convert, destroy,
    err::{self, Res, ResExt},
    error_chain::{self, bail},
    identity,
    time::{self, DurationExt},
    time_stats, uid, Range, SVec16, SVec32, SVec64, SVec8, SampleRate,
};

/// Serde trait re-exports.
pub mod serde {
    pub use serde_derive::{Deserialize, Serialize};
}

/// Inhabited type.
pub enum Inhabited {}
