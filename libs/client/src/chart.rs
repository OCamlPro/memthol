//! Charts.

use plotters::prelude::*;

pub use charts::chart::ChartSpec;

use crate::common::*;

pub mod axis;
pub mod new;

// pub use axis::{XAxis, YAxis};
pub use charts::chart::ChartUid;

/// The collection of charts.
pub struct Charts {
    /// The actual collection of charts.
    charts: Vec<Chart>,
    /// Callback to send messages to the model.
    to_model: Callback<Msg>,
    /// Chart constructor element.
    new_chart: new::NewChart,
}

impl Charts {
    /// Constructs an empty collection of charts.
    pub fn new(to_model: Callback<Msg>) -> Self {
        Self {
            charts: vec![],
            to_model,
            new_chart: new::NewChart::new(),
        }
    }

    pub fn len(&self) -> usize {
        self.charts.len()
    }

    /// Sends a message to the model.
    pub fn send(&self, msg: Msg) {
        self.to_model.emit(msg)
    }

    /// Retrieves the chart corresponding to some UID.
    fn get_mut(&mut self, uid: ChartUid) -> Res<(usize, &mut Chart)> {
        debug_assert_eq!(
            self.charts
                .iter()
                .filter(|chart| chart.uid() == uid)
                .count(),
            1
        );
        for (index, chart) in self.charts.iter_mut().enumerate() {
            if chart.uid() == uid {
                return Ok((index, chart));
            }
        }
        bail!("unknown chart UID #{}", uid)
    }

    /// Destroys a chart.
    fn destroy(&mut self, uid: ChartUid) -> Res<ShouldRender> {
        let (index, _) = self
            .get_mut(uid)
            .chain_err(|| format!("while destroying chart"))?;
        let chart = self.charts.remove(index);
        chart.destroy();
        Ok(true)
    }
}

/// # Internal message handling
impl Charts {
    /// Applies an operation.
    pub fn update(
        &mut self,
        filters: &filter::Filters,
        action: msg::ChartsMsg,
    ) -> Res<ShouldRender> {
        use msg::ChartsMsg::*;
        match action {
            Build(uid) => self.build(uid),
            Move { uid, up } => self.move_chart(uid, up),
            ToggleVisible(uid) => self.toggle_visible(uid),
            Destroy(uid) => self.destroy(uid),

            RefreshFilters => self.refresh_filters(filters),

            NewChartSetX(x_axis) => self.new_chart.set_x_axis(x_axis),
            NewChartSetY(y_axis) => self.new_chart.set_y_axis(y_axis),
        }
    }

    pub fn mounted(&mut self) -> ShouldRender {
        let mut res = false;
        for chart in &mut self.charts {
            let should_render = chart.mounted();
            res = res || should_render
        }
        res
    }

    /// Refreshes all filters in all charts.
    fn refresh_filters(&mut self, filters: &filter::Filters) -> Res<ShouldRender> {
        for chart in &mut self.charts {
            chart.replace_filters(filters)?
        }

        // Rendering is done at JS-level, no need to render the HTML.
        Ok(false)
    }

    /// Move a chart, up if `up`, down otherwise.
    fn move_chart(&mut self, uid: ChartUid, up: bool) -> Res<ShouldRender> {
        let mut index = None;
        for (idx, chart) in self.charts.iter().enumerate() {
            if chart.uid() == uid {
                index = Some(idx)
            }
        }

        let index = if let Some(index) = index {
            index
        } else {
            bail!("cannot move chart with unknown UID #{}", uid)
        };

        let other_index = if up {
            // Move up.
            if index > 0 {
                index - 1
            } else {
                return Ok(false);
            }
        } else {
            let other_index = index + 1;
            // Move down.
            if other_index < self.charts.len() {
                other_index
            } else {
                return Ok(false);
            }
        };

        self.charts.swap(index, other_index);

        Ok(true)
    }

    /// Forces a chart to build and bind itself to its container.
    fn build(&mut self, uid: ChartUid) -> Res<ShouldRender> {
        let (_, chart) = self
            .get_mut(uid)
            .chain_err(|| format!("while building and binding chart #{}", uid))?;
        chart.build_chart()?;
        Ok(true)
    }

