//! Charts.

pub use charts::chart::ChartSpec;

use crate::common::*;

pub mod axis;
pub mod new;

// pub use axis::{XAxis, YAxis};
pub use charts::chart::ChartUid;

// /// The collection of charts.
// pub struct Charts {
//     /// The actual collection of charts.
//     charts: Vec<Chart>,
//     /// Callback to send messages to the model.
//     to_model: Callback<Msg>,
//     /// Chart constructor element.
//     new_chart: new::NewChart,
// }

// impl Charts {
//     /// Constructs an empty collection of charts.
//     pub fn new(to_model: Callback<Msg>) -> Self {
//         Self {
//             charts: vec![],
//             to_model,
//             new_chart: new::NewChart::new(),
//         }
//     }

//     /// Sends a message to the model.
//     pub fn send(&self, msg: Msg) {
//         self.to_model.emit(msg)
//     }

//     /// Retrieves the chart corresponding to some UID.
//     fn get_mut(&mut self, uid: ChartUid) -> Res<(usize, &mut Chart)> {
//         debug_assert_eq!(
//             self.charts
//                 .iter()
//                 .filter(|chart| chart.uid() == uid)
//                 .count(),
//             1
//         );
//         for (index, chart) in self.charts.iter_mut().enumerate() {
//             if chart.uid() == uid {
//                 return Ok((index, chart));
//             }
//         }
//         bail!("unknown chart UID #{}", uid)
//     }
// }

// /// # Internal message handling
// impl Charts {
//     /// Applies an operation.
//     pub fn update(
//         &mut self,
//         filters: &filter::Filters,
//         action: msg::ChartsMsg,
//     ) -> Res<ShouldRender> {
//         use msg::ChartsMsg::*;
//         match action {
//             Build(uid) => self.build(uid, filters),
//             Move { uid, up } => self.move_chart(uid, up),
//             ToggleVisible(uid) => self.toggle_visible(uid),
//             Destroy(uid) => self.destroy(uid),

//             RefreshFilters => self.refresh_filters(filters),

//             NewChartSetX(x_axis) => self.new_chart.set_x_axis(x_axis),
//             NewChartSetY(y_axis) => self.new_chart.set_y_axis(y_axis),
//         }
//     }

//     /// Refreshes all filters in all charts.
//     fn refresh_filters(&mut self, filters: &filter::Filters) -> Res<ShouldRender> {
//         for chart in &self.charts {
//             chart.replace_filters(filters)?
//         }

//         // Rendering is done at JS-level, no need to render the HTML.
//         Ok(false)
//     }

//     /// Move a chart, up if `up`, down otherwise.
//     fn move_chart(&mut self, uid: ChartUid, up: bool) -> Res<ShouldRender> {
//         let mut index = None;
//         for (idx, chart) in self.charts.iter().enumerate() {
//             if chart.uid() == uid {
//                 index = Some(idx)
//             }
//         }

//         let index = if let Some(index) = index {
//             index
//         } else {
//             bail!("cannot move chart with unknown UID #{}", uid)
//         };

//         let other_index = if up {
//             // Move up.
//             if index > 0 {
//                 index - 1
//             } else {
//                 return Ok(false);
//             }
//         } else {
//             let other_index = index + 1;
//             // Move down.
//             if other_index < self.charts.len() {
//                 other_index
//             } else {
//                 return Ok(false);
//             }
//         };

//         self.charts.swap(index, other_index);

//         Ok(true)
//     }

//     /// Forces a chart to build its actual (JS) graph and bind it to its container.
//     fn build(&mut self, uid: ChartUid, filters: &filter::Filters) -> Res<ShouldRender> {
//         let (_, chart) = self
//             .get_mut(uid)
//             .chain_err(|| format!("while building and binding chart #{}", uid))?;
//         chart.build_chart(filters)?;
//         Ok(true)
//     }

