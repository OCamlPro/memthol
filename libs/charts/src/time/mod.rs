//! Types and helpers for time charts.

use crate::base::*;

pub mod size;

/// A time chart.
pub struct TimeChart {
    /// Time of the latest allocation.
    timestamp: SinceStart,
}
impl TimeChart {
    /// Constructor.
    pub fn new() -> Self {
        let timestamp = SinceStart::zero();
        Self { timestamp }
    }
}