    /// Toggles the visibility of a chart.
    fn toggle_visible(&mut self, uid: ChartUid) -> Res<ShouldRender> {
        let (_, chart) = self
            .get_mut(uid)
            .chain_err(|| format!("while changing chart visibility"))?;
        chart.toggle_visible();
        Ok(true)
    }
}

/// # Rendering
impl Charts {
    /// Renders the charts.
    pub fn render(&self, model: &Model) -> Html {
        let res = html! {
            <g class=style::class::chart::CONTAINER>
                { for self.charts.iter().map(|chart| chart.render(model)) }
                { self.new_chart.render(model) }
            </g>
        };
        res
    }
}

/// # Server message handling.
impl Charts {
    /// Applies an operation from the server.
    pub fn server_update(
        &mut self,
        filters: &filter::Filters,
        action: msg::from_server::ChartsMsg,
    ) -> Res<ShouldRender> {
        use msg::from_server::{ChartMsg, ChartsMsg};

        let should_render = match action {
            ChartsMsg::NewChart(spec) => {
                debug!("received a chart-creation message from the server");
                let uid = spec.uid();
                let chart = Chart::new(spec, filters)?;
                self.charts.push(chart);
                self.send(msg::ChartsMsg::build(uid));
                true
            }

            ChartsMsg::NewPoints {
                mut points,
                refresh_filters,
            } => {
                debug!("received an overwrite-points message from the server");
                for chart in &mut self.charts {
                    if let Some(points) = points.remove(&chart.uid()) {
                        chart.overwrite_points(points)?
                    }
                }
                if refresh_filters {
                    self.refresh_filters(filters)?;
                }
                false
            }
            ChartsMsg::AddPoints(mut points) => {
                debug!("received an add-points message from the server");
                for chart in &mut self.charts {
                    if let Some(points) = points.remove(&chart.uid()) {
                        chart.add_points(points)?
                    }
                }
                false
            }

            ChartsMsg::Chart { uid, msg } => {
                debug!("received a message specific to chart #{} from server", uid);
                let (_index, chart) = self.get_mut(uid)?;
                match msg {
                    ChartMsg::NewPoints(points) => chart.overwrite_points(points)?,
                    ChartMsg::Points(points) => chart.add_points(points)?,
                }
                true
            }
            // msg => bail!(
            //     "unsupported message from server: {}",
            //     msg.as_json().unwrap_or_else(|_| format!("{:?}", msg))
            // ),
        };
        Ok(should_render)
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
    /// Actual DOM chart canvas.
    canvas: String,
    /// Chart drawing area.
    chart: Option<DrawingArea<CanvasBackend, plotters::coord::Shift>>,
    /// The points.
    points: Option<point::Points>,
    /// The filters, used to color the series and hide what the user asks to hide.
    filters: Map<charts::uid::LineUid, (charts::filter::FilterSpec, bool)>,
    /// Previous filter map, used when updating filters to keep track of those that are hidden.
    prev_filters: Map<charts::uid::LineUid, (charts::filter::FilterSpec, bool)>,
}
impl Chart {
    /// Constructor.
    pub fn new(spec: ChartSpec, all_filters: &filter::Filters) -> Res<Self> {
        let container = style::class::chart::class(spec.uid());
        let canvas = style::class::chart::canvas(spec.uid());

        let mut filters = Map::new();
        all_filters.specs_apply(|spec| {
            let prev = filters.insert(spec.uid(), (spec.clone(), true));
            debug_assert!(prev.is_none());
            Ok(())
        })?;

        Ok(Self {
            spec,
            visible: true,
            container,
            canvas,
            chart: None,
            points: None,
            filters,
            prev_filters: Map::new(),
        })
    }

    /// UID accessor.
    pub fn uid(&self) -> ChartUid {
        self.spec.uid()
    }

    /// Toggles the visibility of the chart.
    pub fn toggle_visible(&mut self) {
        self.visible = !self.visible
    }

    pub fn div_container(&self) -> &str {
        &self.container
    }
    pub fn canvas(&self) -> &str {
        &self.canvas
    }
}

/// # Functions for message-handling.
impl Chart {
    /// Destroys the chart.
    pub fn destroy(self) {}

