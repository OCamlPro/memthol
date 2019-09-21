//! Filter-handling.

use crate::base::*;

use charts::filter::{Filter, FilterSpec};

pub use charts::filter::{FilterUid, SubFilterUid};

/// Stores all the filters.
pub struct Filters {
    /// Sends messages to the model.
    to_model: Callback<Msg>,
    /// UID of the filter currently selected.
    ///
    /// `None` for `catch_all`.
    active: Option<FilterUid>,
    /// Catch-all filter.
    catch_all: FilterSpec,
    /// Actual filters.
    filters: Map<FilterUid, Filter>,
    /// Deleted filters.
    deleted: Vec<Filter>,
}

impl Filters {
    /// Constructor.
    pub fn new(to_model: Callback<Msg>) -> Self {
        Filters {
            to_model,
            active: None,
            catch_all: FilterSpec::new_catch_all(),
            filters: Map::new(),
            deleted: vec![],
        }
    }

    /// Gives mutable access to a filter specification.
    pub fn get_spec_mut(&mut self, uid: Option<FilterUid>) -> Option<&mut FilterSpec> {
        match uid {
            None => Some(&mut self.catch_all),
            Some(uid) => self.filters.get_mut(&uid).map(Filter::spec_mut),
        }
    }

    /// Pushes a filter.
    pub fn push(&mut self, filter: Filter) -> Res<()> {
        let uid = filter.uid();
        let prev = self.filters.insert(uid, filter);
        if let Some(filter) = prev {
            bail!(
                "found two filters with uid #{}, named `{}` and `{}`",
                uid,
                filter.spec().name(),
                self.filters.get(&uid).unwrap().spec().name()
            )
        }
        Ok(())
    }

    /// Removes a filter.
    pub fn remove(&mut self, uid: FilterUid) -> Res<()> {
        if let Some(filter) = self.filters.remove(&uid) {
            self.deleted.push(filter);
            Ok(())
        } else {
            bail!("failed to remove filter #{}: unknown UID", uid)
        }
    }

    /// Returns the number active and deleted filters, **not** including the catch-all.
    pub fn len(&self) -> usize {
        self.filters.len() + self.deleted.len()
    }

    /// Applies a function to all specification filters, including the deleted filters.
    ///
    /// The function is given a boolean flag that's true when the filter was deleted.
    ///
    /// Typically used when refreshing filters for a chart.
    pub fn specs_apply<F>(&self, mut f: F) -> Res<()>
    where
        F: FnMut(&FilterSpec, bool) -> Res<()>,
    {
        f(&self.catch_all, false)?;
        for filter in self.filters.values() {
            f(filter.spec(), false)?
        }
        for filter in &self.deleted {
            f(filter.spec(), true)?
        }
        Ok(())
    }
}

/// # Internal message handling
impl Filters {
    /// Applies a filter operation.
    pub fn update(&mut self, msg: msg::FilterMsg) -> Res<ShouldRender> {
        match msg {
            msg::FilterMsg::ToggleFilter(uid) => {
                self.active = uid;
                Ok(true)
            }
            msg::FilterMsg::ChangeName { uid, new_name } => {
                self.change_name(uid, new_name)?;
                Ok(true)
            }
            msg::FilterMsg::ChangeColor { uid, new_color } => {
                self.change_color(uid, new_color)?;
                Ok(true)
            }
        }
    }

    /// Changes the name of a filter.
    pub fn change_name(&mut self, uid: Option<FilterUid>, new_name: ChangeData) -> Res<()> {
        let new_name = match new_name.text_value() {
            Ok(new_name) => new_name,
            Err(e) => {
                let e: err::Err = e.into();
                bail!(e.chain_err(|| format!("while retrieving the new name of a filter")))
            }
        };
        if new_name.is_empty() {
            bail!("filter names cannot be empty")
        }
        if let Some(spec) = self.get_spec_mut(uid) {
            spec.set_name(new_name);
            spec.set_edited()
        } else {
            bail!(
                "unable to update the name of unknown filter #{} to `{}`",
                uid.map(|uid| uid.to_string())
                    .unwrap_or_else(|| "??".into()),
                new_name
            )
        }

        Ok(())
    }

    /// Changes the color of a filter.
    pub fn change_color(&mut self, uid: Option<FilterUid>, new_color: ChangeData) -> Res<()> {
        let new_color = match new_color.text_value() {
            Ok(new_color) => charts::color::Color::from_str(new_color)
                .chain_err(|| "while changing the color of a filter")?,
            Err(e) => {
                let e: err::Err = e.into();
                bail!(e.chain_err(|| format!("while retrieving the new color of a filter")))
            }
        };

        if let Some(spec) = self.get_spec_mut(uid) {
            info!("new color: {}", new_color);
            spec.set_color(new_color);
            spec.set_edited()
        } else {
            bail!(
                "unable to update the name of unknown filter #{} to `{}`",
                uid.map(|uid| uid.to_string())
                    .unwrap_or_else(|| "??".into()),
                new_color
            )
        }

        Ok(())
    }
}

/// # Server message handling
impl Filters {
    /// Applies an operation from the server.
    pub fn server_update(&mut self, msg: msg::from_server::FiltersMsg) -> Res<ShouldRender> {
        use msg::from_server::FiltersMsg::*;
        match msg {
            Rm(uid) => {
                self.remove(uid)?;
                Ok(true)
            }
            UpdateSpecs { catch_all, specs } => {
                self.update_specs(catch_all, specs)?;
                Ok(true)
            }
        }
    }

