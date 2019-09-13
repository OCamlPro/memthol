//! Chart types and helpers.

use crate::base::{time::TimeChart, *};

/// A chart.
pub enum Chart {
    /// A time chart.
    Time(TimeChart),
}
impl Chart {
    /// Creates a time chart.
    pub fn new_time() -> Self {
        Self::Time(TimeChart::new())
    }
}
