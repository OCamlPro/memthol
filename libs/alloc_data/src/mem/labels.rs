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

//! Handles the internals of label sharing.

use crate::prelude::Str;

// Macro defined in `crate::mem`.
new! {
    mod mem for Vec<super::Str>, uid: Labels
}

pub use mem::{AsRead, AsWrite, Labels};

/// Registers a list of labels and returns its UID.
pub fn add(labels: Vec<Str>) -> Labels {
    let mut mem = mem::write();
    mem.get_uid(labels)
}

/// Retrieves a list of labels from its UID.
pub fn get(uid: Labels) -> std::sync::Arc<Vec<Str>> {
    let mem = mem::read();
    mem.get_elm(uid)
}
