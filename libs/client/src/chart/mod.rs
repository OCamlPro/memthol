//! Charts.

pub use charts::chart::ChartSpec;

use crate::base::*;

pub use axis::{XAxis, YAxis};

pub mod axis;

/// The collection of charts.
pub struct Charts {
    /// The actual collection of charts.
    charts: Vec<Chart>,
    /// Callback to send messages to the model.
    to_model: Callback<Msg>,
}

impl Charts {
    /// Constructs an empty collection of charts.
    pub fn new(to_model: Callback<Msg>) -> Self {
        Self {
            charts: vec![],
            to_model,
        }
    }

    /// Sends a message to the model.
    pub fn send(&self, msg: Msg) {
        self.to_model.emit(msg)
    }

    /// Retrieves the chart corresponding to some UID.
    fn get_mut(&mut self, uid: charts::uid::ChartUid) -> Res<&mut Chart> {
        debug_assert_eq!(
            self.charts
                .iter()
                .filter(|chart| chart.uid() == uid)
                .count(),
            1
        );
        for chart in &mut self.charts {
            if chart.uid() == uid {
                return Ok(chart);
            }
        }
        bail!("unknown chart UID #{}", uid)
    }

    /// Applies an operation.
    pub fn update(&mut self, action: msg::ChartsMsg) -> Res<ShouldRender> {
        let should_render = match action {
            msg::ChartsMsg::Build(uid) => {
                let chart = self
                    .get_mut(uid)
                    .chain_err(|| format!("while building and binding chart #{}", uid))?;
                chart.build_chart()?;
                true
            }
        };

        Ok(should_render)
    }

    /// Alies an operation from the server.
    pub fn server_update(&mut self, action: msg::from_server::ChartsMsg) -> Res<ShouldRender> {
        use msg::from_server::ChartsMsg;
        let should_render = match action {
            ChartsMsg::NewChart(spec) => {
                let uid = spec.uid();
                let chart = Chart::new(spec);
                self.charts.push(chart);
                self.send(msg::ChartsMsg::build(uid));
                true
            }

            ChartsMsg::NewPoints(mut points) => {
                for chart in &mut self.charts {
                    if let Some(points) = points.remove(&chart.uid()) {
                        chart.overwrite_points(points)
                    }
                }
                false
            }
            ChartsMsg::AddPoints(mut points) => {
                for chart in &mut self.charts {
                    if let Some(points) = points.remove(&chart.uid()) {
                        chart.add_points(points)
                    }
                }
                false
            }

            msg => bail!(
                "unsupported message from server: {}",
                msg.as_json().unwrap_or_else(|_| format!("{:?}", msg))
            ),
        };
        Ok(should_render)
    }

    /// Renders the charts.
    pub fn render(&self) -> Html {
        html! {
            <g class=style::class::chart::CONTAINER>
                { for self.charts.iter().map(Chart::render) }
            </g>
        }
    }
}

/// A chart.
pub struct Chart {
    /// Chart specification.
    spec: ChartSpec,
    /// True if the chart is expanded.
    visible: bool,
    /// DOM element containing the chart.
    container: String,
    /// Actual chart as a JS value.
    chart: Option<JsVal>,
    /// Points from the server that have not been treated yet.
    ///
    /// The boolean is true when the points should overwrite the existing points.
    points: Vec<(point::Points, bool)>,
}
impl Chart {
    /// Constructor.
    pub fn new(spec: ChartSpec) -> Self {
        let container = style::class::chart::class(spec.uid());
        Self {
            spec,
            visible: false,
            container,
            chart: None,
            points: vec![],
        }
    }

    /// UID accessor.
    pub fn uid(&self) -> charts::uid::ChartUid {
        self.spec.uid()
    }

