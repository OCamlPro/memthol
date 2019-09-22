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

impl TimePoints {
    /// True if there are no points.
    pub fn is_empty(&self) -> bool {
        match self {
            Self::Size(points) => points.is_empty(),
        }
    }
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

impl TimeChart {
    /// Default constructor.
    pub fn default(filters: &Filters) -> Self {
        Self::Size(TimeSize::default(filters))
    }
}

impl ChartExt for TimeChart {
    fn new_points(&mut self, filters: &mut Filters, init: bool) -> Res<Points> {
        match self {
            Self::Size(time_size_chart) => time_size_chart.new_points(filters, init),
        }
    }

    fn reset(&mut self, filters: &Filters) {
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