    /// Replaces the filters of the chart.
    pub fn replace_filters(&mut self, filters: &filter::Filters) -> Res<()> {
        self.prev_filters.clear();
        std::mem::swap(&mut self.filters, &mut self.prev_filters);

        debug_assert!(self.filters.is_empty());

        filters.specs_apply(|spec| {
            let visible = self
                .prev_filters
                .remove(&spec.uid())
                .map(|(_spec, visible)| visible)
                .unwrap_or(true);
            let prev = self.filters.insert(spec.uid(), (spec.clone(), visible));
            if prev.is_some() {
                bail!(
                    "collision, found two filters with uid #{} while replacing filters",
                    spec.uid(),
                )
            }
            Ok(())
        })?;
        self.draw()
    }

    /// Builds the actual JS chart and attaches it to its container.
    ///
    /// Also, makes the chart visible.
    pub fn build_chart(&mut self) -> Res<()> {
        if self.chart.is_some() {
            bail!("asked to build and bind a chart that's already built and binded")
        }

        self.mounted();

        let backend: CanvasBackend =
            plotters::prelude::CanvasBackend::new(&self.canvas).expect("could not find canvas");

        let chart: DrawingArea<CanvasBackend, plotters::coord::Shift> = backend.into_drawing_area();
        chart.fill(&WHITE).unwrap();

        self.chart = Some(chart);

        Ok(())
    }

    /// Draws the chart.
    ///
    /// # TODO
    ///
    /// - this function contains code that's highly specific to the kind of points we are drawing.
    ///   It should be exported, probably in the `charts` crate, to keep this function focused on
    ///   what it does.
    pub fn draw(&mut self) -> Res<()> {
        self.mounted();
        let filters = &self.filters;
        if let Some(chart) = &mut self.chart {
            if let Some(points) = &self.points {
                match points {
                    charts::point::Points::Time(charts::point::TimePoints::Size(points)) => {
                        let (mut min_x, mut max_x, mut min_y, mut max_y) = (None, None, None, None);

                        for point in points.iter() {
                            if min_x.is_none() {
                                min_x = Some(&point.key)
                            }
                            max_x = Some(&point.key);

                            let (new_min_y, new_max_y) = point.vals.map.iter().fold(
                                (min_y, max_y),
                                |(mut min_y, mut max_y), (uid, val)| {
                                    let visible = filters
                                        .get(uid)
                                        .map(|(_spec, visible)| *visible)
                                        .unwrap_or(false);
                                    if visible {
                                        if let Some(min_y) = &mut min_y {
                                            if val < min_y {
                                                *min_y = *val
                                            }
                                        } else {
                                            min_y = Some(*val)
                                        }
                                        if let Some(max_y) = &mut max_y {
                                            if val > max_y {
                                                *max_y = *val
                                            }
                                        } else {
                                            max_y = Some(*val)
                                        }
                                        (min_y, max_y)
                                    } else {
                                        (min_y, max_y)
                                    }
                                },
                            );
                            min_y = new_min_y;
                            max_y = new_max_y;
                        }

                        let (min_x, max_x, min_y, max_y) = match (min_x, max_x, min_y, max_y) {
                            (Some(min_x), Some(max_x), Some(min_y), Some(max_y)) => {
                                (*min_x, *max_x, min_y as u32, max_y as u32)
                            }
                            (min_x, max_x, min_y, max_y) => {
                                warn!(
                                    "could not retrieve chart min/max values for chart #{} \
                                    ({:?}, {:?}, {:?}, {:?})",
                                    self.spec.uid(),
                                    min_x,
                                    max_x,
                                    min_y,
                                    max_y,
                                );
                                return Ok(());
                            }
                        };

                        chart.fill(&WHITE).unwrap();

                        let (width, _height) = chart.get_base_pixel();

                        let mut chart_cxt: ChartContext<
                            CanvasBackend,
                            RangedCoord<RangedDateTime<chrono::offset::Utc>, RangedCoordu32>,
                        > = ChartBuilder::on(&chart)
                            .margin(5 * width / 100)
                            .x_label_area_size(10)
                            .y_label_area_size(10)
                            .build_ranged(
                                RangedDateTime::from(std::ops::Range {
                                    start: min_x.date().clone(),
                                    end: max_x.date().clone(),
                                }),
                                min_y..max_y,
                            )
                            .unwrap();

                        chart_cxt
                            .configure_mesh()
                            .disable_x_mesh()
                            .disable_y_mesh()
                            .draw()
                            .unwrap();

                        for (uid, (spec, visible)) in filters {
                            if !visible {
                                continue;
                            }

                            let points = points.iter().filter_map(|point| {
                                point
                                    .vals
                                    .map
                                    .get(uid)
                                    .map(|val| (point.key.date().clone(), *val as u32))
                            });
                            let &charts::color::Color { r, g, b } = spec.color();
                            let color: palette::rgb::Rgb<palette::encoding::srgb::Srgb, _> =
                                palette::rgb::Rgb::new(r, g, b);
                            chart_cxt
                                .draw_series(LineSeries::new(points, color.stroke_width(5)))
                                .map_err(|e| e.to_string())?;
                        }

                        chart
                            .present()
                            .map_err(|e| format!("error while presenting chart: {}", e))?
                    }
                }
            }
        }
        Ok(())
    }

