//! Chart types and helpers.

use crate::base::{time::TimeChart, *};

/// A chart.
pub enum Chart {
    /// A time chart.
    Time(TimeChart),
}

impl Default for Chart {
    fn default() -> Self {
        Self::Time(TimeChart::default())
    }
}

impl ChartExt for Chart {
    fn new_points(&mut self, filters: &Filters, init: bool) -> Res<Points> {
        match self {
            Self::Time(time_chart) => time_chart.new_points(filters, init),
        }
    }
}

impl Chart {
    /// Creates a time chart.
    pub fn new_time() -> Self {
        Self::Time(TimeChart::new())
    }
}
