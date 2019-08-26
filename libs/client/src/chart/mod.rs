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
    pub fn update(&mut self, data: &Storage) {
        for time_chart in &mut self.charts {
            time_chart.update(data)
        }
    }

    /// Renders itself as HTML.
    pub fn render(&self) -> Html {
        info! { "rendering charts" }
        html! {
            <g id={HTML_CHART_CONTAINER_ID}>
                { for self.charts.iter().map(time::TimeChart::render) }
            </g>
        }
    }

    /// Initial JS setup.
    pub fn init(&mut self, data: &Storage) {
        info! { "init update" }
        for time_chart in &mut self.charts {
            time_chart.init(data)
        }
    }
}
