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

//! Common imports for the modules in this crate.

pub use base::prelude::*;

pub use crate::{parse::CanParse, *};

/// A duration since the start of the run as microseconds.
pub type Clock = u64;
/// A difference between two [`Clock`] values.
///
/// [`Clock`]: type.Clock.html (Clock type)
pub type DeltaClock = u64;

/// Type of allocation UIDs.
pub type AllocUid = u64;

/// Type of PIDs.
pub type Pid = u64;
