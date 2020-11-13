/*<LICENSE>
    This file is part of Memthol.

    Copyright (C) 2020 OCamlPro.

    Memthol is free software: you can redistribute it and/or modify
    it under the terms of the GNU General Public License as published by
    the Free Software Foundation, either version 3 of the License, or
    (at your option) any later version.

    Memthol is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
    GNU General Public License for more details.

    You should have received a copy of the GNU General Public License
    along with Memthol.  If not, see <https://www.gnu.org/licenses/>.
*/

//! Charts.

use plotters::prelude::*;

pub use charts::chart::{settings, ChartSpec};

prelude! {}

pub mod axis;
pub mod new;

/// The collection of charts.
pub struct Charts {
    /// The actual collection of charts.
    charts: Vec<Chart>,
    /// Chart constructor element.
    new_chart: new::NewChart,
    /// Name of the DOM node containing all the charts.
    dom_node_id: &'static str,
    /// Link to the model.
    link: Link,
}

impl Charts {
    /// Constructs an empty collection of charts.
    pub fn new(link: Link) -> Self {
        Self {
            charts: vec![],
            link,
            new_chart: new::NewChart::new(),
            dom_node_id: "charts_list",
        }
    }

    /// Name of the DOM node containing all the charts.
    pub fn dom_node_id(&self) -> &str {
        &self.dom_node_id
    }

    /// Number of charts.
    pub fn len(&self) -> usize {
        self.charts.len()
    }

    /// Sends a message to the model.
    pub fn send(&self, msg: Msg) {
        self.link.send_message(msg)
    }