    /// Updates the specifications of the filters in the map.
    ///
    /// - also sends a message to the model to refresh the filters in all graphs.
    pub fn update_specs(
        &mut self,
        catch_all: Option<FilterSpec>,
        mut specs: Map<FilterUid, FilterSpec>,
    ) -> Res<()> {
        if let Some(mut spec) = catch_all {
            spec.unset_edited();
            self.catch_all = spec
        }
        for filter in self.filters.values_mut() {
            if let Some(mut spec) = specs.remove(&filter.uid()) {
                spec.unset_edited();
                *filter.spec_mut() = spec
            }
        }
        if !specs.is_empty() {
            bail!(
                "done updating filter specification, left with {} unknown filter UIDs",
                specs.len()
            )
        }
        self.to_model.emit(msg::ChartsMsg::refresh_filters());
        Ok(())
    }
}

/// # Rendering
impl Filters {
    /// Renders the tabs for each filter.
    pub fn render_tabs(&self) -> Html {
        html! {
            <>
                // Actual filters.
                { for self.filters.values().map(|filter| {
                    let active = Some(filter.uid()) == self.active;
                    filter.spec().render_tab(active)
                } ) }
                // Catch all.
                { self.catch_all.render_tab(self.active == None) }
            </>
        }
    }

    /// Renders the active filter.
    pub fn render_filter(&self) -> Html {
        let settings = match self.active {
            None => self.catch_all.render_settings(),
            Some(uid) => match self.filters.get(&uid) {
                Some(filter) => filter.spec().render_settings(),
                None => html!(<a/>),
            },
        };
        html! {
            <>
                { settings }
            </>
        }
    }
}

/// Extension trait for `FilterSpec`.
pub trait FilterSpecExt {
    /// Renders a spec as a tab.
    fn render_tab(&self, active: bool) -> Html;

    /// Renders the settings of a filter specification.
    fn render_settings(&self) -> Html;

    /// Adds itself as a series to a chart.
    fn add_series_to(&self, spec: &chart::ChartSpec, chart: &JsVal);
}

impl FilterSpecExt for FilterSpec {
    fn render_tab(&self, active: bool) -> Html {
        let uid = self.uid();
        let (class, colorize) = style::class::tabs::footer_get(active, self.color());
        let inner = html! {
            <a
                class = {class}
                style = {colorize}
                onclick = |_| if active {
                    msg::FooterMsg::toggle_tab(footer::FooterTab::Filter)
                } else {
                    msg::FilterMsg::toggle_filter(uid)
                }
            > {
                self.name()
            } </a>
        };
        html! {
            <li class={ style::class::tabs::li::get(false) }>
                { inner }
            </li>
        }
    }

    fn render_settings(&self) -> Html {
        let uid = self.uid();
        let spec = self.clone();
        html!(
            <>
                <ul class=style::class::filter::LINE>
                    <li class=style::class::filter::BUTTONS>
                        {
                            buttons::tickbox(
                                // Tickbox is ticked when the spec is not edited.
                                !self.edited(),
                                // If ticked, clicking does nothing.
                                |_| Msg::Noop,
                                // If unticked, clicking notifies the server.
                                move |_| msg::to_server::FiltersMsg::update_spec(
                                    uid, spec.clone()
                                ).into(),
                            )
                        }
                    </li>
                    <li class = style::class::filter::line::CELL>
                        <a class = style::class::filter::line::SECTION_CELL>
                            { "Settings" }
                        </a>
                    </li>
                </ul>

                <ul class=style::class::filter::LINE>
                    <li class=style::class::filter::BUTTONS/>

                    <li class=style::class::filter::line::CELL>
                        <a class=style::class::filter::line::SETTINGS_CELL>
                            { "name" }
                        </a>
                    </li>
                    <li class=style::class::filter::line::CELL>
                        <a class=style::class::filter::line::VAL_CELL>
                            <input
                                type="text"
                                onchange=|data| msg::FilterMsg::change_name(uid, data)
                                value=self.name()
                                class=style::class::filter::line::SETTINGS_VALUE_CELL
                            />
                        </a>
                    </li>
                </ul>

                <ul class=style::class::filter::LINE>
                    <li class=style::class::filter::BUTTONS/>

                    <li class=style::class::filter::line::CELL>
                        <a class=style::class::filter::line::SETTINGS_CELL>
                            { "color" }
                        </a>
                    </li>
                    <li class=style::class::filter::line::CELL>
                        <a class=style::class::filter::line::VAL_CELL>
                            <input
                                type="color"
                                onchange=|data| msg::FilterMsg::change_color(uid, data)
                                value=self.color().to_string()
                                class=style::class::filter::line::SETTINGS_VALUE_CELL
                            />
                        </a>
                    </li>
                </ul>
            </>
        )
    }

    fn add_series_to(&self, spec: &chart::ChartSpec, chart: &JsVal) {
        let series = js!(
            let color = @{self.color().to_string()};
            var series = new am4charts.LineSeries();
            series.interpolationDuration = @{cst::charts::INTERP_DURATION};
            series.defaultState.transitionDuration = 200;
            series.hiddenState.transitionDuration = 200;
            series.stroke = color;
            series.strokeWidth = 1;
            series.name = @{self.name()};
            series.fill = color;
            series.fillOpacity = 0.4;
            return series;
        );
        use chart::axis::AxisExt;
        spec.x_axis().series_apply(&series, self.uid());
        spec.y_axis().series_apply(&series, self.uid());
        js!(@(no_return)
            var chart = @{chart};
            console.log("adding " + @{&series}.name + " to chart");
            var series = @{series};
            chart.series.push(series);
            chart.scrollbarX.series.push(series);
        )
    }
}
