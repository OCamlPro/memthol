//! Filter-handling.

use crate::base::*;

use charts::filter::{Filter, FilterSpec};

pub use charts::filter::{FilterUid, SubFilter, SubFilterUid};

/// Stores all the filters.
pub struct Filters {
    /// Sends messages to the model.
    to_model: Callback<Msg>,
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

    /// Gives mutable access to a filter.
    pub fn get_mut(&mut self, uid: FilterUid) -> Res<&mut Filter> {
        self.filters
            .get_mut(&uid)
            .ok_or_else(|| format!("unknown filter UID #{}", uid).into())
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
    pub fn update(&mut self, msg: msg::FiltersMsg) -> Res<ShouldRender> {
        use msg::{FilterSpecMsg::*, FiltersMsg::*};
        match msg {
            FilterSpec {
                uid,
                msg: ChangeName(new_name),
            } => {
                self.change_name(uid, new_name)?;
                Ok(true)
            }
            FilterSpec {
                uid,
                msg: ChangeColor(new_color),
            } => {
                self.change_color(uid, new_color)?;
                Ok(true)
            }
            Filter { uid, msg } => {
                let filter = self.get_mut(uid)?;
                filter_update(filter, msg)
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
            Add(filter) => {
                let prev = self.filters.insert(filter.uid(), filter);
                if let Some(prev) = prev {
                    bail!("filter UID collision on #{}", prev.uid())
                }
                Ok(true)
            }
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
    pub fn render_tabs(&self, active: Option<Option<FilterUid>>) -> Html {
        html! {
            <>
                // Catch all.
                { self.catch_all.render_tab(active == Some(None)) }
                // Actual filters.
                { for self.filters.values().rev().map(|filter| {
                    let active = Some(Some(filter.uid())) == active;
                    filter.spec().render_tab(active)
                } ) }
                // Add filter button.
                <li
                    class = style::class::tabs::li::get(false)
                >
                    { Button::add(
                        "Create a new filter",
                        |_| msg::to_server::FiltersMsg::add_new().into()
                    ) }
                </li>
            </>
        }
    }

    /// Renders the active filter.
    pub fn render_filter(&self, active: Option<FilterUid>) -> Html {
        let (settings, filter_opt) = match active {
            None => (self.catch_all.render_settings(), None),
            Some(uid) => match self.filters.get(&uid) {
                Some(filter) => (filter.spec().render_settings(), Some(filter)),
                None => (html!(<a/>), None),
            },
        };
        html! {
            <>
                { settings }
                {
                    if let Some(filter) = filter_opt {
                        render_subs(filter)
                    } else {
                        html!(<a/>)
                    }
                }
            </>
        }
    }
}

/// Extension trait for `FilterSpec`.
pub trait FilterSpecExt {
    /// Renders a spec as a tab.
    fn render_tab(&self, active: bool) -> Html;

    /// Adds itself as a series to a chart.
    fn add_series_to(&self, spec: &chart::ChartSpec, chart: &JsVal);

    /// Renders the settings of a filter specification.
    fn render_settings(&self) -> Html;
}

impl FilterSpecExt for FilterSpec {
    fn render_tab(&self, active: bool) -> Html {
        let uid = self.uid();
        let (class, colorize) = style::class::tabs::footer_get(active, self.color());
        let inner = html! {
            <a
                class = class
                style = colorize
                onclick = |_| msg::FooterMsg::toggle_tab(footer::FooterTab::filter(uid))
            > {
                self.name()
            } </a>
        };
        html! {
            <li class = style::class::tabs::li::get(false)>
                { inner }
            </li>
        }
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

    fn render_settings(&self) -> Html {
        let uid = self.uid();
        let spec = self.clone();
        html!(
            <>
                <ul class = style::class::filter::LINE>
                    <li class = style::class::filter::BUTTONS>
                        {
                            if self.edited() {
                                Button::inactive_tickbox(
                                    "Apply the settings",
                                    move |_| msg::to_server::FiltersMsg::update_spec(
                                        uid, spec.clone()
                                    ).into()
                                )
                            } else {
                                Button::active_tickbox(
                                    "Settings have not been edited",
                                    move |_| Msg::Noop,
                                )
                            }
                        }
                    </li>
                    <li class = style::class::filter::line::CELL>
                        <a class = style::class::filter::line::SECTION_CELL>
                            { "Settings" }
                        </a>
                    </li>
                </ul>

                <ul class = style::class::filter::LINE>
                    <li class = style::class::filter::BUTTONS/>

                    <li class = style::class::filter::line::CELL>
                        <a class = style::class::filter::line::SETTINGS_CELL>
                            { "name" }
                        </a>
                    </li>
                    <li class = style::class::filter::line::CELL>
                        <a class = style::class::filter::line::VAL_CELL>
                            <input
                                type = "text"
                                onchange = |data| msg::FilterSpecMsg::change_name(uid, data)
                                value = self.name()
                                class = style::class::filter::line::SETTINGS_VALUE_CELL
                            />
                        </a>
                    </li>
                </ul>

                <ul class = style::class::filter::LINE>
                    <li class = style::class::filter::BUTTONS/>

                    <li class = style::class::filter::line::CELL>
                        <a class = style::class::filter::line::SETTINGS_CELL>
                            { "color" }
                        </a>
                    </li>
                    <li class = style::class::filter::line::CELL>
                        <a class = style::class::filter::line::VAL_CELL>
                            <input
                                type = "color"
                                onchange = |data| msg::FilterSpecMsg::change_color(uid, data)
                                value = self.color().to_string()
                                class = style::class::filter::line::SETTINGS_VALUE_CELL
                            />
                        </a>
                    </li>
                </ul>
            </>
        )
    }
}

fn filter_update(filter: &mut Filter, msg: msg::FilterMsg) -> Res<ShouldRender> {
    use msg::FilterMsg::*;
    match msg {
        AddNew => {
            let sub = SubFilter::default();
            filter.insert(sub)?;
            filter.set_edited();
            Ok(true)
        }
        Sub(sub) => filter.replace(sub).map(|()| true),
    }
}

fn render_subs(filter: &Filter) -> Html {
    html! {
        <>
            <ul class = style::class::filter::LINE>
                <li class = style::class::filter::BUTTONS>
                    {
                        let uid = filter.uid();
                        // # TODO
                        // Fix this.
                        let subs: Vec<_> = filter.iter().map(SubFilter::clone).collect();
                        if filter.edited() {
                            Button::inactive_tickbox(
                                "Apply the modifications",
                                move |_| msg::to_server::FilterMsg::replace_subs(
                                    uid, subs.clone()
                                ).into()
                            )
                        } else {
                            Button::active_tickbox(
                                "Filters have not been edited",
                                move |_| Msg::Noop,
                            )
                        }
                    }
                </li>
                <li class = style::class::filter::line::CELL>
                    <a class = style::class::filter::line::SECTION_CELL>
                        { "Filters" }
                    </a>
                </li>
            </ul>
            { for filter.iter().map(|sub| {
                if sub.is_from_client() {
                    info!("from client");
                } else {
                    info!("not from client");
                }
                html!(
                    <ul class = style::class::filter::LINE>
                        <li class = style::class::filter::BUTTONS>
                            { Button::close(
                                "Remove the filter",
                                move |_| Msg::warn("filter removing is not implemented")
                            ) }
                        </li>
                        {
                            let uid = filter.uid();
                            sub::prop_selector(
                                sub,
                                move |res| {
                                    let res = res.map(|sub|
                                        msg::FilterMsg::update_sub(uid, sub)
                                    );
                                    err::msg_of_res(res)
                                }
                            )
                        }
                        {
                            let uid = filter.uid();
                            sub::render_cells(
                                sub,
                                move |res| {
                                    let res = res.map(|sub|
                                        msg::FilterMsg::update_sub(uid, sub)
                                    );
                                    err::msg_of_res(res)
                                }
                            )
                        }
                    </ul>
                )}
            ) }

            // Add button to add subfilters.
            <ul class = style::class::filter::LINE>
                <li class = style::class::filter::BUTTONS>
                    {
                        let uid = filter.uid();
                        Button::add("Add a new subfilter", move |_| msg::FilterMsg::add_new(uid))
                    }
                </li>
            </ul>
        </>
    }
}

mod sub {
    use super::*;
    use charts::filter::{FilterKind, SubFilter};

    pub fn render_cells<Update>(filter: &SubFilter, update: Update) -> Html
    where
        Update: Fn(Res<SubFilter>) -> Msg + 'static + Clone,
    {
        pub use charts::filter::sub::RawSubFilter::*;
        // Function that constructs a subfilter with the same UID.
        let uid = filter.uid();
        match filter.raw() {
            Size(filter) => sub::size::render(filter, move |res| {
                let res = res.map(|raw| SubFilter::new(uid, Size(raw)));
                update(res)
            }),
            Label(_filter) => html! {
                <>
                </>
            },
        }
    }

    pub fn prop_selector<Update>(filter: &SubFilter, update: Update) -> Html
    where
        Update: Fn(Res<SubFilter>) -> Msg + 'static,
    {
        let filter = filter.clone();
        let selected = filter.kind();
        html! {
            <li class=style::class::filter::line::CELL>
                <a class=style::class::filter::line::PROP_CELL>
                    <Select<FilterKind>
                        selected=Some(selected)
                        options=FilterKind::all()
                        onchange=move |kind| {
                            let mut filter = filter.clone();
                            filter.change_kind(kind);
                            update(Ok(filter))
                        }
                    />
                </a>
            </li>
        }
    }

    mod size {
        use super::*;
        use charts::filter::{ord::Kind, SizeFilter};

        /// Renders a size filter given an update function.
        pub fn render<Update>(filter: &SizeFilter, update: Update) -> Html
        where
            Update: Fn(Res<SizeFilter>) -> Msg + Clone + 'static,
        {
            html! {
                <>
                    <li class=style::class::filter::line::CELL>
                        <a class=style::class::filter::line::CMP_CELL>
                            { kind_selector(filter, update.clone()) }
                        </a>
                    </li>
                    { render_values(filter, update) }
                </>
            }
        }

        fn parse_text_data(data: ChangeData) -> Res<usize> {
            use alloc_data::Parseable;
            data.text_value()
                .and_then(|text| usize::parse(text).map_err(|e| e.into()))
        }

        fn kind_selector<Update>(filter: &SizeFilter, update: Update) -> Html
        where
            Update: Fn(Res<SizeFilter>) -> Msg + 'static,
        {
            let selected = filter.cmp_kind();
            html! {
                <Select<Kind>
                    selected=Some(selected)
                    options=Kind::all()
                    onchange=move |kind| update(Ok(SizeFilter::default_of_cmp(kind)))
                />
            }
        }

        fn render_values<Update>(filter: &SizeFilter, update: Update) -> Html
        where
            Update: Fn(Res<SizeFilter>) -> Msg + Clone + 'static,
        {
            match filter {
                SizeFilter::Cmp { cmp, val } => {
                    let cmp = *cmp;
                    html! {
                        <li class=style::class::filter::line::CELL>
                            <a class=style::class::filter::line::VAL_CELL>
                                <input
                                    type="text"
                                    class=style::class::filter::line::TEXT_VALUE
                                    value=val.to_string()
                                    onchange=|data| update(
                                        parse_text_data(data)
                                            .chain_err(||
                                                format!(
                                                    "while parsing value for filter operator `{}`",
                                                    cmp
                                                )
                                            )
                                            .map(|val|
                                                SizeFilter::Cmp { cmp, val }
                                            )
                                    )
                                >
                                </input>
                            </a>
                        </li>
                    }
                }
                SizeFilter::In { lb, ub } => {
                    let (lb_clone, ub_clone) = (lb.clone(), ub.clone());
                    let other_update = update.clone();
                    html! {
                        <li class=style::class::filter::line::CELL>
                            <a class=style::class::filter::line::VAL_CELL>
                                <code> { "[" } </code>
                                <input
                                    type="text"
                                    class=style::class::filter::line::TEXT_VALUE
                                    value=lb.to_string()
                                    onchange=|data| {
                                        let ub = ub_clone.clone();
                                        other_update(
                                            parse_text_data(data)
                                                .and_then(|lb| SizeFilter::between(lb, ub))
                                                .chain_err(||
                                                    format!(
                                                        "while parsing lower bound \
                                                        for filter operator `{}`",
                                                        Kind::In
                                                    )
                                                )
                                        )
                                    }
                                >
                                </input>
                                <code> { "," } </code>
                                <input
                                    type="text"
                                    class=style::class::filter::line::TEXT_VALUE
                                    value=ub.to_string()
                                    onchange=|data| {
                                        let lb = lb_clone.clone();
                                        update(
                                            parse_text_data(data)
                                                .and_then(|ub| SizeFilter::between(lb, ub))
                                                .chain_err(||
                                                    format!(
                                                        "while parsing upper bound \
                                                        for filter operator `{}`",
                                                        Kind::In
                                                    )
                                                )
                                        )
                                    }
                                >
                                </input>
                                <code> { "]" } </code>
                            </a>
                        </li>
                    }
                }
            }
        }

    }

}
