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

//! Crate's prelude.

pub use crate::parser::{self, Parseable};
pub use crate::{
    err::{self, bail, Res, ResExt},
    mem::{self, labels::Labels, str::Str, trace::Trace},
    Alloc, AllocKind, BigUint, Builder, CLoc, Diff, Init, Loc, Span,
};

pub use base::prelude::{serde::*, *};

/// Imports this crate's prelude.
#[macro_export]
macro_rules! prelude {
    () => {
        use $crate::prelude::*;
    };
}
