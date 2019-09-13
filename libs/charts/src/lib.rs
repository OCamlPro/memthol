//! Server-side chart handling.

pub mod base;
mod chart;
pub mod data;
pub mod err;
pub mod filter;
pub mod time;

pub use chart::Chart;

use base::*;

/// Aggregates some charts.
pub struct Charts {
    /// List of active charts.
    charts: Vec<Chart>,
    /// List of filters.
    filters: Vec<Filter>,
}
