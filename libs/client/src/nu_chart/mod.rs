//! Charts.

use crate::base::*;

use point::JsExt;

// pub mod time;

new_uid! {
    mod chart_uid {
        uid: ChartUid,
        set: ChartUidSet,
        map: ChartUidMap,
    }
}

pub struct Chart {
    /// HTML identifier of the chart.
    html_id: String,
    /// Actual chart.
    chart: Option<JsVal>,
    /// True if the chart is visible.
    visible: bool,
    /// Index of the chart.
    index: index::Chart,
}

impl Chart {
    /// Sets the index to the previous index of the current one.
    ///
    /// Used when removing a chart. Panics if the current index is zero.
    pub fn prev_index(&mut self) {
        if let Some(index) = self.index.prev() {
            self.index = index
        } else {
            panic!("index #{} has no previous index", self.index)
        }
    }

    /// Forces the index of a chart.
    pub fn set_index(&mut self, index: index::Chart) {
        self.index = index
    }

    /// Title of the chart.
    pub fn title(&self) -> String {
        "<title placeholder>".into()
    }

    /// Initializes itself and becomes visible if it wasn't.
    pub fn init(&mut self) {
        if self.chart.is_none() {
            let chart = new_chart(&self.html_id);
            self.chart = Some(chart);
        }
        self.visible = true
    }
}

/// # Message handling
impl Chart {
    /// Handles a message from the server.
    pub fn update(&mut self, msg: msg::from_server::ChartMsg) {
        use charts::{point::Points::*, time::TimePoints::Size};
        use msg::from_server::ChartMsg::*;

        match msg {
            Points(Time(Size(points))) => self.add_points(points),
            NewPoints(Time(Size(points))) => self.overwrite_points(points),
        }
    }

    /// Appends some points to the chart.
    pub fn add_points<Key, Val>(&self, points: Vec<Point<Key, Val>>)
    where
        Key: JsExt,
        Val: JsExt,
    {
        js!(@(no_return)
            @{&self.chart}.addData(@{points.to_js()});
        )
    }

    /// Overwrites the points in a chart.
    pub fn overwrite_points<Key, Val>(&self, points: Vec<Point<Key, Val>>)
    where
        Key: JsExt,
        Val: JsExt,
    {
        js!(@(no_return)
            let chart = @{&self.chart};
            chart.data = @{points.to_js()};
            chart.invalidateRawData();
        )
    }
}

/// # Rendering
impl Chart {
    /// Renders itself.
    pub fn render(&self) -> Html {
        use msg::ChartsMsg;

        let expand_or_collapse_button = if self.visible {
            self.collapse_button()
        } else {
            self.expand_button()
        };
        let index = self.index;
        html! {
            <g>
            <center class=style::class::chart::HEADER>
                { expand_or_collapse_button }
                { buttons::move_up(move |_| ChartsMsg::nu_move_up(index)) }
                { buttons::move_down(move |_| ChartsMsg::nu_move_down(index)) }
                { buttons::close(move |_| ChartsMsg::nu_close(index)) }

                <h2> { self.title() } </h2>
            </center>
                <div
                    id=&self.html_id
                    class=style::class::chart::style(self.visible)
                />
            </g>
        }
    }

    /// Creates a collapse button for this chart.
    fn collapse_button(&self) -> Html {
        let index = self.index;
        buttons::collapse(move |_| msg::ChartsMsg::nu_collapse(index))
    }
    /// Creates an expand button for this chart.
    fn expand_button(&self) -> Html {
        let index = self.index;
        buttons::expand(move |_| msg::ChartsMsg::nu_expand(index))
    }
}

/// # Cosmetic stuff
impl Chart {
    /// Collapses the chart and changes the collapse button to an expand button.
    pub fn collapse(&mut self) -> ShouldRender {
        if self.visible {
            self.visible = false;
            true
        } else {
            warn!(
                "asked to collapse chart #{}, but it is already collapsed",
                self.index
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
                self.index
            );
            false
        }
    }
}

/// Creates an amchart for.
fn new_chart(html_id: &str) -> JsVal {
    js!(
        am4core.useTheme(am4themes_animated);
        var chart = am4core.create(@{html_id}, am4charts.XYChart);

        chart.data = [];

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
        series.interpolationDuration = 0;
        series.defaultState.transitionDuration = 0;
        series.strokeWidth = 2;
        series.minBulletDistance = 15;
        series.title = "fuck";

        var series = chart.series.push(new am4charts.LineSeries());
        series.dataFields.dateX = "x";
        series.dataFields.valueY = "y";
        series.tooltipText = "{y}";
        series.interpolationDuration = 10;
        series.defaultState.transitionDuration = 0;
        series.strokeWidth = 2;
        series.minBulletDistance = 15;
        series.title = "ass";

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
