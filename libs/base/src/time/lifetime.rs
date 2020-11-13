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

//! A lifetime is a thin wrapper around a duration.

prelude! { time::* }

/// Wrapper around a duration representing a lifetime.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Lifetime {
    /// Actual duration.
    duration: Duration,
}

impl Lifetime {
    /// A duration of 0 nanoseconds.
    pub fn zero() -> Self {
        Lifetime {
            duration: std::time::Duration::new(0, 0),
        }
    }
}

impl DurationExt for Lifetime {
    fn as_duration(&self) -> &std::time::Duration {
        &self.duration
    }
}

implement! {
    impl Lifetime {
        Display {
            |&self, fmt| {
                self.display_micros().fmt(fmt)
            }
        }

        Default {
            Self { duration: Duration::default() }
        }

        From {
            from Duration => |duration| Self { duration }
        }
        Into {
            to Duration => |self| self.duration
        }

        Deref {
            to Duration => |&self| &self.duration
        }
    }
}
