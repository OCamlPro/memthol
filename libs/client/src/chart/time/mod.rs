//! Time charts.

use crate::base::*;

use msg::ChartsMsg;

mod value;

pub use value::Value;

/// A time chart.
pub struct TimeChart {
    /// HTML identifier of the chart.
    html_id: String,
    /// UID of the chart.
    uid: ChartUid,
    /// Type of value we're building.
    value: Value,
    /// Actual chart.
    chart: Option<JsVal>,
    /// Chart id.
    chart_id: String,
    /// True if the chart is visible.
    visible: bool,
}
impl TimeChart {
    /// Constructor.
    fn new(html_id: String, uid: ChartUid, value: Value) -> Self {
        let chart_id = format!("chart_{}", uid);
        Self {
            html_id,
            uid,
            value,
            chart: None,
            visible: false,
            chart_id,
        }
    }

    /// Destroys a chart.
    pub fn destroy(self) {
        if let Some(chart) = self.chart {
            stdweb::js!(@(no_return)
                @{chart}.dispose()
            )
        }
    }

    /// The UID of this chart.
    pub fn uid(&self) -> &ChartUid {
        &self.uid
    }

    /// Default time chart constructor.
    pub fn default(html_id: String, uid: ChartUid) -> Self {
        Self::total_size(html_id, uid)
    }

    /// Total size over time chart constructor.
    pub fn total_size(html_id: String, uid: ChartUid) -> Self {
        Self::new(html_id, uid, Value::total_size())
    }

    /// Highest lifetime over time chart constructor.
    pub fn highest_lifetime(html_id: String, uid: ChartUid) -> Self {
        Self::new(html_id, uid, Value::highest_lifetime())
    }

    /// Updates itself for the most recent diff.
    ///
    /// Fails if there are no diffs in `data`.
    pub fn update(&mut self, data: &Storage) {
        if let Some(diff) = data.last_diff() {
            let points = self.value.points_of_diff(data, diff);
            js! {
                let chart = @{&self.chart};
                chart.addData(@{points})
            }
        } else {
            panic!("asked to update for the most recent diff, but there's no diff at all")
        }
    }

    /// Adds the origin point, if any.
    fn update_origin(&mut self, data: &Storage) {
        if let Some(point) = self.value.origin(data) {
            js! {
                @{&self.chart}.addData(@{point})
            }
        }
    }

    /// Updates itself with all the data from the whole history.
    ///
    /// This is only used when creating the actual graph, *i.e.* in `init`.
    fn update_history(&mut self, data: &Storage) {
        js! { @{&self.chart}.data = [] }
        self.update_origin(data);
        data.diff_iter(|diff| {
            let points = self.value.points_of_diff(data, diff);
            js! {
                @{&self.chart}.addData(@{points})
            }
        })
    }

    /// Creates a collapse button for this chart.
    fn collapse_button(&self) -> Html {
        let uid = self.uid;
        html! {
            <img
                class="collapse_button"
                onclick=|_| ChartsMsg::collapse(uid)
            />
        }
    }
    /// Creates an expand button for this chart.
    fn expand_button(&self) -> Html {
        let uid = self.uid;
        html! {
            <img
                class="expand_button"
                onclick=|_| ChartsMsg::expand(uid)
            />
        }
    }

    /// Renders itself.
    pub fn render(&self) -> Html {
        let expand_or_collapse_button = if self.visible {
            self.collapse_button()
        } else {
            self.expand_button()
        };
        let uid = self.uid;
        html! {
            <g>
            <center>
                {expand_or_collapse_button}
                <img
                    class="move_up_button"
                    onclick=|_| ChartsMsg::move_up(uid)
                />
                <img
                    class="move_down_button"
                    onclick=|_| ChartsMsg::move_down(uid)
                />
                <img
                    class="close_button"
                    onclick=|_| ChartsMsg::close(uid)
                />
                <h2> { format!("{} over time", self.value.desc()) } </h2>
            </center>
                <div id={&self.html_id}
                    class={if self.visible { "chart_style" } else { "hidden_chart_style" }}
                />
            </g>
        }
    }

