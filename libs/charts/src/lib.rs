//! Server-side chart handling.
//!
//! Note that most types in this crate implement (de)serialization to/from json. These types
//! implement the [`Json`] trait which provides straightforward conversion functions. This is used
//! heavily for server/client exchanges.
//!
//! # Basic Workflow
//!
//! All allocation-related data is stored in a global state in the [`data`] module. It features a
//! [`Watcher`] type which, after [`start`]ing it, will monitor a directory for init and diff files.
//!
//! [`Json`]: ./base/trait.Json.html (The Json trait)
//! [`data`]: ./data/index.html (The data module)
//! [`Watcher`]: ./data/struct.Watcher.html (The Watcher struct in module data)
//! [`start`]: ./data/fn.start.html (The start function in module data)

pub mod base;
pub mod chart;
pub mod data;
pub mod err;
pub mod filter;
pub mod index;
pub mod msg;
pub mod point;
pub mod time;

pub use base::Json;
pub use chart::Chart;

use base::*;

/// Trait implemented by all charts.
pub trait ChartExt: Default {
    /// Generates the new points of the chart.
    fn new_points(&mut self, filters: &Filters, init: bool) -> Res<Points>;
}

/// Aggregates some charts.
pub struct Charts {
    /// List of active charts.
    charts: Vec<Chart>,
    /// List of filters.
    filters: Filters,
}

impl Charts {
    /// Constructor.
    pub fn new() -> Self {
        Self {
            charts: vec![],
            filters: Filters::new(),
        }
    }

    /// Pushes a new chart.
    pub fn push(&mut self, chart: Chart) {
        self.charts.push(chart)
    }

    /// Extracts the new points for the different charts.
    pub fn new_points(&mut self, init: bool) -> Res<Vec<Points>> {
        let mut res = Vec::with_capacity(self.charts.len());
        for chart in &mut self.charts {
            res.push(chart.new_points(&self.filters, init)?);
        }
        Ok(res)
    }

    /// Handles a message from the client.
    pub fn handle_msg(&mut self, msg: msg::to_server::Msg) -> Res<()> {
        use msg::to_server::Msg::*;

        match msg {
            Filters { msg } => self.filters.update(msg),
        }

        Ok(())
    }
}
