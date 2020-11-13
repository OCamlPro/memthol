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

//! Axis-related stuff.

prelude! {}

/// X-axis spec.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, strum_macros::EnumIter)]
pub enum XAxis {
    /// Time.
    Time,
}
impl XAxis {
    /// Description of a x-axis.
    pub fn desc(&self) -> &'static str {
        match self {
            Self::Time => "time",
        }
    }

    /// The legal y-axes that can be combined with this x-axis.
    pub fn y_axes(&self) -> Vec<YAxis> {
        match self {
            Self::Time => vec![YAxis::TotalSize],
        }
    }

    /// A list of all the x-axes.
    pub fn all() -> Vec<XAxis> {
        use strum::IntoEnumIterator;
        Self::iter().collect()
    }
}

impl fmt::Display for XAxis {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        self.desc().fmt(fmt)
    }
}

impl Default for XAxis {
    fn default() -> Self {
        XAxis::Time
    }
}

/// Y-axis spec.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum YAxis {
    /// Total size.
    TotalSize,
    // /// Highest lifetime.
    // MaxLifetime,
}
impl YAxis {
    /// Description of a y-axis.
    pub fn desc(self) -> &'static str {
        match self {
            Self::TotalSize => "total size",
            // Self::MaxLifetime => "highest lifetime",
        }
    }

    /// True if `self` supports stacked-area rendering.
    pub fn can_stack_area(self) -> bool {
        match self {
            Self::TotalSize => true,
        }
    }
}

impl fmt::Display for YAxis {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        self.desc().fmt(fmt)
    }
}