//     /// Toggles the visibility of a chart.
//     fn toggle_visible(&mut self, uid: ChartUid) -> Res<ShouldRender> {
//         let (_, chart) = self
//             .get_mut(uid)
//             .chain_err(|| format!("while changing chart visibility"))?;
//         chart.toggle_visible();
//         Ok(true)
//     }

//     /// Destroys a chart.
//     fn destroy(&mut self, uid: ChartUid) -> Res<ShouldRender> {
//         let (index, _) = self
//             .get_mut(uid)
//             .chain_err(|| format!("while destroying chart"))?;
//         let chart = self.charts.remove(index);
//         chart.destroy();
//         Ok(true)
//     }
// }

// /// # Rendering
// impl Charts {
//     /// Renders the charts.
//     pub fn render(&self) -> Html {
//         html! {
//             <g class=style::class::chart::CONTAINER>
//                 { for self.charts.iter().map(Chart::render) }
//                 { self.new_chart.render() }
//             </g>
//         }
//     }
// }

// /// # Server message handling.
// impl Charts {
//     /// Alies an operation from the server.
//     pub fn server_update(
//         &mut self,
//         filters: &filter::Filters,
//         action: msg::from_server::ChartsMsg,
//     ) -> Res<ShouldRender> {
//         use msg::from_server::{ChartMsg, ChartsMsg};
//         let should_render = match action {
//             ChartsMsg::NewChart(spec) => {
//                 info!("received a chart-creation message from the server");
//                 let uid = spec.uid();
//                 let chart = Chart::new(spec);
//                 self.charts.push(chart);
//                 self.send(msg::ChartsMsg::build(uid));
//                 true
//             }

//             ChartsMsg::NewPoints {
//                 mut points,
//                 refresh_filters,
//             } => {
//                 info!("received a overwrite-points message from the server");
//                 for chart in &mut self.charts {
//                     if let Some(points) = points.remove(&chart.uid()) {
//                         chart.overwrite_points(points)
//                     }
//                 }
//                 if refresh_filters {
//                     self.refresh_filters(filters)?;
//                 }
//                 false
//             }
//             ChartsMsg::AddPoints(mut points) => {
//                 info!("received a add-points message from the server");
//                 for chart in &mut self.charts {
//                     if let Some(points) = points.remove(&chart.uid()) {
//                         chart.add_points(points)
//                     }
//                 }
//                 false
//             }

//             ChartsMsg::Chart { uid, msg } => {
//                 info!("received a message specific to chart #{} from server", uid);
//                 let (_index, chart) = self.get_mut(uid)?;
//                 match msg {
//                     ChartMsg::NewPoints(points) => chart.overwrite_points(points),
//                     ChartMsg::Points(points) => chart.add_points(points),
//                 }
//                 true
//             } // msg => bail!(
//               //     "unsupported message from server: {}",
//               //     msg.as_json().unwrap_or_else(|_| format!("{:?}", msg))
//               // ),
//         };
//         Ok(should_render)
//     }
// }

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
    pub fn uid(&self) -> ChartUid {
        self.spec.uid()
    }

    /// Toggles the visibility of the chart.
    pub fn toggle_visible(&mut self) {
        self.visible = !self.visible
    }
}

/// # Functions for message-handling.
impl Chart {
    /// Destroys the chart.
    pub fn destroy(self) {
        if let Some(chart) = self.chart {
            js!(@(no_return)
                @{chart}.dispose();
            )
        }
    }

    /// Adds/remove a legend to/from the chart.
    fn toggle_legend(chart: &JsVal, on: bool) {
        if on {
            js!(@(no_return)
                var chart = @{chart};
                if (chart.legend === undefined) {
                    chart.legend = new am4charts.Legend();
                    chart.legend.labels.template.text = "[bold {color}]{name}[/]";
                }
            )
        } else {
            js!(@(no_return)
                var chart = @{chart};
                if (chart.legend !== undefined) {
                    chart.legend.dispose();
                    chart.legend = undefined
                }
            )
        }
    }

