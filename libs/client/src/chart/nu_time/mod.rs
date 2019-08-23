//! Time charts.

use crate::base::*;

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
}
impl TimeChart {
    /// Constructor.
    fn new(html_id: String, uid: ChartUid, value: Value) -> Self {
        Self {
            html_id,
            uid,
            value,
            chart: None,
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
        use stdweb::js;
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
        use stdweb::js;
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
        use stdweb::js;
        self.update_origin(data);
        data.diff_iter(|diff| {
            let points = self.value.points_of_diff(data, diff);
            js! {
                @{&self.chart}.addData(@{points})
            }
        })
    }

    /// Renders itself.
    pub fn render(&self) -> Html {
        html! {
            <center>
                <h2> { format!("{} over time", self.value.desc()) } </h2>
                <div id={&self.html_id}
                    style="width: 100%; height: 500px;"
                />
            </center>
        }
    }

    /// Initializes itself.
    pub fn init(&mut self, data: &Storage) {
        use stdweb::js;
        let chart = js! {
            am4core.useTheme(am4themes_animated);
            var chart = am4core.create(@{&self.html_id}, am4charts.XYChart);

            chart.data = [];

            chart.padding(0, 0, 0, 0);

            var dateAxis = chart.xAxes.push(new am4charts.DateAxis());
            dateAxis.dateFormats.setKey("second", "ss");
            dateAxis.dateFormats.setKey("millisecond", "nnn");
            dateAxis.periodChangeDateFormats.setKey("second", "[bold]h:mm a");
            dateAxis.periodChangeDateFormats.setKey("minute", "[bold]h:mm a");
            dateAxis.periodChangeDateFormats.setKey("hour", "[bold]h:mm a");

            var valueAxis = chart.yAxes.push(new am4charts.ValueAxis());

            var series = chart.series.push(new am4charts.LineSeries());
            series.dataFields.dateX = "x";
            series.dataFields.valueY = "y";
            series.tooltipText = "{y}";
            series.interpolationDuration = 100;
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
        };
        self.chart = Some(chart);
        self.update_history(data)
    }
}
