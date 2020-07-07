//! Types and helpers for time charts.

use crate::common::*;

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