    /// Retrieves the chart corresponding to some UID.
    fn get_mut(&mut self, uid: uid::Chart) -> Res<(usize, &mut Chart)> {
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
    fn destroy(&mut self, uid: uid::Chart) -> Res<ShouldRender> {
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
        filters: filter::Reference,
        action: msg::ChartsMsg,
    ) -> Res<ShouldRender> {
        use msg::ChartsMsg::*;

        match action {
            Move { uid, up } => self.move_chart(uid, up),
            Destroy(uid) => self.destroy(uid),

            RefreshFilters => self.refresh_filters(filters),

            NewChartSetX(x_axis) => self.new_chart.set_x_axis(x_axis),
            NewChartSetY(y_axis) => self.new_chart.set_y_axis(y_axis),

            ChartMsg { uid, msg } => {
                let (_, chart) = self.get_mut(uid)?;
                chart.update(msg)
            }
        }
    }

    /// Runs post-rendering actions.
    pub fn rendered(&mut self, filters: filter::Reference, stats: &AllFilterStats) {
        for chart in &mut self.charts {
            if let Err(e) = chart.rendered(filters, stats) {
                alert!("error while running `rendered`: {}", e)
            }
        }
    }

    /// Refreshes all filters in all charts.
    fn refresh_filters(&mut self, filters: filter::Reference) -> Res<ShouldRender> {
        for chart in &mut self.charts {
            chart.replace_filters(filters)?
        }

        // Rendering is done at JS-level, no need to render the HTML.
        Ok(false)
    }

    /// Move a chart, up if `up`, down otherwise.
    fn move_chart(&mut self, uid: uid::Chart, up: bool) -> Res<ShouldRender> {
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
}

/// # Rendering
impl Charts {
    /// Renders the charts.
    pub fn render(&self, model: &Model) -> Html {
        let charts_len = self.charts.len();

        html! {
            <>
                <div
                    id = model.charts().dom_node_id()
                >
                    { for self.charts.iter().enumerate().map(
                        |(pos, chart)| chart.render(
                            model,
                            layout::chart::ChartPos::from_pos_and_len(pos, charts_len)
                        )
                    ) }
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
        filters: filter::Reference,
        stats: &AllFilterStats,
        action: msg::from_server::ChartsMsg,
    ) -> Res<ShouldRender> {
        use msg::from_server::{ChartMsg, ChartsMsg};

        let should_render = match action {
            ChartsMsg::NewChart(spec, settings) => {
                log::info!("creating new chart");
                let chart = Chart::new(spec, settings, self.link.clone())?;
                self.charts.push(chart);
                true
            }

            ChartsMsg::NewPoints {
                mut points,
                refresh_filters,
            } => {
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
                for chart in &mut self.charts {
                    if let Some(points) = points.remove(&chart.uid()) {
                        chart.add_points(points, filters, stats)?
                    }
                }
                false
            }

            ChartsMsg::Chart { uid, msg } => {
                let (_index, chart) = self.get_mut(uid)?;
                match msg {
                    ChartMsg::NewPoints(points) => chart.overwrite_points(points)?,
                    ChartMsg::Points(points) => chart.add_points(points, filters, stats)?,
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
    /// Chart settings.
    settings: settings::Chart,
    /// Link to the model.
    link: Link,
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
        plotters::drawing::DrawingArea<plotters::CanvasBackend, plotters::coord::Shift>,
        web_sys::HtmlCanvasElement,
    )>,
    /// The points.
    points: Option<point::Points>,
    /// Previous filter map, used when updating filters to keep track of those that are hidden.
    prev_active: BTMap<uid::Line, bool>,

    /// This flag indicates whether the chart should be redrawn after HTML rendering.
    ///
    /// Note that only function [`draw`](#method.draw) is allowed to set this flag to false. This is
    /// because no drawing takes place when the chart is not visible. Hence, `draw` does **not**
    /// unset this flag so that its value is preserved until the chart is visible again. At that
    /// point the chart will be redrawn.
    redraw: bool,
    /// True if the chart settings are visible.
    settings_visible: bool,
}
impl Chart {
    /// Constructor.
    pub fn new(spec: ChartSpec, settings: settings::Chart, link: Link) -> Res<Self> {
        let top_container = format!("chart_container_{}", spec.uid().get());
        let container = format!("chart_canvas_container_{}", spec.uid().get());
        let canvas = format!("chart_canvas_{}", spec.uid().get());
        let collapsed_canvas = format!("{}_collapsed", canvas);

        Ok(Self {
            spec,
            settings,
            link,
            top_container,
            container,
            canvas,
            collapsed_canvas,
            chart: None,
            points: None,
            prev_active: BTMap::new(),
            settings_visible: false,
            redraw: true,
        })
    }

    /// Handles a message for this chart.
    pub fn update(&mut self, msg: msg::ChartMsg) -> Res<ShouldRender> {
        use msg::ChartMsg::*;
        match msg {
            SettingsToggleVisible => self.toggle_settings_visible(),
            FilterToggleVisible(l_uid) => self.filter_toggle_visible(l_uid)?,
            SettingsUpdate(msg) => self.settings.update(msg),
        }
        Ok(true)
    }

    /// UID accessor.
    pub fn uid(&self) -> uid::Chart {
        self.spec.uid()
    }

    /// True if the chart already has an associated canvas.
    pub fn has_canvas(&self) -> bool {
        self.chart.is_some()
    }

    /// True if the chart is visible.
    pub fn is_visible(&self) -> bool {
        self.settings.is_visible()
    }
    /// True if the settings of the chart are visible.
    ///
    /// - settings can only be visible if the chart is visible.
    pub fn is_settings_visible(&self) -> bool {
        self.is_visible() && self.settings_visible
    }

    /// Chart settings.
    #[inline]
    pub fn settings(&self) -> &settings::Chart {
        &self.settings
    }
    /// Chart title.
    #[inline]
    pub fn title(&self) -> &str {
        self.settings().title()
    }

    /// Chart specification.
    pub fn spec(&self) -> &ChartSpec {
        &self.spec
    }

    /// DOM identifier for the chart's top container.
    pub fn top_container_id(&self) -> &str {
        &self.top_container
    }
    /// DOM identifier for a chart's direct container.
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
    pub fn toggle_visible(&mut self) {
        self.settings.toggle_visible()
    }
    /// Toggles the visibility of the settings.
    pub fn toggle_settings_visible(&mut self) {
        self.settings_visible = !self.settings_visible
    }

    /// Accessor for filter visibility.
    pub fn filter_visibility(&self) -> &BTMap<uid::Line, bool> {
        &self.spec.active()
    }

    /// Destroys the chart.
    pub fn destroy(self) {}
}

/// # Features that (can) trigger a re-draw.
impl Chart {
    /// Toggles the visibility of a filter for the chart.
    pub fn filter_toggle_visible(&mut self, uid: uid::Line) -> Res<()> {
        if let Some(is_visible) = self.spec.active_mut().get_mut(&uid) {
            *is_visible = !*is_visible;
            self.redraw = true;
        } else {
            bail!("cannot toggle visibility of unknown filter {}", uid)
        }
        Ok(())
    }

    /// Replaces the filters of the chart.
    pub fn replace_filters(&mut self, filters: filter::Reference) -> Res<()> {
        self.prev_active.clear();
        let active = self.spec.active_mut();
        let prev_active = &mut self.prev_active;
        std::mem::swap(active, prev_active);

        debug_assert!(active.is_empty());

        filters.specs_apply(|spec| {
            let spec_uid = spec.uid();
            let visible = prev_active.get(&spec_uid).cloned().unwrap_or(true);
            let prev = active.insert(spec_uid, visible);
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
        filters: filter::Reference,
        stats: &AllFilterStats,
    ) -> Res<()> {
        let mut redraw = false;
        if let Some(my_points) = &mut self.points {
            let changed = my_points.extend(&mut points)?;
            if changed {
                self.draw(filters, stats)?
            }
            redraw = true;
        } else if !points.is_empty() {
            self.points = Some(points);
            self.draw(filters, stats)?;
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

/// # Canvas Handling.
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
        if let Some((_chart, canvas)) = self.chart.as_ref() {
            if let Some(parent) = canvas.parent_element() {
                parent.remove_child(&canvas).map_err(error_from_js_val)?;
            } else {
                return Ok(false);
            }
        }

        Ok(true)
    }

    /// Binds the canvas backend to the right DOM element.
    ///
    /// Creates the chart if needed.
    fn bind_canvas(&mut self) -> Res<()> {
        if let Some((_chart, canvas)) = self.chart.as_ref() {
            let canvas_container = self.get_canvas_container()?;
            let kids = canvas_container.children();
            for index in 0..kids.length() {
                if let Some(kid) = kids.item(index) {
                    kid.remove()
                }
            }
            canvas_container
                .append_child(canvas)
                .map_err(error_from_js_val)?;
        }

        Ok(())
    }

    /// Unbinds and re-binds the chart canvas.
    pub fn rebind_canvas(&mut self) -> Res<()> {
        self.try_unbind_canvas()?;
        self.bind_canvas()
    }

    /// Builds the actual JS chart and attaches it to its container.
    ///
    /// Also, makes the chart visible.
    pub fn build_chart(&mut self) -> Res<()> {
        log::info!("building chart");
        if self.settings.is_visible() {
            if self.chart.is_some() {
                bail!("asked to build and bind a chart that's already built and binded")
            }

            use wasm_bindgen::JsCast;
            let canvas: web_sys::HtmlCanvasElement = self
                .get_canvas()?
                .clone()
                .dyn_into()
                .expect("failed to retrieve chart canvas");
            let width = canvas.client_width();
            let height = canvas.client_height();
            let width = if width >= 0 { width as u32 } else { 0 };
            let height = if height >= 0 { height as u32 } else { 0 };
            log::info!(
                "original width/height: {}/{}",
                canvas.width(),
                canvas.height()
            );
            canvas.set_width(width);
            canvas.set_height(height);

            {
                let res_width = if width <= Self::CHART_X_DIFF {
                    width
                } else {
                    width - Self::CHART_X_DIFF
                };
                let res_height = if height <= Self::CHART_Y_DIFF {
                    height
                } else {
                    height - Self::CHART_Y_DIFF
                };
                log::info!(
                    "sending new resolution: {}x{} ({}x{})",
                    res_width,
                    res_height,
                    width,
                    height
                );
                self.link.send_message(Msg::ToServer(
                    charts::msg::ChartSettingsMsg::set_resolution(
                        self.spec.uid(),
                        (res_width, res_height),
                    ),
                ))
            }

            let backend: plotters::CanvasBackend =
                plotters::CanvasBackend::new(&self.canvas).expect("could not find canvas");

            let chart: plotters::prelude::DrawingArea<
                plotters::CanvasBackend,
                plotters::coord::Shift,
            > = backend.into_drawing_area();
            chart.fill(&plotters::style::colors::WHITE).unwrap();

            self.chart = Some((chart, canvas));
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Copy)]
struct Styler;
impl charts::point::StyleExt for Styler {
    fn mesh_conf<X, Y, DB>(&self, mesh: &mut plotters::chart::MeshStyle<X::Range, Y::Range, DB>)
    where
        X: charts::point::CoordExt,
        Y: charts::point::CoordExt,
        DB: plotters::prelude::DrawingBackend,
    {
        mesh.disable_x_mesh()
            .label_style(("sans-serif", 20).into_font())
            .axis_style(&plotters::prelude::BLACK)
            .bold_line_style(
                plotters::prelude::ShapeStyle::from(&plotters::prelude::BLACK.mix(0.2))
                    .stroke_width(1),
            )
            .light_line_style(&plotters::prelude::BLACK.mix(0.0));
    }

    fn shape_conf(&self, color: &charts::color::Color) -> plotters::style::ShapeStyle {
        let style = color.stroke_width(3);
        style
    }
}

impl Chart {
    /// Size of the x-axis label area.
    const X_LABEL_AREA: u32 = 30;
    /// Size of the y-axis label area.
    const Y_LABEL_AREA: u32 = 120;
    /// Size of the top margin.
    const TOP_MARGIN: u32 = Self::X_LABEL_AREA / 3;
    /// Size of the right margin.
    const RIGHT_MARGIN: u32 = Self::Y_LABEL_AREA / 3;

    /// Difference between the chart's canvas x-size and the chart's x-size.
    const CHART_X_DIFF: u32 = Self::Y_LABEL_AREA + Self::RIGHT_MARGIN;
    /// Difference between the chart's canvas y-size and the chart's y-size.
    const CHART_Y_DIFF: u32 = Self::X_LABEL_AREA + Self::TOP_MARGIN;

    /// Draws the chart, **takes care of updating `self.redraw`**.
    ///
    /// If the chart is not visible, drawing is postponed until the chart becomes visible. Meaning
    /// that this function does nothing if the chart is not visible.
    pub fn draw(&mut self, filters: filter::Reference, stats: &AllFilterStats) -> Res<()> {
        // If the chart's not visible, do nothing. We will draw once the chart becomes visible
        // again.
        if !self.settings.is_visible() {
            return Ok(());
        }

        let visible_filters = self.spec.active();

        if let Some((chart, canvas)) = &mut self.chart {
            let width = canvas.client_width();
            canvas.set_width(if width >= 0 { width as u32 } else { 0 });
            let height = canvas.client_height();
            canvas.set_height(if height >= 0 { height as u32 } else { 0 });

            let (chart_w, chart_h) = (canvas.width(), canvas.height());

            use wasm_bindgen::JsCast;
            let html_canvas: web_sys::HtmlCanvasElement = canvas.clone().dyn_into().unwrap();

            if html_canvas.width() != chart_w {
                html_canvas.set_width(chart_w)
            }
            if html_canvas.height() != chart_h {
                html_canvas.set_height(chart_h)
            }

            if let Some(points) = &self.points {
                chart.fill(&plotters::style::colors::WHITE).unwrap();

                let mut builder = plotters::prelude::ChartBuilder::on(&chart);
                builder
                    .margin_top(Self::TOP_MARGIN)
                    .margin_right(Self::RIGHT_MARGIN)
                    .x_label_area_size(Self::X_LABEL_AREA)
                    .y_label_area_size(Self::Y_LABEL_AREA);

                let is_catch_all_active = stats
                    .get(uid::Line::CatchAll)
                    .map(|stats| stats.alloc_count > 0)
                    .unwrap_or(true);
                let is_active = |f_uid: uid::Line| {
                    visible_filters.get(&f_uid).cloned().unwrap_or(false)
                        && (!f_uid.is_catch_all() || is_catch_all_active)
                };

                points.render(
                    &self.settings,
                    builder,
                    &Styler,
                    is_active,
                    filters.specs_iter().filter(|spec| is_active(spec.uid())),
                )?;

                chart
                    .present()
                    .map_err(|e| format!("error while presenting chart: {}", e))?
            }
        }

        Ok(())
    }
}

/// # Rendering
impl Chart {
    /// Runs post-rendering actions.
    pub fn rendered(&mut self, filters: filter::Reference, stats: &AllFilterStats) -> Res<()> {
        self.rebind_canvas()?;

        if self.chart.is_none() {
            self.build_chart()?;
            self.redraw = true;
        }

        if self.redraw {
            // Do **not** unset `self.redraw` here, function `draw` is in charge of that.
            self.draw(filters, stats)?;
        }
        Ok(())
    }

    /// Renders the chart.
    pub fn render(&self, model: &Model, pos: layout::chart::ChartPos) -> Html {
        layout::chart::render(model, self, pos)
    }
}
