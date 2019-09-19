//! Chart types and helpers.

use crate::base::{uid::ChartUid, *};

use axis::{XAxis, YAxis};

pub mod axis;
pub mod time;

/// A chart with no UID.
pub enum RawChart {
    /// A time chart.
    Time(time::TimeChart),
}

impl ChartExt for RawChart {
    fn new_points(&mut self, filters: &Filters, init: bool) -> Res<Points> {
        match self {
            Self::Time(time_chart) => time_chart.new_points(filters, init),
        }
    }

    fn reset(&mut self) {
        match self {
            Self::Time(chart) => chart.reset(),
        }
    }
}

impl RawChart {
    /// Creates a raw chart.
    pub fn new(x_axis: XAxis, y_axis: YAxis) -> Res<Self> {
        let chart = match x_axis {
            XAxis::Time => Self::Time(match y_axis {
                YAxis::TotalSize => time::TimeChart::new_total_size(),
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
    pub fn new(x_axis: XAxis, y_axis: YAxis) -> Res<Self> {
        let spec = ChartSpec::new(x_axis, y_axis);
        let chart = RawChart::new(x_axis, y_axis)?;
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
    fn new_points(&mut self, filters: &Filters, init: bool) -> Res<Points> {
        self.chart.new_points(filters, init)
    }

    fn reset(&mut self) {
        self.chart.reset()
    }
}

/// A chart specification, for the client.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChartSpec {
    /// UID,
    uid: ChartUid,
    /// X-axis.
    x_axis: XAxis,
    /// Y-axis.
    y_axis: YAxis,
}
impl ChartSpec {
    /// Creates a new chart spec.
    pub fn new(x_axis: XAxis, y_axis: YAxis) -> Self {
        Self {
            uid: ChartUid::fresh(),
            x_axis,
            y_axis,
        }
    }

    /// Description of a chart.
    pub fn desc(&self) -> String {
        format!("{} over {}", self.y_axis.desc(), self.x_axis.desc())
    }

    /// UID accessor.
    pub fn uid(&self) -> ChartUid {
        self.uid
    }

    /// X-axis accessor.
    pub fn x_axis(&self) -> &XAxis {
        &self.x_axis
    }
    /// Y-axis accessor.
    pub fn y_axis(&self) -> &YAxis {
        &self.y_axis
    }
}