    // /// Replaces the filters of the chart.
    // pub fn replace_filters(&self, filters: &filter::Filters) -> Res<()> {
    //     let chart = if let Some(chart) = self.chart.as_ref() {
    //         chart
    //     } else {
    //         return Ok(());
    //     };

    //     // Remove the legend if there's no active filter, turn it on if there are some.
    //     Self::toggle_legend(chart, filters.len() > 0);

    //     // Remove all series from the chart and DISPOSE. Otherwise they'll be orphaned.
    //     js!(@(no_return)
    //         var chart = @{chart};
    //         // if (chart.legend !== undefined) {
    //         //     chart.legend.dispose();
    //         // }
    //         // chart.legend = undefined;
    //         while (chart.series.length > 0) {
    //             chart.series.pop().dispose()
    //         }
    //     );
    //     filters.specs_apply(|filter| {
    //         use crate::filter::FilterSpecExt;
    //         filter.add_series_to(&self.spec, chart);
    //         Ok(())
    //     })?;
    //     Ok(())
    // }

    // /// Builds the actual JS chart and attaches it to the right container.
    // ///
    // /// Also, makes the chart visible.
    // pub fn build_chart(&mut self, filters: &filter::Filters) -> Res<()> {
    //     use axis::AxisExt;

    //     if self.chart.is_some() {
    //         bail!("asked to build and bind a chart that's already built and binded")
    //     }

    //     let chart = js!(
    //         am4core.useTheme(am4themes_animated);
    //         var chart = am4core.create(@{&self.container}, am4charts.XYChart);
    //         chart.data = [];
    //         // Cosmetic stuff.
    //         chart.scrollbarX = new am4charts.XYChartScrollbar();
    //         chart.scrollbarX.parent = chart.bottomAxesContainer;
    //         chart.cursor = new am4charts.XYCursor();

    //         return chart
    //     );

    //     self.spec.x_axis().chart_apply(&chart);
    //     self.spec.y_axis().chart_apply(&chart);

    //     for (points, overwrite) in self.points.drain(0..) {
    //         if overwrite {
    //             Self::really_overwrite_points(&chart, points)
    //         } else {
    //             Self::really_add_points(&chart, points)
    //         }
    //     }

    //     self.chart = Some(chart);
    //     self.visible = true;

    //     self.replace_filters(filters)
    //         .chain_err(|| format!("while build chart #{}", self.uid()))?;

    //     Ok(())
    // }

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
}

// /// # Rendering
// impl Chart {
//     /// Renders the chart.
//     pub fn render(&self) -> Html {
//         let uid = self.uid();
//         html! {
//             <g>
//                 <center class=style::class::chart::HEADER>
//                     { self.expand_or_collapse_button() }
//                     { Button::move_up(
//                         "Move the chart up",
//                         move |_| msg::ChartsMsg::move_up(uid)
//                     ) }
//                     { Button::move_down(
//                         "Move the chart down",
//                         move |_| msg::ChartsMsg::move_down(uid)
//                     ) }
//                     { Button::close(
//                         "Close the chart",
//                         move |_| msg::ChartsMsg::destroy(uid)
//                     ) }

//                     <h2> { self.spec.desc() } </h2>
//                 </center>
//                 <div id={&self.container}
//                     class=style::class::chart::style(self.visible)
//                 />
//             </g>
//         }
//     }

//     /// Creates a collapse or expand button depending on whether the chart is visible.
//     fn expand_or_collapse_button(&self) -> Html {
//         let uid = self.uid();
//         if self.visible {
//             Button::collapse("Collapse the chart", move |_| {
//                 msg::ChartsMsg::toggle_visible(uid)
//             })
//         } else {
//             Button::expand("Expand the chart", move |_| {
//                 msg::ChartsMsg::toggle_visible(uid)
//             })
//         }
//     }
// }
