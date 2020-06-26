//! Charts.

use plotters::prelude::*;

pub use charts::chart::ChartSpec;

use crate::common::*;

pub mod axis;
pub mod new;

pub use charts::chart::ChartUid;

/// The collection of charts.
pub struct Charts {
    /// The actual collection of charts.
    charts: Vec<Chart>,
    /// Callback to send messages to the model.
    to_model: Callback<Msg>,
    /// Chart constructor element.
    new_chart: new::NewChart,
    /// Name of the DOM node containing all the charts.
    dom_node_id: &'static str,
}

impl Charts {
    /// Constructs an empty collection of charts.
    pub fn new(to_model: Callback<Msg>) -> Self {
        Self {
            charts: vec![],
            to_model,
            new_chart: new::NewChart::new(),
            dom_node_id: "charts_list",
        }
    }

    /// Name of the DOM node containing all the charts.
    pub fn dom_node_id(&self) -> &str {
        &self.dom_node_id
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
            1,
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

        let filters = filters.reference_filters();

        match action {
            Move { uid, up } => self.move_chart(uid, up),
            ToggleVisible(uid) => self.toggle_visible(uid),
            Destroy(uid) => self.destroy(uid),

            FilterToggleVisible(uid, filter_uid) => self.filter_toggle_visible(uid, filter_uid),

            RefreshFilters => self.refresh_filters(filters),

            NewChartSetX(x_axis) => self.new_chart.set_x_axis(x_axis),
            NewChartSetY(y_axis) => self.new_chart.set_y_axis(y_axis),
        }
    }

    pub fn rendered(&mut self, filters: &filter::ReferenceFilters) {
        for chart in &mut self.charts {
            if let Err(e) = chart.rendered(filters) {
                alert!("error while running `rendered`: {}", e)
            }
        }
    }

    /// Refreshes all filters in all charts.
    fn refresh_filters(&mut self, filters: &filter::ReferenceFilters) -> Res<ShouldRender> {
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

        let changed = if up {
            self.try_move_chart_up(index)
        } else {
            self.try_move_chart_up(index + 1)
        }
        .chain_err(|| {
            format!(
                "while moving chart {} {}",
                uid,
                if up { "up" } else { "down" }
            )
        })?;

        Ok(changed)
    }

    /// Tries to move a chart. If the move is illegal, returns `false`.
    fn try_move_chart_up(&mut self, index: usize) -> Res<bool> {
        // Make sure the move is legal.
        let did_something = if index == 0 || index >= self.charts.len() {
            false
        } else {
            self.charts.swap(index, index - 1);
            true
        };
        Ok(did_something)
    }

    /// Toggles the visibility of a chart.
    fn toggle_visible(&mut self, uid: ChartUid) -> Res<ShouldRender> {
        let (_, chart) = self
            .get_mut(uid)
            .chain_err(|| format!("while changing chart visibility"))?;
        chart.toggle_visible()
    }

    /// Toggles the visibility of a chart.
    fn filter_toggle_visible(
        &mut self,
        uid: ChartUid,
        filter_uid: charts::uid::LineUid,
    ) -> Res<ShouldRender> {
        let (_, chart) = self
            .get_mut(uid)
            .chain_err(|| format!("while changing chart visibility"))?;
        chart.filter_toggle_visible(filter_uid)?;
        Ok(true)
    }
}

