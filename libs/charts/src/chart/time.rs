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

//! Types and helpers for time charts.

prelude! {}

pub mod size;

pub use size::TimeSize;

/// A time chart.
#[derive(Debug, Serialize, Deserialize)]
pub enum TimeChart {
    /// Total size over time chart.
    Size(TimeSize),
}

impl TimeChart {
    /// Default constructor.
    pub fn default(filters: &Filters) -> Self {
        Self::Size(TimeSize::default(filters))
    }
}

#[cfg(any(test, feature = "server"))]
impl TimeChart {
    /// Extracts the new points since the last time it was called.
    pub fn new_points(
        &mut self,
        filters: &mut Filters,
        init: bool,
        resolution: chart::settings::Resolution,
        time_windopt: &TimeWindopt,
    ) -> Res<Option<Points>> {
        match self {
            Self::Size(time_size_chart) => {
                time_size_chart.new_points(filters, init, resolution, time_windopt)
            }
        }
    }

    /// Resets a chart.
    pub fn reset(&mut self, filters: &Filters) {
        match self {
            Self::Size(chart) => chart.reset(filters),
        }
    }
}

impl TimeChart {
    /// Total size over time constructor.
    pub fn new_total_size(filters: &Filters) -> Self {
        Self::Size(TimeSize::new(filters))
    }
}
