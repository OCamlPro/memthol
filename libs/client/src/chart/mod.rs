//! A chart combines axes to create points.

use crate::base::*;

pub mod time;

new_uid! {
    mod chart_uid {
        uid: ChartUid,
        set: ChartUidSet,
        map: ChartUidMap,
    }
}
pub use chart_uid::*;

/// Prefix for the HTML identifier of all the charts.
pub static CHART_HTML_PREFIX: &str = "memthol_chart_html_id";

/// Generates a fresh uid and constructs a unique HTML id.
fn generate_chart_uid_and_id() -> (ChartUid, String) {
    let uid = ChartUid::fresh();
    let id = format!("{}_{}", CHART_HTML_PREFIX, uid);
    (uid, id)
}

/// Name of the HTML container for all charts.
static HTML_CHART_CONTAINER_ID: &str = "memthol_chart_container";

/// Stores the collection of charts.
pub struct Charts {
    /// Time charts.
    charts: Vec<time::TimeChart>,
}
impl Charts {
    /// Constructor.
    pub fn new() -> Self {
        let (uid, id) = generate_chart_uid_and_id();
        let total_size = time::TimeChart::total_size(id, uid);
        let (uid, id) = generate_chart_uid_and_id();
        let highest_lifetime = time::TimeChart::highest_lifetime(id, uid);
        Self {
            charts: vec![total_size, highest_lifetime],
        }
    }

    /// Updates the actual charts.
    pub fn update_data(&mut self, data: &Storage) {
        for time_chart in &mut self.charts {
            time_chart.update(data)
        }
    }

    /// Renders itself as HTML.
    pub fn render(&self) -> Html {
        html! {
            <g id={HTML_CHART_CONTAINER_ID}>
                { for self.charts.iter().map(time::TimeChart::render) }
            </g>
        }
    }

    /// Initial JS setup.
    pub fn init(&mut self, data: &Storage) {
        for time_chart in &mut self.charts {
            time_chart.init(data)
        }
    }
}

/// # Actions.
impl Charts {
    /// Handles a chart message.
    pub fn update(&mut self, msg: msg::ChartsMsg) -> ShouldRender {
        use msg::ChartsMsg::*;
        match msg {
            // Refresh-s happen when the layout has changed.
            //
            // When this happens, all charts must be deactivated before we can refresh their
            // respective targets.
            RefreshAll => {
                let charts_and_data: Vec<_> = self
                    .charts
                    .iter_mut()
                    .map(|chart| {
                        let data = chart.chart_dispose();
                        (chart, data)
                    })
                    .collect();
                for (chart, data) in charts_and_data {
                    chart.refresh_target(data)
                }
                false
            }
            Close { uid } => self.close_chart(uid),
            Move { uid, up } => self.move_chart(uid, up),
            Visibility { uid, show } => {
                if show {
                    self.expand_chart(uid)
                } else {
                    self.collapse_chart(uid)
                }
            }
        }
    }

    /// Collapses a chart.
    pub fn collapse_chart(&mut self, uid: ChartUid) -> ShouldRender {
        for chart in &mut self.charts {
            if chart.uid() == &uid {
                let should_render = chart.collapse();
                return should_render;
            }
        }
        warn!("asked to collapse chart #{} which does not exist", uid);
        false
    }
    /// Expands a chart.
    pub fn expand_chart(&mut self, uid: ChartUid) -> ShouldRender {
        for chart in &mut self.charts {
            if chart.uid() == &uid {
                let should_render = chart.expand();
                return should_render;
            }
        }
        warn!("asked to expand chart #{} which does not exist", uid);
        false
    }

    /// Closes a chart.
    pub fn close_chart(&mut self, uid: ChartUid) -> ShouldRender {
        let mut index = None;
        for (idx, chart) in self.charts.iter().enumerate() {
            if chart.uid() == &uid {
                index = Some(idx)
            }
        }
        if let Some(idx) = index {
            let chart = self.charts.remove(idx);
            chart.destroy();
            true
        } else {
            warn!(
                "asked to destroy chart #{}, but there is no such chart",
                uid
            );
            false
        }
    }

    /// Move a chart.
    pub fn move_chart(&mut self, uid: ChartUid, up: bool) -> ShouldRender {
        let mut index = None;
        for (idx, chart) in self.charts.iter().enumerate() {
            if chart.uid() == &uid {
                index = Some(idx)
            }
        }

        if let Some(index) = index {
            if up && 0 < index {
                self.charts.swap(index - 1, index);
                return true;
            } else if !up && index + 1 < self.charts.len() {
                self.charts.swap(index, index + 1);
                return true;
            }
        } else {
            warn!(
                "asked to move chart #{} {}, but there is no such chart",
                uid,
                if up { "up" } else { "down" }
            )
        }

        false
    }
}