    /// Builds the actual JS chart and attaches it to the right container.
    ///
    /// Also, makes the chart visible.
    pub fn build_chart(&mut self) -> Res<()> {
        use axis::AxisExt;

        if self.chart.is_some() {
            bail!("asked to build and bind a chart that's already built and binded")
        }

        let chart = js!(
            am4core.useTheme(am4themes_animated);
            var chart = am4core.create(@{&self.container}, am4charts.XYChart);
            chart.data = [];
            return chart
        );

        self.spec.x_axis().chart_apply(&chart);
        self.spec.y_axis().chart_apply(&chart);

        // Default series, for allocations not caught by any filter.
        let default_series = js!(
            var series = @{&chart}.series.push(new am4charts.LineSeries());
            series.interpolationDuration = 100;
            series.defaultState.transitionDuration = 0;
            series.strokeWidth = 2;
            series.minBulletDistance = 15;
            series.title = "rest";
            return series;
        );

        self.spec.x_axis().series_apply(&default_series, None);
        self.spec.y_axis().series_apply(&default_series, None);

        js!(@(no_return)
            let chart = @{&chart};

            // Cosmetic stuff.

            // X-axis scrollbar.
            chart.scrollbarX = new am4charts.XYChartScrollbar();
            chart.scrollbarX.series.push(@{&default_series});
            chart.scrollbarX.parent = chart.bottomAxesContainer;

            // Progression bullet at the end of the line.
            var bullet = @{&default_series}.createChild(am4charts.CircleBullet);
            bullet.circle.radius = 5;
            bullet.fillOpacity = 1;
            bullet.fill = chart.colors.getIndex(0);
            bullet.isMeasured = false;
        );

        for (points, overwrite) in self.points.drain(0..) {
            if overwrite {
                Self::really_overwrite_points(&chart, points)
            } else {
                Self::really_add_points(&chart, points)
            }
        }

        self.chart = Some(chart);
        self.visible = true;

        Ok(())
    }

    /// Appends some points to the chart.
    pub fn add_points(&mut self, points: point::Points) {
        if let Some(chart) = self.chart.as_ref() {
            Self::really_add_points(chart, points)
        } else {
            self.points.push((points, false))
        }
    }

    /// Appends some points to the chart.
    fn really_add_points(chart: &JsVal, points: point::Points) {
        match points {
            point::Points::Time(points) => match points {
                charts::point::TimePoints::Size(points) => Self::inner_add_points(chart, points),
            },
        }
    }

    /// Appends some points to the chart.
    fn inner_add_points<Key, Val>(chart: &JsVal, points: Vec<point::Point<Key, Val>>)
    where
        Key: JsExt + fmt::Display,
        Val: JsExt + fmt::Display,
    {
        js!(@(no_return)
            @{chart}.addData(@{points.as_js()});
        )
    }

    /// Overwrites the points in a chart.
    pub fn overwrite_points(&mut self, points: point::Points) {
        if let Some(chart) = self.chart.as_ref() {
            Self::really_overwrite_points(chart, points)
        } else {
            self.points.push((points, true))
        }
    }

    /// Overwrites the points in a chart.
    fn really_overwrite_points(chart: &JsVal, points: point::Points) {
        match points {
            point::Points::Time(points) => match points {
                charts::point::TimePoints::Size(points) => {
                    Self::inner_overwrite_points(chart, points)
                }
            },
        }
    }

    /// Overwrites the points in a chart.
    fn inner_overwrite_points<Key, Val>(chart: &JsVal, points: Vec<Point<Key, Val>>)
    where
        Key: JsExt + fmt::Display,
        Val: JsExt + fmt::Display,
    {
        js!(@(no_return)
            let chart = @{chart};
            chart.data = @{points.as_js()};
            chart.invalidateRawData();
        )
    }

    /// Creates a collapse button for this chart.
    fn collapse_button(&self) -> Html {
        let uid = self.uid();
        buttons::collapse(move |_| format!("collapse #{}", uid).into())
    }
    /// Creates an expand button for this chart.
    fn expand_button(&self) -> Html {
        let uid = self.uid();
        buttons::expand(move |_| format!("expand #{}", uid).into())
    }

    /// Renders the chart.
    pub fn render(&self) -> Html {
        let expand_or_collapse_button = if self.visible {
            self.collapse_button()
        } else {
            self.expand_button()
        };

        let uid = self.uid();

        html! {
            <g>
                <center class=style::class::chart::HEADER>
                    { expand_or_collapse_button }
                    { buttons::move_up(move |_| format!("move_up {}", uid).into()) }
                    { buttons::move_down(move |_| format!("move_down {}", uid).into()) }
                    { buttons::close(move |_| format!("close {}", uid).into()) }

                    <h2> { self.spec.desc() } </h2>
                </center>
                <div id={&self.container}
                    class=style::class::chart::style(self.visible)
                />
            </g>
        }
    }
}
