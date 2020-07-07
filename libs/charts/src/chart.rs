//! Chart types and helpers.

prelude! {}

use axis::{XAxis, YAxis};

pub mod axis;
mod spec;
pub mod time;

pub use spec::ChartSpec;
pub use uid::ChartUid;

/// A chart with no UID.
pub enum RawChart {
    /// A time chart.
    Time(time::TimeChart),
}

impl ChartExt for RawChart {
    fn new_points(&mut self, filters: &mut Filters, init: bool) -> Res<Points> {
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

pub struct Chart {
    spec: ChartSpec,
    chart: RawChart,
}
impl Chart {
    /// Creates a time chart.
    pub fn new(filters: &filter::Filters, x_axis: XAxis, y_axis: YAxis) -> Res<Self> {
        let spec = ChartSpec::new(x_axis, y_axis);
        let chart = RawChart::new(filters, x_axis, y_axis)?;
        let slf = Self { spec, chart };
        Ok(slf)
    }

    /// Spec accessor.
    #[inline]
    pub fn spec(&self) -> &ChartSpec {
        &self.spec
    }

    /// UID accessor.
    #[inline]
    pub fn uid(&self) -> ChartUid {
        self.spec().uid()
    }
}

impl ChartExt for Chart {
    fn new_points(&mut self, filters: &mut Filters, init: bool) -> Res<Points> {
        self.chart.new_points(filters, init)
    }

    fn reset(&mut self, filters: &filter::Filters) {
        self.chart.reset(filters)
    }
}
