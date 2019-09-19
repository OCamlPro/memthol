//! Types and helpers for time charts.

use crate::base::*;

pub mod size;

pub use size::{TimeSize, TimeSizePoint, TimeSizePoints};

/// A point for a time chart.
pub type TimePoint<Val> = Point<Date, Val>;

/// Some points for a time chart.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TimePoints {
    Size(size::TimeSizePoints),
}
impl From<size::TimeSizePoints> for TimePoints {
    fn from(points: size::TimeSizePoints) -> Self {
        Self::Size(points)
    }
}

/// A time chart.
#[derive(Debug, Serialize, Deserialize)]
pub enum TimeChart {
    /// Total size over time chart.
    Size(TimeSize),
}

impl Default for TimeChart {
    fn default() -> Self {
        Self::Size(TimeSize::default())
    }
}

impl ChartExt for TimeChart {
    fn new_points(&mut self, filters: &Filters, init: bool) -> Res<Points> {
        match self {
            Self::Size(time_size_chart) => time_size_chart.new_points(filters, init),
        }
    }

    fn reset(&mut self) {
        match self {
            Self::Size(chart) => chart.reset(),
        }
    }
}

impl TimeChart {
    /// Total size over time constructor.
    pub fn new_total_size() -> Self {
        Self::Size(TimeSize::new())
    }
}