    /// Appends some points to the chart.
    pub fn add_points(&mut self, mut points: point::Points) -> Res<()> {
        if let Some(my_points) = &mut self.points {
            let changed = my_points.extend(&mut points)?;
            if changed {
                self.draw()?
            }
            Ok(())
        } else if !points.is_empty() {
            self.points = Some(points);
            self.draw()?;
            Ok(())
        } else {
            Ok(())
        }
    }

    /// Overwrites the points in a chart.
    pub fn overwrite_points(&mut self, points: point::Points) -> Res<()> {
        self.points = Some(points);
        self.draw()
    }
}

/// # Rendering
impl Chart {
    pub fn mounted(&mut self) -> ShouldRender {
        let document = web_sys::window()
            .expect("could not retrieve document window")
            .document()
            .expect("could not retrieve document from window");
        let canvas = document
            .get_element_by_id(&self.canvas)
            .expect("could not retrieve chart canvas");

        let (width, height) = (
            match canvas.client_width() {
                n if n >= 0 => n as u32,
                _ => {
                    alert!("An error occured while resizing a chart's canvas: negative width.");
                    panic!("fatal")
                }
            },
            match canvas.client_height() {
                n if n >= 0 => n as u32,
                _ => {
                    alert!("An error occured while resizing a chart's canvas: negative height.");
                    panic!("fatal")
                }
            },
        );

        use wasm_bindgen::JsCast;
        let html_canvas: web_sys::HtmlCanvasElement = canvas.clone().dyn_into().unwrap();

        if html_canvas.width() != width {
            html_canvas.set_width(width)
        }
        if html_canvas.height() != height {
            html_canvas.set_height(height)
        }

        false
    }
    /// Renders the chart.
    pub fn render(&self, model: &Model) -> Html {
        let uid = self.uid();
        html! {
            <g>
                <center class=style::class::chart::HEADER>
                    { self.expand_or_collapse_button(model) }
                    { buttons::move_up(
                        model,
                        "Move the chart up",
                        move |_| msg::ChartsMsg::move_up(uid)
                    ) }
                    { buttons::move_down(
                        model,
                        "Move the chart down",
                        move |_| msg::ChartsMsg::move_down(uid)
                    ) }
                    { buttons::close(
                        model,
                        "Close the chart",
                        move |_| msg::ChartsMsg::destroy(uid)
                    ) }

                    <h2> { self.spec.desc() } </h2>
                </center>
                <div
                    class=style::class::chart::style(self.visible)
                    id={&self.container}
                >
                    <canvas
                        id={&self.canvas}
                        class=style::class::chart::canvas::style()
                    />
                </div>
            </g>
        }
    }

    /// Creates a collapse or expand button depending on whether the chart is visible.
    fn expand_or_collapse_button(&self, model: &Model) -> Html {
        let uid = self.uid();
        if self.visible {
            buttons::collapse(model, "Collapse the chart", move |_| {
                msg::ChartsMsg::toggle_visible(uid)
            })
        } else {
            buttons::expand(model, "Expand the chart", move |_| {
                msg::ChartsMsg::toggle_visible(uid)
            })
        }
    }
}
