//! A chart combines axes to create points.

use crate::base::*;

pub mod axis;
pub mod boxed_chart;
pub mod time;
pub mod window;

pub use boxed_chart::default;
use boxed_chart::*;

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

/// A chart.
pub struct Chart {
    /// Chart UID.
    uid: ChartUid,
    /// Abstract boxed chart.
    chart: Option<BChart>,
    /// Chart HTML identifier (unique).
    html_id: String,
    /// Chart window specification.
    window: window::Window,
}

impl Chart {
    /// Constructor.
    pub fn new() -> Self {
        let (uid, html_id) = generate_chart_uid_and_id();
        let window = window::Window::default();
        Self {
            uid,
            chart: None,
            html_id,
            window,
        }
    }

    /// UID accessor.
    pub fn uid(&self) -> ChartUid {
        self.uid
    }

    /// Renders the chart as HTML.
    pub fn render(&self) -> Html {
        html! {
            <center>
                <div id={&self.html_id}
                    style={self.window.as_style()}
                />
            </center>
        }
    }

    /// Initial setup for JS.
    pub fn init_setup(&mut self, data: &data::Storage) {
        let mut chart = boxed_chart::default(&self.html_id);
        data.iter(|alloc| chart.add_alloc(alloc));
        self.chart = Some(chart)
    }

    /// Initial setup for JS.
    pub fn init_setup_for<X, Y>(&mut self, data: &data::Storage)
    where
        X: axis::XAxis + 'static,
        X::Value: axis::CloneSubOrd,
        Y: axis::YAxis + 'static,
        Y::Value: axis::CloneSubOrd,
    {
        let chart_data: ChartData<X, Y> = boxed_chart::ChartData::init(&self.html_id);
        let mut chart = Box::new(chart_data);
        data.iter(|alloc| chart.add_alloc(alloc));
        self.chart = Some(chart)
    }

    /// Chart accessor.
    ///
    /// Only legal after `init_setup`.
    #[allow(dead_code)]
    fn get_chart(&self) -> &BChart {
        self.chart.as_ref().unwrap_or_else(|| {
            panic!(
                "trying to retrieve the actual chart of uninitialized chart #{} `{}`",
                self.uid, self.html_id
            )
        })
    }

    /// Chart accessor (mutable).
    ///
    /// Only legal after `init_setup`.
    fn get_chart_mut(&mut self) -> &mut BChart {
        if let Some(chart) = self.chart.as_mut() {
            chart
        } else {
            panic!(
                "trying to retrieve the actual chart of uninitialized chart #{} `{}`",
                self.uid, self.html_id
            )
        }
    }

    /// Registers a new allocation.
    pub fn add_alloc(&mut self, alloc: &Alloc) {
        self.get_chart_mut().add_alloc(alloc)
    }

    /// Registers a new death.
    pub fn add_death(&mut self, uid: &AllocUid, tod: AllocDate) {
        self.get_chart_mut().add_death(uid, tod)
    }

    /// Updates the actual chart.
    pub fn update(&mut self) {
        self.get_chart_mut().update()
    }
}

/// Name of the HTML container for all charts.
static HTML_CHART_CONTAINER_ID: &str = "memthol_chart_container";

/// Stores the collection of charts.
pub struct Charts {
    /// Storage containing all the allocation data.
    data: data::Storage,
    // /// The collection of charts.
    // charts: Vec<Chart>,
    /// Time / total size chart.
    time_total_size: time::TimeTotalSizeChart,
}
impl Charts {
    /// Constructor.
    pub fn new() -> Self {
        let (uid, id) = generate_chart_uid_and_id();
        Self {
            data: data::Storage::new(),
            time_total_size: time::TimeTotalSizeChart::new(uid, id),
        }
    }

    // /// Adds a new graph.
    // ///
    // /// Returns its index.
    // pub fn add(&mut self, chart: Chart) -> usize {
    //     let idx = self.charts.len();
    //     self.charts.push(chart);
    //     idx
    // }

    /// Adds some allocation data.
    pub fn add_alloc(&mut self, alloc: Alloc) {
        let alloc = self.data.add_alloc(alloc);
        if let Some(alloc) = alloc {
            self.time_total_size.add_alloc(alloc)
        }
    }

    /// Registers the death of some allocation data.
    pub fn add_death(&mut self, uid: &AllocUid, tod: AllocDate) {
        self.data.add_death(uid, tod);
        self.time_total_size.add_death(uid, tod)
    }

    /// Registers a diff.
    ///
    /// Does **not** update the actual charts.
    pub fn add_diff(&mut self, AllocDiff { new, dead, .. }: AllocDiff) {
        for alloc in new {
            self.add_alloc(alloc)
        }
        for (uid, tod) in dead {
            self.add_death(&uid, tod)
        }
    }

    /// Updates the actual charts.
    pub fn update(&mut self) {
        self.time_total_size.update()
    }

    /// Renders itself as HTML.
    pub fn render(&self) -> Html {
        info! { "rendering charts" }
        html! {
            <g id={HTML_CHART_CONTAINER_ID}>
                { self.time_total_size.render() }
            </g>
        }
    }

    /// Initial JS setup.
    pub fn init_setup(&mut self) {
        info! { "init update" }
        self.time_total_size.init()
    }

    /// Initializes the start date of the profiling run.
    pub fn date_init(&mut self, date: AllocNuDate) {
        self.time_total_size.date_init(date)
    }
}