/// # Rendering
impl Charts {
    pub fn render(&self, model: &Model) -> Html {
        info!("rendering charts");
        for chart in &self.charts {
            info!("- {}", chart.uid())
        }
        html! {
            <>
                <div
                    id = model.charts().dom_node_id()
                >
                    { for self.charts.iter().map(|chart| chart.render(model)) }
                </div>
                { self.new_chart.render(model) }
            </>
        }
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

        let filters = filters.reference_filters();

        let should_render = match action {
            ChartsMsg::NewChart(spec) => {
                debug!("received a chart-creation message from the server");
                // let uid = spec.uid();
                let chart = Chart::new(spec, filters)?;
                self.charts.push(chart);
                // self.send(msg::ChartsMsg::build(uid));
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
                true
            }
            ChartsMsg::AddPoints(mut points) => {
                debug!("received an add-points message from the server");
                for chart in &mut self.charts {
                    if let Some(points) = points.remove(&chart.uid()) {
                        chart.add_points(points, filters)?
                    }
                }
                false
            }

            ChartsMsg::Chart { uid, msg } => {
                debug!("received a message specific to chart #{} from server", uid);
                let (_index, chart) = self.get_mut(uid)?;
                match msg {
                    ChartMsg::NewPoints(points) => chart.overwrite_points(points)?,
                    ChartMsg::Points(points) => chart.add_points(points, filters)?,
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
    /// DOM element containing the chart and its tabs.
    top_container: String,
    /// DOM element containing the canvas.
    container: String,
    /// Id of the actual DOM chart canvas.
    canvas: String,
    /// Id of the collapsed version of the chart canvas.
    collapsed_canvas: String,
    /// Chart drawing area.
    // chart: Option<DrawingArea<CanvasBackend, plotters::coord::Shift>>,
    chart: Option<(
        DrawingArea<CanvasBackend, plotters::coord::Shift>,
        web_sys::Element,
    )>,
    /// The points.
    points: Option<point::Points>,
    /// The filters, used to color the series and hide what the user asks to hide.
    filters: Map<charts::uid::LineUid, bool>,
    /// Previous filter map, used when updating filters to keep track of those that are hidden.
    prev_filters: Map<charts::uid::LineUid, bool>,

    redraw: bool,
}
impl Chart {
    /// Constructor.
    pub fn new(spec: ChartSpec, all_filters: &filter::ReferenceFilters) -> Res<Self> {
        let top_container = format!("chart_container_{}", spec.uid().get());
        let container = format!("chart_canvas_container_{}", spec.uid().get());
        let canvas = format!("chart_canvas_{}", spec.uid().get());
        let collapsed_canvas = format!("{}_collapsed", canvas);

        let mut filters = Map::new();
        all_filters.specs_apply(|spec| {
            let prev = filters.insert(spec.uid(), true);
            debug_assert!(prev.is_none());
            Ok(())
        })?;

        Ok(Self {
            spec,
            visible: true,
            top_container,
            container,
            canvas,
            collapsed_canvas,
            chart: None,
            points: None,
            filters,
            prev_filters: Map::new(),
            redraw: true,
        })
    }

    /// UID accessor.
    pub fn uid(&self) -> ChartUid {
        self.spec.uid()
    }

    pub fn has_canvas(&self) -> bool {
        self.chart.is_some()
    }

    /// True if the chart is visible.
    pub fn is_visible(&self) -> bool {
        self.visible
    }

    /// Chart specification.
    pub fn spec(&self) -> &ChartSpec {
        &self.spec
    }

    pub fn top_container_id(&self) -> &str {
        &self.top_container
    }
    pub fn container_id(&self) -> &str {
        &self.container
    }
    /// ID of the chart canvas.
    pub fn canvas_id(&self) -> &str {
        &self.canvas
    }
    /// ID of the collapsed canvas.
    pub fn collapsed_canvas_id(&self) -> &str {
        &self.collapsed_canvas
    }

    /// Toggles the visibility of the chart.
    pub fn toggle_visible(&mut self) -> Res<ShouldRender> {
        self.visible = !self.visible;
        Ok(true)
    }

    pub fn filter_visibility(&self) -> &Map<charts::uid::LineUid, bool> {
        &self.filters
    }

    /// Destroys the chart.
    pub fn destroy(self) {}
}

/// # Features that (can) trigger a re-draw.
impl Chart {
    /// Toggles the visibility of a filter for the chart.
    pub fn filter_toggle_visible(&mut self, uid: charts::uid::LineUid) -> Res<()> {
        if let Some(is_visible) = self.filters.get_mut(&uid) {
            *is_visible = !*is_visible;
            self.redraw = true;
            Ok(())
        } else {
            bail!("cannot toggle visibility of unknown filter {}", uid)
        }
    }

    /// Replaces the filters of the chart.
    pub fn replace_filters(&mut self, filters: &filter::ReferenceFilters) -> Res<()> {
        self.prev_filters.clear();
        std::mem::swap(&mut self.filters, &mut self.prev_filters);

        debug_assert!(self.filters.is_empty());

        filters.specs_apply(|spec| {
            let spec_uid = spec.uid();
            let visible = self.prev_filters.get(&spec_uid).cloned().unwrap_or(true);
            let prev = self.filters.insert(spec_uid, visible);
            debug_assert!(prev.is_none());
            Ok(())
        })?;

        self.redraw = true;
        Ok(())
    }

    /// Appends some points to the chart.
    pub fn add_points(
        &mut self,
        mut points: point::Points,
        filters: &filter::ReferenceFilters,
    ) -> Res<()> {
        let mut redraw = false;
        if let Some(my_points) = &mut self.points {
            let changed = my_points.extend(&mut points)?;
            if changed {
                self.draw(filters)?
            }
            redraw = true;
        } else if !points.is_empty() {
            self.points = Some(points);
            self.draw(filters)?;
            redraw = true;
        }

        self.redraw = self.redraw || redraw;
        Ok(())
    }

    /// Overwrites the points in a chart.
    pub fn overwrite_points(&mut self, points: point::Points) -> Res<()> {
        self.points = Some(points);
        self.redraw = true;
        Ok(())
    }
}

/// # Functions for message-handling.
impl Chart {
    /// Retrieves the canvas associated with a chart.
    fn get_canvas_container(&self) -> Res<web_sys::Element> {
        js::get_element_by_id(&self.container)
            .chain_err(|| format!("while retrieving canvas container for chart {}", self.uid()))
    }

    /// Retrieves the canvas associated with a chart.
    fn get_canvas(&self) -> Res<web_sys::Element> {
        js::get_element_by_id(&self.canvas)
            .chain_err(|| format!("while retrieving canvas for chart {}", self.uid()))
    }

    fn try_unbind_canvas(&self) -> Res<bool> {
        debug!("try_unbind_canvas {}", self.spec.uid());

        if let Some((_chart, canvas)) = self.chart.as_ref() {
            info!("canvas id: {}", canvas.id());

            if let Some(parent) = canvas.parent_element() {
                info!(
                    "unbinding canvas {} to container {}",
                    canvas.id(),
                    parent.id()
                );
                parent.remove_child(&canvas).map_err(err::from_js_val)?;
            } else {
                return Ok(false);
            }
        } else {
            debug!("no canvas to unbind")
        }

        Ok(true)
    }

    /// Binds the canvas backend to the right DOM element.
    ///
    /// Creates the chart if needed.
    fn bind_canvas(&mut self) -> Res<()> {
        debug!("bind_canvas {}", self.spec.uid());

        if let Some((_chart, canvas)) = self.chart.as_ref() {
            let canvas_container = self.get_canvas_container()?;
            info!(
                "binding canvas {} to container {}",
                canvas.id(),
                canvas_container.id()
            );

            canvas_container
                .append_child(canvas)
                .map_err(err::from_js_val)?;
        } else {
            debug!("no canvas to bind")
        }

        Ok(())
    }

    pub fn rebind_canvas(&mut self) -> Res<()> {
        self.try_unbind_canvas()?;
        self.bind_canvas()
    }

    /// Builds the actual JS chart and attaches it to its container.
    ///
    /// Also, makes the chart visible.
    pub fn build_chart(&mut self) -> Res<()> {
        if self.visible {
            info!("building chart");
            if self.chart.is_some() {
                bail!("asked to build and bind a chart that's already built and binded")
            }

            let backend: CanvasBackend =
                plotters::prelude::CanvasBackend::new(&self.canvas).expect("could not find canvas");

            // let (width, height) = backend.get_size();
            // info!("backend: {}/{}", width, height);

            let chart: DrawingArea<CanvasBackend, plotters::coord::Shift> =
                backend.into_drawing_area();
            chart.fill(&WHITE).unwrap();

            let canvas = self.get_canvas()?;

            self.chart = Some((chart, canvas));
        }

        Ok(())
    }

    /// Draws the chart.
    ///
    /// # TODO
    ///
    /// - this function contains code that's highly specific to the kind of points we are drawing.
    ///   It should be exported, probably in the `charts` crate, to keep this function focused on
    ///   what it does.
    pub fn draw(&mut self, filters: &filter::ReferenceFilters) -> Res<()> {
        let visible_filters = &self.filters;

        debug!("drawing {}", self.spec.uid());

        if let Some((chart, canvas)) = &mut self.chart {
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
                        alert!(
                            "An error occured while resizing a chart's canvas: negative height."
                        );
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

            if let Some(points) = &self.points {
                match points {
                    charts::point::Points::Time(charts::point::TimePoints::Size(points)) => {
                        let (mut min_x, mut max_x, mut max_y) = (None, None, 5);

                        for point in points.iter() {
                            if min_x.is_none() {
                                min_x = Some(&point.key)
                            }
                            max_x = Some(&point.key);

                            max_y = point.vals.map.iter().fold(max_y, |max_y, (uid, val)| {
                                let visible = visible_filters.get(uid).cloned().unwrap_or(false);
                                if visible && *val > max_y {
                                    *val
                                } else {
                                    max_y
                                }
                            });
                        }

                        let min_y = 0;
                        let max_y = (max_y + std::cmp::max(1, max_y / 20)) as u32;

                        let (min_x, max_x) = match (min_x, max_x) {
                            (Some(min_x), Some(max_x)) => (*min_x, *max_x),
                            (min_x, max_x) => {
                                warn!(
                                    "could not retrieve chart min/max x-values for chart #{} \
                                    ({:?}, {:?})",
                                    self.spec.uid(),
                                    min_x,
                                    max_x,
                                );
                                return Ok(());
                            }
                        };

                        info!("chart size: {}/{}, {}/{}", min_x, max_x, min_y, max_y);

                        // chart.fill(&BLACK.mix(0.5)).unwrap();
                        chart.fill(&WHITE).unwrap();

                        // let (width, height) = chart.dim_in_pixel();
                        // let margin = height * 2 / 100;
                        let (x_label_area, y_label_area) = (30, 60);
                        let (margin_top, margin_right) = (x_label_area / 3, y_label_area / 3);

                        let mut chart_cxt: ChartContext<
                            CanvasBackend,
                            RangedCoord<RangedDuration, RangedCoordu32>,
                        > = ChartBuilder::on(&chart)
                            .margin_top(margin_top)
                            .margin_right(margin_right)
                            .x_label_area_size(x_label_area)
                            .y_label_area_size(y_label_area)
                            .build_ranged(
                                chrono::Duration::from_std(Duration::from_millis(0)).unwrap()
                                    ..(max_x.date().clone() - min_x.date().clone()),
                                min_y..max_y,
                            )
                            .map_err(|e| err::Err::from(e.to_string()))?;

                        chart_cxt
                            .configure_mesh()
                            .disable_x_mesh()
                            .label_style(("sans-serif", 15).into_font())
                            .x_label_formatter(&|date| {
                                let mut secs = date.to_std().unwrap().as_secs();
                                println!("secs: {}", secs);
                                let mut mins = secs / 60;
                                secs = secs - mins * 60;
                                println!("- {}:{}", mins, secs);
                                let hours = mins / 60;
                                mins = mins - hours * 60;
                                println!("- {}:{}:{}", hours, mins, secs);
                                let mut s = String::with_capacity(10);
                                use std::fmt::Write;
                                if hours > 0 {
                                    write!(&mut s, "{}h", hours).unwrap()
                                }
                                if mins > 0 {
                                    write!(&mut s, "{}m", mins).unwrap()
                                }
                                write!(&mut s, "{}s", secs).unwrap();
                                s
                            })
                            .draw()
                            .unwrap();

                        filters.specs_apply(|spec| {
                            let uid = spec.uid();

                            if visible_filters.get(&uid).cloned().unwrap_or(false) {
                                let color = color::to_plotters(spec.color());

                                let point_iter = points.iter().filter_map(|point| {
                                    point.vals.map.get(&uid).map(|val| {
                                        (
                                            point.key.date().clone() - min_x.date().clone(),
                                            *val as u32,
                                        )
                                    })
                                });

                                chart_cxt
                                    .draw_series(LineSeries::new(point_iter, color.stroke_width(1)))
                                    .map_err(|e| e.to_string())?;
                            }
                            Ok(())
                        })?;

                        chart
                            .present()
                            .map_err(|e| format!("error while presenting chart: {}", e))?
                    }
                }
            }
        }

        Ok(())
    }
}

/// # Rendering
impl Chart {
    pub fn rendered(&mut self, filters: &filter::ReferenceFilters) -> Res<()> {
        self.rebind_canvas()?;

        if self.chart.is_none() {
            self.build_chart()?;
            self.redraw = true;
        }

        if self.redraw {
            self.draw(filters)?;
            self.redraw = false;
        }
        Ok(())
    }

    /// Renders the chart.
    pub fn render(&self, model: &Model) -> Html {
        self.try_unbind_canvas().unwrap();
        layout::chart::render(model, self)
    }
}
