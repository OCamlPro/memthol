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

//! Filter specification.

use super::*;

/// A filter specification.
///
/// Contains the following:
///
/// - an optional UID;
/// - a name;
/// - a color.
///
/// The UID is optional because the filter specification can belong the "catch all" line of charts.
/// It is made from the points that all filters miss.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FilterSpec {
    /// Uid of the filter.
    ///
    /// None if the specification if for the catch-all filter.
    uid: uid::Line,
    /// Name of the filter.
    name: String,
    /// Color of the filter.
    color: Color,
}
impl FilterSpec {
    /// Constructor for user-defined filters.
    pub fn new(color: Color) -> Self {
        let uid = uid::Filter::fresh();
        let name = "new filter".to_string();
        Self {
            uid: uid::Line::Filter(uid),
            name,
            color,
        }
    }

    /// Constructs a specification for the catch-all filter.
    pub fn new_catch_all() -> Self {
        Self {
            uid: uid::Line::CatchAll,
            name: "catch all".into(),
            color: Color::new(0x01, 0x93, 0xff),
        }
    }

    /// Constructs a specification for the everything filter.
    pub fn new_everything() -> Self {
        Self {
            uid: uid::Line::Everything,
            name: "everything".into(),
            color: Color::new(0xff, 0x66, 0x00),
        }
    }

    /// True if the specification describes the *everything* filter.
    pub fn is_everything(&self) -> bool {
        self.uid == uid::Line::Everything
    }
    /// True if the specification describes the *catch-all* filter.
    pub fn is_catch_all(&self) -> bool {
        self.uid == uid::Line::CatchAll
    }
    /// True if the filter is user-provided.
    pub fn is_user_provided(&self) -> bool {
        !(self.is_everything() || self.is_catch_all())
    }

    /// UID accessor.
    pub fn uid(&self) -> uid::Line {
        self.uid
    }

    /// Name accessor.
    pub fn name(&self) -> &str {
        &self.name
    }
    /// Name setter.
    pub fn set_name(&mut self, name: impl Into<String>) {
        self.name = name.into()
    }

    /// Color accessor.
    pub fn color(&self) -> &Color {
        &self.color
    }
    /// Color setter.
    pub fn set_color(&mut self, color: Color) {
        self.color = color
    }
}