    /// Disposes of the underlying chart, if any, and returns its data, if any.
    ///
    /// Returns an empty list if there was no chart.
    ///
    /// This is used by `Charts::refresh`: all charts are deactivated by this function, and
    /// then all charts are refreshed using `Self::chart_dispose`.
    pub fn chart_dispose(&mut self) -> JsVal {
        if let Some(chart) = std::mem::replace(&mut self.chart, None) {
            js!(
                let data = @{&chart}.data;
                @{&chart}.data = [];
                @{chart}.dispose();
                return data
            )
        } else {
            js!(return [])
        }
    }

    /// Creates a new chart binded to this chart's HTML identifier.
    ///
    /// Does not do anything if it already has a chart.
    ///
    /// This is used by `Charts::refresh`: all charts are deactivated by `Self::chart_dispose`, and
    /// then all charts are refreshed using this function.
    pub fn refresh_target(&mut self, data: JsVal) {
        if self.chart.is_none() {
            let new_chart = Self::new_chart(&self.html_id, &self.chart_id);
            js! {
                @{&new_chart}.data = @{data};
            }
            self.chart = Some(new_chart)
        }
    }

    /// Creates an amchart for this chart.
    fn new_chart(html_id: &str, chart_id: &str) -> JsVal {
        js!(
            am4core.useTheme(am4themes_animated);
            var chart = am4core.create(@{html_id}, am4charts.XYChart);

            chart.data = [];
            chart.id = @{chart_id};

            // chart.padding(0, 0, 0, 0);

            var dateAxis = chart.xAxes.push(new am4charts.DateAxis());
            dateAxis.dateFormats.setKey("second", "ss");
            dateAxis.dateFormats.setKey("millisecond", "nnn");
            dateAxis.periodChangeDateFormats.setKey("second", "[bold]h:mm a");
            dateAxis.periodChangeDateFormats.setKey("minute", "[bold]h:mm a");
            dateAxis.periodChangeDateFormats.setKey("hour", "[bold]h:mm a");
            dateAxis.extraMax = 0.2;

            var valueAxis = chart.yAxes.push(new am4charts.ValueAxis());

            var series = chart.series.push(new am4charts.LineSeries());
            series.dataFields.dateX = "x";
            series.dataFields.valueY = "y";
            series.tooltipText = "{y}";
            series.interpolationDuration = 10;
            series.defaultState.transitionDuration = 0;
            series.strokeWidth = 2;
            series.minBulletDistance = 15;

            // Create vertical scrollbar and place it before the value axis
            // chart.scrollbarY = new am4core.Scrollbar();
            // chart.scrollbarY.parent = chart.leftAxesContainer;
            // chart.scrollbarY.toBack();

            // Create a horizontal scrollbar with previe and place it underneath the date axis
            chart.scrollbarX = new am4charts.XYChartScrollbar();
            chart.scrollbarX.series.push(series);
            chart.scrollbarX.parent = chart.bottomAxesContainer;

            // bullet at the front of the line
            var bullet = series.createChild(am4charts.CircleBullet);
            bullet.circle.radius = 5;
            bullet.fillOpacity = 1;
            bullet.fill = chart.colors.getIndex(0);
            bullet.isMeasured = false;

            series.events.on("validated", function() {
                if (series.dataItems.last !== undefined) {
                    bullet.moveTo(series.dataItems.last.point);
                    bullet.validatePosition()
                }
            });

            return chart;
        )
    }

    /// Initializes itself.
    pub fn init(&mut self, data: &Storage) {
        if self.chart.is_none() {
            let chart = Self::new_chart(&self.html_id, &self.chart_id);
            self.chart = Some(chart);
        }
        self.update_history(data);
        self.visible = true
    }
}

/// # Cosmetic stuff.
impl TimeChart {
    /// Collapses the chart and changes the collapse button to an expand button.
    pub fn collapse(&mut self) -> ShouldRender {
        if self.visible {
            self.visible = false;
            true
        } else {
            warn!(
                "asked to collapse chart #{}, but it is already collapsed",
                self.uid
            );
            false
        }
    }
    /// Expands the chart and changes the expand button to a collapse button.
    pub fn expand(&mut self) -> ShouldRender {
        if !self.visible {
            self.visible = true;
            true
        } else {
            warn!(
                "asked to expand chart #{}, but it is already expanded",
                self.uid
            );
            false
        }
    }
}
