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

cfg_item! {
    cfg(client) {
        pub use crate::Memory;
    }
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
