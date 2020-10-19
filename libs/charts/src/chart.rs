//! Chart types and helpers.

prelude! {}

use axis::{XAxis, YAxis};

pub mod spec;

pub mod axis;
pub mod settings;
pub mod time;

pub use settings::ChartSettings;
pub use spec::ChartSpec;
pub use uid::ChartUid;

/// A chart with no UID.
#[cfg(any(test, feature = "server"))]
pub enum RawChart {
    /// A time chart.
    Time(time::TimeChart),
}

#[cfg(any(test, feature = "server"))]
impl RawChart {
    fn new_points(
        &mut self,
        filters: &mut Filters,
        init: bool,
        _resolution: settings::Resolution,
    ) -> Res<Points> {
        match self {
            Self::Time(time_chart) => time_chart.new_points(filters, init),
        }
    }

    fn reset(&mut self, filters: &filter::Filters) {
        match self {
            Self::Time(chart) => chart.reset(filters),
        }
    }
}

#[cfg(any(test, feature = "server"))]
impl RawChart {
    /// Creates a raw chart.
    pub fn new(filters: &filter::Filters, x_axis: XAxis, y_axis: YAxis) -> Res<Self> {
        let chart = match x_axis {
            XAxis::Time => Self::Time(match y_axis {
                YAxis::TotalSize => time::TimeChart::new_total_size(filters),
            }),
        };
        Ok(chart)
    }
}

#[cfg(any(test, feature = "server"))]
pub struct Chart {
    spec: ChartSpec,
    settings: ChartSettings,
    #[allow(dead_code)]
    chart: RawChart,
    still_init: bool,
}
#[cfg(any(test, feature = "server"))]
impl Chart {
    /// Creates a time chart.
    pub fn new(filters: &filter::Filters, x_axis: XAxis, y_axis: YAxis) -> Res<Self> {
        let spec = ChartSpec::new(x_axis, y_axis);
        let settings = ChartSettings::from_axes(spec.desc(), x_axis, y_axis);
        let chart = RawChart::new(filters, x_axis, y_axis)?;
        let slf = Self {
            spec,
            settings,
            chart,
            still_init: true,
        };
        Ok(slf)
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
    pub fn settings(&self) -> &ChartSettings {
        &self.settings
    }
    /// Settings mutable accessor.
    #[inline]
    pub fn settings_mut(&mut self) -> &mut ChartSettings {
        &mut self.settings
    }

    /// UID accessor.
    #[inline]
    pub fn uid(&self) -> ChartUid {
        self.spec().uid()
    }
}

#[cfg(any(test, feature = "server"))]
impl Chart {
    pub fn new_points(&mut self, filters: &mut Filters, init: bool) -> Res<Option<Points>> {
        self.still_init = self.still_init || init;
        if let Some(resolution) = self.settings.resolution() {
            let res = self
                .chart
                .new_points(filters, self.still_init, resolution)
                .map(Some);
            self.still_init = false;
            res
        } else {
            Ok(None)
        }
    }

    pub fn reset(&mut self, filters: &filter::Filters) {
        self.chart.reset(filters)
    }
}
