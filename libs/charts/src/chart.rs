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

//! Chart types and helpers.

prelude! {}

use axis::{XAxis, YAxis};

pub mod spec;

pub mod axis;
pub mod settings;
pub mod time;
pub use spec::ChartSpec;

/// A chart with no UID.
#[cfg(any(test, feature = "server"))]
pub enum RawChart {
    /// A time chart.
    Time(time::TimeChart),
}

#[cfg(any(test, feature = "server"))]
impl RawChart {
    /// Constructor.
    fn new_points(
        &mut self,
        filters: &mut Filters,
        init: bool,
        resolution: settings::Resolution,
        time_windopt: &TimeWindopt,
    ) -> Res<Option<Points>> {
        match self {
            Self::Time(time_chart) => {
                time_chart.new_points(filters, init, resolution, time_windopt)
            }
        }
    }

    /// Resets a raw chart.
    fn reset(&mut self, filters: &filter::Filters) {
        match self {
            Self::Time(chart) => chart.reset(filters),
        }
    }
}

#[cfg(any(test, feature = "server"))]
impl RawChart {
    /// Constructor.
    pub fn new(filters: &filter::Filters, x_axis: XAxis, y_axis: YAxis) -> Res<Self> {
        let chart = match x_axis {
            XAxis::Time => Self::Time(match y_axis {
                YAxis::TotalSize => time::TimeChart::new_total_size(filters),
            }),
        };
        Ok(chart)
    }
}

/// A chart a specification and some settings.
#[cfg(any(test, feature = "server"))]
pub struct Chart {
    /// Chart specification.
    spec: ChartSpec,
    /// Chart settings.
    settings: settings::Chart,
    /// Raw chart.
    #[allow(dead_code)]
    chart: RawChart,
    /// If true, the chart has not been initialized yet.
    ///
    /// This typically happens server-side, as the server needs the actual resolution of the chart
    /// (which only the client-side knows) before it can send the initial points.
    still_init: bool,
}
#[cfg(any(test, feature = "server"))]
impl Chart {
    /// Creates a chart.
    pub fn new(
        filters: &filter::Filters,
        x_axis: XAxis,
        y_axis: YAxis,
        active: BTMap<uid::Line, bool>,
    ) -> Res<Self> {
        let spec = ChartSpec::new(x_axis, y_axis, active);
        let settings = settings::Chart::from_axes(spec.desc(), x_axis, y_axis);
        let chart = RawChart::new(filters, x_axis, y_axis)?;
        let slf = Self {
            spec,
            settings,
            chart,
            still_init: true,
        };
        Ok(slf)
    }

    /// Constructor.
    pub fn from_spec(title: Option<String>, filters: &Filters, spec: ChartSpec) -> Res<Self> {
        let settings = settings::Chart::from_axes(
            title.unwrap_or_else(|| spec.desc()),
            spec.x_axis().clone(),
            spec.y_axis().clone(),
        );
        let chart = RawChart::new(filters, spec.x_axis().clone(), spec.y_axis().clone())?;
        Ok(Self {
            spec,
            settings,
            chart,
            still_init: true,
        })
    }

    /// Applies an update to its settings.
    pub fn update(&mut self, msg: msg::to_server::ChartMsg) -> bool {
        use msg::to_server::ChartMsg::*;
        match msg {
            SettingsUpdate(msg) => self.settings.update(msg),
        }
    }

    /// Spec accessor.
    #[inline]
    pub fn spec(&self) -> &ChartSpec {
        &self.spec
    }

    /// Settings accessor.
    #[inline]
    pub fn settings(&self) -> &settings::Chart {
        &self.settings
    }
    /// Settings mutable accessor.
    #[inline]
    pub fn settings_mut(&mut self) -> &mut settings::Chart {
        &mut self.settings
    }

    /// UID accessor.
    #[inline]
    pub fn uid(&self) -> uid::Chart {
        self.spec().uid()
    }
}

#[cfg(any(test, feature = "server"))]
impl Chart {
    /// Retrieves new points since the last time it was called.
    pub fn new_points(
        &mut self,
        init: bool,
        filters: &mut Filters,
        time_windopt: &TimeWindopt,
    ) -> Res<Option<Points>> {
        self.still_init = self.still_init || init;
        if let Some(resolution) = self.settings.resolution() {
            let res = self
                .chart
                .new_points(filters, self.still_init, resolution, time_windopt);
            self.still_init = false;
            res
        } else {
            Ok(None)
        }
    }

    /// Resets a chart.
    pub fn reset(&mut self, filters: &filter::Filters) {
        self.chart.reset(filters)
    }
}
