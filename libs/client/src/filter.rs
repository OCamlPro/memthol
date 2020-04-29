//! Filter-handling.

use crate::common::*;

use charts::filter::{Filter, FilterSpec};

pub use charts::filter::{FilterUid, LineUid, SubFilter, SubFilterUid};

/// Filter rendering info.
pub struct FilterRenderInfo {
    /// True if the filter has been edited *w.r.t.* the server version.
    pub edited: bool,
    /// True if the filter is the first in the list of match filters.
    pub is_first: bool,
    /// True if the filter is the last in the list of match filters.
    pub is_last: bool,
}
impl FilterRenderInfo {
    /// Constructs a non-match filter info.
    ///
    /// *Non-match* filters are the *everything* and *catch-all* filters.
    pub fn new_non_match() -> Self {
        Self {
            edited: false,
            is_first: false,
            is_last: false,
        }
    }
}

/// Stores all the filters.
pub struct Filters {
    /// Sends messages to the model.
    to_model: Callback<Msg>,
    /// Catch-all filter.
    catch_all: FilterSpec,
    /// Everything filter.
    everything: FilterSpec,
    /// Actual filters.
    filters: Vec<Filter>,
    /// True if a filter was (re)moved and the server was not notified.
    edited: bool,
}

impl Filters {
    /// Constructor.
    pub fn new(to_model: Callback<Msg>) -> Self {
        Filters {
            to_model,
            catch_all: FilterSpec::new_catch_all(),
            everything: FilterSpec::new_everything(),
            filters: Vec::new(),
            edited: false,
        }
    }

    /// True if at least one filter was edited.
    pub fn edited(&self) -> bool {
        if self.edited || self.everything.edited() || self.catch_all.edited() {
            return true;
        }
        for filter in &self.filters {
            if filter.edited() || filter.spec().edited() {
                return true;
            }
        }
        false
    }

    /// Retrieves a filter from its UID.
    fn get_filter(&self, uid: FilterUid) -> Res<(usize, &Filter)> {
        for (index, filter) in self.filters.iter().enumerate() {
            if filter.uid() == uid {
                return Ok((index, filter));
            }
        }
        bail!("unknown filter uid #{}", uid)
    }

    /// Retrieves a filter from its UID, mutable version.
    fn get_filter_mut(&mut self, uid: FilterUid) -> Res<(usize, &mut Filter)> {
        for (index, filter) in self.filters.iter_mut().enumerate() {
            if filter.uid() == uid {
                return Ok((index, filter));
            }
        }
        bail!("unknown filter uid #{}", uid)
    }

    /// Gives mutable access to a filter specification.
    pub fn get_spec_mut(&mut self, uid: LineUid) -> Res<&mut FilterSpec> {
        match uid {
            LineUid::CatchAll => Ok(&mut self.catch_all),
            LineUid::Everything => Ok(&mut self.everything),
            LineUid::Filter(uid) => self
                .get_filter_mut(uid)
                .map(|(_index, filter)| filter.spec_mut()),
        }
    }

    /// Pushes a filter.
    pub fn push(&mut self, filter: Filter) {
        self.filters.push(filter)
    }

    /// Removes a filter.
    pub fn remove(&mut self, uid: FilterUid) -> Res<()> {
        let (index, _) = self.get_filter(uid)?;
        self.filters.remove(index);
        Ok(())
    }

    /// Returns the number filters, **not** including the catch-all.
    pub fn len(&self) -> usize {
        self.filters.len()
    }

    /// Applies a function to all specification filters.
    ///
    /// Typically used when refreshing filters for a chart.
    pub fn specs_apply<F>(&self, mut f: F) -> Res<()>
    where
        F: FnMut(&FilterSpec) -> Res<()>,
    {
        // Only run on "everything" if there are no filters.
        if self.filters.is_empty() {
            f(&self.everything)?
        } else {
            f(&self.everything)?;
            for filter in &self.filters {
                f(filter.spec())?
            }
            f(&self.catch_all)?
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
            Save => {
                self.edited = false;
                let catch_all = self.catch_all.clone();
                self.catch_all.unset_edited();
                let everything = self.everything.clone();
                self.everything.unset_edited();
                let mut filters = Vec::with_capacity(self.filters.len());

                for filter in &mut self.filters {
                    filters.push(filter.clone());
                    filter.unset_edited();
                    filter.spec_mut().unset_edited()
                }
                self.to_model.emit(
                    msg::to_server::FiltersMsg::update_all(everything, filters, catch_all).into(),
                );
                Ok(true)
            }

            Rm(uid) => {
                self.remove(uid)?;
                self.edited = true;
                self.to_model.emit(msg::FooterMsg::removed(uid));
                Ok(true)
            }

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
                let (_index, filter) = self.get_filter_mut(uid)?;
                filter_update(filter, msg)
            }
            Move { uid, left } => {
                let (index, _) = self.get_filter(uid)?;
                let to_swap = if self.filters.len() < 2 {
                    // There's at most one filter, no move to do.
                    //
                    // This first check guarantees that the third `if` below is legal.
                    None
                } else if left && index > 0 {
                    // Going left and not the left-most index, swap.
                    Some((index - 1, index))
                } else if !left && index < self.filters.len() - 1 {
                    // Going right and not the right-most index, swap.
                    Some((index, index + 1))
                } else {
                    // Everything else causes no move.
                    None
                };
                if let Some((i_1, i_2)) = to_swap {
                    self.filters.swap(i_1, i_2);
                    self.edited = true;
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
        }
    }

    /// Sends a message to the model to refresh the filters of all charts.
    pub fn send_refresh_filters(&self) {
        self.to_model.emit(msg::ChartsMsg::refresh_filters())
    }

    /// Changes the name of a filter.
    pub fn change_name(&mut self, uid: LineUid, new_name: ChangeData) -> Res<()> {
        let new_name = match new_name {
            yew::html::ChangeData::Value(txt) => txt,
            err @ yew::html::ChangeData::Select(_) | err @ yew::html::ChangeData::Files(_) => {
                bail!("unexpected text field update {:?}", err)
            }
        };
        if new_name.is_empty() {
            bail!("filter names cannot be empty")
        }
        let spec = self
            .get_spec_mut(uid)
            .chain_err(|| "while updating a filter's name")?;
        spec.set_name(new_name);
        spec.set_edited();

        Ok(())
    }

    /// Changes the color of a filter.
    pub fn change_color(&mut self, uid: LineUid, new_color: ChangeData) -> Res<()> {
        let new_color = match new_color {
            yew::html::ChangeData::Value(new_color) => charts::color::Color::from_str(new_color)
                .chain_err(|| "while changing the color of a filter")?,
            err @ yew::html::ChangeData::Select(_) | err @ yew::html::ChangeData::Files(_) => {
                bail!("unexpected text field update {:?}", err)
            }
        };

        let spec = self
            .get_spec_mut(uid)
            .chain_err(|| "while updating the color of a filter")?;
        spec.set_color(new_color);
        spec.set_edited();

        Ok(())
    }
}

/// # Server message handling
impl Filters {
    /// Applies an operation from the server.
    pub fn server_update(&mut self, msg: msg::from_server::FiltersMsg) -> Res<ShouldRender> {
        use msg::from_server::FiltersMsg::*;
        match msg {
            Add(filter) => self.add_filter(filter),
            Revert {
                everything,
                filters,
                catch_all,
            } => {
                self.catch_all = catch_all;
                self.everything = everything;
                for filter in &self.filters {
                    let uid = filter.uid();
                    if filters.iter().all(|filter| filter.uid() != uid) {
                        self.to_model.emit(msg::FooterMsg::removed(uid));
                    }
                }
                self.filters.clear();
                for filter in filters {
                    self.push(filter)
                }
                Ok(true)
            }
            UpdateSpecs(specs) => self.update_specs(specs),
        }
    }

    /// Adds a filter in the map.
    ///
    /// - makes the new filter active;
    /// - triggers a refresh of the filters of all the charts.
    pub fn add_filter(&mut self, mut filter: Filter) -> Res<ShouldRender> {
        filter.set_edited();
        let uid = filter.uid();
        self.filters.push(filter);
        self.to_model
            .emit(msg::FooterMsg::toggle_tab(footer::FooterTab::filter(
                LineUid::Filter(uid),
            )));
        Ok(true)
    }

    /// Removes a filter from the map.
    ///
    /// - changes the active filter to catch-all.
    pub fn rm_filter(&mut self, uid: FilterUid) -> Res<ShouldRender> {
        self.remove(uid)?;
        self.to_model.emit(msg::FooterMsg::removed(uid));
        self.send_refresh_filters();
        Ok(true)
    }

    /// Updates the specifications of the filters in the map.
    ///
    /// - triggers a refresh of the filters of all the charts.
    pub fn update_specs(&mut self, mut specs: Map<LineUid, FilterSpec>) -> Res<ShouldRender> {
        // Update "everything" filter.
        if let Some(mut spec) = specs.remove(&LineUid::Everything) {
            spec.unset_edited();
            self.everything = spec
        }
        // Update "catch-all" filter.
        if let Some(mut spec) = specs.remove(&LineUid::CatchAll) {
            spec.unset_edited();
            self.catch_all = spec
        }
        // Check all filters for changes.
        for filter in &mut self.filters {
            if let Some(mut spec) = specs.remove(&LineUid::Filter(filter.uid())) {
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
        self.send_refresh_filters();
        Ok(true)
    }
}

/// # Rendering
impl Filters {
    /// Renders the tabs for each filter.
    pub fn render_tabs(&self, model: &Model, active: Option<LineUid>) -> Html {
        // If there are no user-defined filters, only render the "everything" filter.
        if self.filters.is_empty() {
            self.everything
                .render_tab(model, active == Some(LineUid::Everything), false)
        } else {
            html! {
                <>
                    // Catch-all filter.
                    { self.catch_all.render_tab(model, active == Some(LineUid::CatchAll), false) }
                    <li class = style::class::tabs::li::get(false)>
                        <a
                            class = style::class::tabs::SEP
                        />
                    </li>
                    // Actual filters.
                    { for self.filters.iter().rev().enumerate().map(|(idx, filter)| {
                        let active = Some(LineUid::Filter(filter.uid())) == active;
                        filter.spec().render_tab(model, active, filter.edited())
                    } ) }
                    <li class = style::class::tabs::li::get(false)>
                        <a
                            class = style::class::tabs::SEP
                        />
                    </li>
                    // Everything filter.
                    { self.everything.render_tab(model, active == Some(LineUid::Everything), false)}
                </>
            }
        }
    }

    /// Renders the active filter.
    pub fn render_filter(&self, model: &Model, active: LineUid) -> Html {
        let (settings, filter_opt) = match active {
            LineUid::CatchAll => (
                self.catch_all
                    .render_settings(model, FilterRenderInfo::new_non_match()),
                None,
            ),
            LineUid::Everything => (
                self.everything
                    .render_settings(model, FilterRenderInfo::new_non_match()),
                None,
            ),
            LineUid::Filter(uid) => {
                if let Ok((idx, filter)) = self.get_filter(uid) {
                    let (is_first, is_last) = (idx == 0, idx + 1 == self.filters.len());
                    (
                        filter.spec().render_settings(
                            model,
                            FilterRenderInfo {
                                edited: filter.edited(),
                                is_first,
                                is_last,
                            },
                        ),
                        Some(filter),
                    )
                } else {
                    (html!(<a/>), None)
                }
            }
        };
        html! {
            <>
                <div class = style::class::filter::SEP/>
                <ul class = style::class::filter::LINE>
                    <li class = style::class::filter::BUTTONS_LEFT>
                    </li>
                    <li class = style::class::filter::line::CELL>
                        <a class = style::class::filter::line::SECTION_CELL>
                            { "Settings" }
                        </a>
                    </li>
                    <li class = style::class::filter::BUTTONS_RIGHT>
                        {
                            if let Some(uid) = active.filter_uid() {
                                buttons::close(
                                    model,
                                    "Delete the filter",
                                    move |_| msg::FiltersMsg::rm(uid)
                                )
                            } else {
                                html!(<a/>)
                            }
                        }
                    </li>
                </ul>
                { settings }
                {
                    if let Some(filter) = filter_opt {
                        render_subs(model, filter)
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
    fn render_tab(&self, model: &Model, active: bool, edited: bool) -> Html;

    /// Adds itself as a series to a chart.
    fn add_series_to(&self, spec: &chart::ChartSpec, chart: &JsValue);

    /// Renders the settings of a filter specification.
    fn render_settings(&self, model: &Model, info: FilterRenderInfo) -> Html;
}

impl FilterSpecExt for FilterSpec {
    fn render_tab(&self, model: &Model, active: bool, edited: bool) -> Html {
        let edited = edited || self.edited();
        let uid = self.uid();
        let (class, colorize) = style::class::tabs::footer_get(active, self.color());
        let inner = html! {
            <a
                class = class
                style = colorize
                onclick = model.link.callback(
                    move |_| msg::FooterMsg::toggle_tab(footer::FooterTab::filter(uid))
                )
            > {
                if edited {
                    format!("*{}*", self.name())
                } else {
                    self.name().into()
                }
            } </a>
        };
        html! {
            <li class = style::class::tabs::li::get(false)>
                { inner }
            </li>
        }
    }

    fn add_series_to(&self, spec: &chart::ChartSpec, chart: &JsValue) {
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
            series.fillOpacity = 0.01;
            return series;
        );
        use chart::axis::AxisExt;
        spec.x_axis().series_apply(&series, self.uid());
        spec.y_axis().series_apply(&series, self.uid());
        js!(@(no_return)
            var chart = @{chart};
            var series = @{series};
            chart.series.push(series);
            chart.scrollbarX.series.push(series);
            chart.invalidateRawData();
        );
    }

    fn render_settings(&self, model: &Model, info: FilterRenderInfo) -> Html {
        let uid = self.uid();

        let mut priority = html!(<a/>);

        if let Some(uid) = self.uid().filter_uid() {
            if !info.is_first || !info.is_last {
                priority = html! {
                    <ul class = style::class::filter::LINE>
                        <li class = style::class::filter::BUTTONS_LEFT/>

                        <li class = style::class::filter::line::CELL>
                            <a class = style::class::filter::line::SETTINGS_CELL>
                                { "match priority" }
                            </a>
                        </li>

                        <li class = style::class::filter::line::CELL>
                            <a class = style::class::filter::line::VAL_CELL>
                                {
                                    if !info.is_first {
                                        buttons::text(
                                            model,
                                            "increase (move left)",
                                            "try to match this filter BEFORE the one currently on its left",
                                            move |_| msg::FiltersMsg::move_filter(uid, true),
                                            style::class::filter::line::SETTINGS_BUTTON,
                                        )
                                    } else {
                                        html!(<a/>)
                                    }
                                }
                                {
                                    if !info.is_last {
                                        buttons::text(
                                            model,
                                            "decrease (move right)",
                                            "try to match this filter AFTER the one currently on its right",
                                            move |_| msg::FiltersMsg::move_filter(uid, false),
                                            style::class::filter::line::SETTINGS_BUTTON,
                                        )
                                    } else {
                                        html!(<a/>)
                                    }
                                }
                            </a>
                        </li>
                    </ul>
                }
            }
        }

        html!(
            <>
                <ul class = style::class::filter::LINE>
                    <li class = style::class::filter::BUTTONS_LEFT/>

                    <li class = style::class::filter::line::CELL>
                        <a class = style::class::filter::line::SETTINGS_CELL>
                            { "name" }
                        </a>
                    </li>
                    <li class = style::class::filter::line::CELL>
                        <a class = style::class::filter::line::VAL_CELL>
                            <input
                                type = "text"
                                onchange = model.link.callback(
                                    move |data| msg::FilterSpecMsg::change_name(uid, data)
                                )
                                value = self.name()
                                class = style::class::filter::line::SETTINGS_VALUE_CELL
                            />
                        </a>
                    </li>
                </ul>

                <ul class = style::class::filter::LINE>
                    <li class = style::class::filter::BUTTONS_LEFT/>

                    <li class = style::class::filter::line::CELL>
                        <a class = style::class::filter::line::SETTINGS_CELL>
                            { "color" }
                        </a>
                    </li>
                    <li class = style::class::filter::line::CELL>
                        <a class = style::class::filter::line::VAL_CELL>
                            <input
                                type = "color"
                                onchange = model.link.callback(
                                    move |data| msg::FilterSpecMsg::change_color(uid, data)
                                )
                                value = self.color().to_string()
                                class = style::class::filter::line::SETTINGS_VALUE_CELL
                            />
                        </a>
                    </li>
                </ul>

                { priority }
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
        Sub(sub) => {
            filter.replace(sub)?;
            filter.set_edited();
            Ok(true)
        }
        RmSub(uid) => {
            filter.remove(uid)?;
            filter.set_edited();
            Ok(true)
        }
    }
}

fn render_subs(model: &Model, filter: &Filter) -> Html {
    let uid = filter.uid();
    html! {
        <>
            <ul class = style::class::filter::SEP/>
            <ul class = style::class::filter::LINE>
                <li class = style::class::filter::BUTTONS_LEFT>
                </li>
                <li class = style::class::filter::line::CELL>
                    <a class = style::class::filter::line::SECTION_CELL>
                        { "Filters" }
                    </a>
                </li>
            </ul>
            { for filter.iter().map(|sub| {
                let sub_uid = sub.uid();
                html!(
                    <ul class = style::class::filter::LINE>
                        <li class = style::class::filter::BUTTONS_LEFT>
                            { buttons::close(
                                model,
                                "Remove the filter",
                                move |_| msg::FilterMsg::rm_sub(uid, sub_uid)
                            ) }
                        </li>
                        {{
                            let uid = filter.uid();
                            sub::prop_selector(
                                model,
                                sub,
                                move |res| {
                                    let res = res.map(|sub|
                                        msg::FilterMsg::update_sub(uid, sub)
                                    );
                                    err::msg_of_res(res)
                                }
                            )
                        }}
                        {{
                            let uid = filter.uid();
                            sub::render_cells(
                                model,
                                sub,
                                move |res| {
                                    let res = res.map(|sub|
                                        msg::FilterMsg::update_sub(uid, sub)
                                    );
                                    err::msg_of_res(res)
                                }
                            )
                        }}
                    </ul>
                )}
            ) }

            // Add button to add subfilters.
            <ul class = style::class::filter::LINE>
                <li class = style::class::filter::BUTTONS_LEFT>
                    {{
                        let uid = filter.uid();
                        buttons::add(
                            model,
                            "Add a new subfilter",
                            move |_| msg::FilterMsg::add_new(uid)
                        )
                    }}
                </li>
            </ul>
        </>
    }
}

mod sub {
    use super::*;
    use charts::filter::{FilterKind, SubFilter};

    pub fn render_cells<Update>(model: &Model, filter: &SubFilter, update: Update) -> Html
    where
        Update: Fn(Res<SubFilter>) -> Msg + 'static + Clone,
    {
        pub use charts::filter::sub::RawSubFilter::*;
        // Function that constructs a subfilter with the same UID.
        let uid = filter.uid();
        match filter.raw() {
            Size(filter) => size::render(model, filter, move |res| {
                let res = res.map(|raw| SubFilter::new(uid, Size(raw)));
                update(res)
            }),
            Label(filter) => label::render(model, filter, move |res| {
                let res = res.map(|raw| SubFilter::new(uid, Label(raw)));
                update(res)
            }),
            Loc(filter) => loc::render(model, filter, move |res| {
                let res = res.map(|raw| SubFilter::new(uid, Loc(raw)));
                update(res)
            }),
        }
    }

    pub fn prop_selector<Update>(model: &Model, filter: &SubFilter, update: Update) -> Html
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
                        onchange=model.link.callback(move |kind| {
                            let mut filter = filter.clone();
                            filter.change_kind(kind);
                            update(Ok(filter))
                        })
                    />
                </a>
            </li>
        }
    }

    mod size {
        use super::*;
        use charts::filter::{ord::Kind, SizeFilter};

        /// Renders a size filter given an update function.
        pub fn render<Update>(model: &Model, filter: &SizeFilter, update: Update) -> Html
        where
            Update: Fn(Res<SizeFilter>) -> Msg + Clone + 'static,
        {
            html! {
                <>
                    <li class=style::class::filter::line::CELL>
                        <a class=style::class::filter::line::CMP_CELL>
                            { kind_selector(model, filter, update.clone()) }
                        </a>
                    </li>
                    { render_values(model, filter, update) }
                </>
            }
        }

        fn parse_text_data(data: ChangeData) -> Res<usize> {
            use alloc_data::Parseable;
            match data {
                yew::html::ChangeData::Value(txt) => usize::parse(txt).map_err(|e| e.into()),
                err @ yew::html::ChangeData::Select(_) | err @ yew::html::ChangeData::Files(_) => {
                    bail!("unexpected text field update {:?}", err)
                }
            }
        }

        fn kind_selector<Update>(model: &Model, filter: &SizeFilter, update: Update) -> Html
        where
            Update: Fn(Res<SizeFilter>) -> Msg + 'static,
        {
            let selected = filter.cmp_kind();
            html! {
                <Select<Kind>
                    selected=Some(selected)
                    options=Kind::all()
                    onchange=model.link.callback(
                        move |kind| update(Ok(SizeFilter::default_of_cmp(kind)))
                    )
                />
            }
        }

        fn render_values<Update>(model: &Model, filter: &SizeFilter, update: Update) -> Html
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
                                    onchange=model.link.callback(move |data| update(
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
                                    ))
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
                                    onchange=model.link.callback(move |data| {
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
                                    })
                                >
                                </input>
                                <code> { "," } </code>
                                <input
                                    type="text"
                                    class=style::class::filter::line::TEXT_VALUE
                                    value=ub.to_string()
                                    onchange=model.link.callback(move |data| {
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
                                    })
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

    mod label {
        use super::*;
        use charts::filter::{
            label::{LabelPred, LabelSpec},
            LabelFilter,
        };

        pub fn render<Update>(model: &Model, filter: &LabelFilter, update: Update) -> Html
        where
            Update: Fn(Res<LabelFilter>) -> Msg + Clone + 'static,
        {
            let specs = filter.specs();

            html! {
                <>
                    <li class=style::class::filter::line::CELL>
                        <a class=style::class::filter::line::CMP_CELL>
                            { kind_selector(model, filter, update.clone()) }
                        </a>
                    </li>
                    <li class=style::class::filter::line::CELL>
                        <a class=style::class::filter::line::VAL_CELL>
                            <code> { "[" } </code>
                            {
                                for specs.iter().enumerate().map(
                                    |(index, spec)| {
                                        html! {
                                            // Attach to nothing, will become kid of the `<div>` above.
                                            <>
                                                { add_new(model, filter, update.clone(), index) }
                                                {{
                                                    let slf = filter.clone();
                                                    let update = update.clone();
                                                    render_spec(
                                                        model,
                                                        spec,
                                                        move |spec| update(
                                                            spec.map(
                                                                |spec| {
                                                                    let mut filter = slf.clone();
                                                                    filter.replace(index, spec);
                                                                    filter
                                                                }
                                                            )
                                                        )
                                                    )
                                                }}
                                            </>
                                        }
                                    }
                                )
                            }
                            { add_new(model, filter, update.clone(), specs.len()) }
                            <code> { "]" } </code>
                        </a>
                    </li>
                </>
            }
        }

        fn kind_selector<Update>(model: &Model, filter: &LabelFilter, update: Update) -> Html
        where
            Update: Fn(Res<LabelFilter>) -> Msg + 'static,
        {
            let (selected, specs) = (filter.pred(), filter.specs());
            let specs = specs.clone();
            html! {
                <Select<LabelPred>
                    selected=Some(selected)
                    options=LabelPred::all()
                    onchange=model.link.callback(
                        move |kind| update(Ok(LabelFilter::new(kind, specs.clone())))
                    )
                />
            }
        }

        ///
        pub fn add_new<Update>(
            model: &Model,
            filter: &LabelFilter,
            update: Update,
            index: usize,
        ) -> Html
        where
            Update: Fn(Res<LabelFilter>) -> Msg + Clone + 'static,
        {
            let slf = filter.clone();
            html! {
                <code
                    class=style::class::filter::line::ADD_LABEL
                    onclick=model.link.callback(move |_| {
                        let mut filter = slf.clone();
                        filter.insert(
                            index, LabelSpec::default()
                        );
                        update(Ok(filter))
                    })
                >{"+"}</code>
            }
        }

        fn render_spec<Update>(model: &Model, spec: &LabelSpec, update: Update) -> Html
        where
            Update: Fn(Res<LabelSpec>) -> Msg + 'static,
        {
            let value = match spec {
                LabelSpec::Value(value) => format!("{}", value),
                LabelSpec::Regex(regex) => format!("#\"{}\"#", regex),
                LabelSpec::Anything => "...".into(),
            };
            html! {
                <input
                    type="text"
                    class=style::class::filter::line::TEXT_VALUE
                    value=value
                    onchange=model.link.callback(move |data: ChangeData| update(
                        match data {
                            yew::html::ChangeData::Value(txt) => LabelSpec::new(txt),
                            err @ yew::html::ChangeData::Select(_) |
                            err @ yew::html::ChangeData::Files(_) => {
                                Err(err::Err::from(
                                    format!("unexpected text field update {:?}", err)
                                ))
                            }
                        }
                    ))
                />
            }
        }
    }

    mod loc {
        use super::*;
        use charts::filter::{
            loc::{LocPred, LocSpec},
            LocFilter,
        };

        pub fn render<Update>(model: &Model, filter: &LocFilter, update: Update) -> Html
        where
            Update: Fn(Res<LocFilter>) -> Msg + Clone + 'static,
        {
            let specs = filter.specs();

            html! {
                <>
                    <li class=style::class::filter::line::CELL>
                        <a class=style::class::filter::line::CMP_CELL>
                            { kind_selector(model, filter, update.clone()) }
                        </a>
                    </li>
                    <li class=style::class::filter::line::CELL>
                        <a class=style::class::filter::line::VAL_CELL>
                            <code> { "[" } </code>
                            {
                                for specs.iter().enumerate().map(
                                    |(index, spec)| {
                                        html! {
                                            // Attach to nothing, will become kid of the `<div>` above.
                                            <>
                                                { add_new(model, filter, update.clone(), index) }
                                                {{
                                                    let slf = filter.clone();
                                                    let update = update.clone();
                                                    render_spec(
                                                        model,
                                                        spec,
                                                        move |spec| update(
                                                            spec.map(
                                                                |spec| {
                                                                    let mut filter = slf.clone();
                                                                    filter.replace(index, spec);
                                                                    filter
                                                                }
                                                            )
                                                        )
                                                    )
                                                }}
                                            </>
                                        }
                                    }
                                )
                            }
                            { add_new(model, filter, update.clone(), specs.len()) }
                            <code> { "]" } </code>
                        </a>
                    </li>
                </>
            }
        }

        fn kind_selector<Update>(model: &Model, filter: &LocFilter, update: Update) -> Html
        where
            Update: Fn(Res<LocFilter>) -> Msg + 'static,
        {
            let (selected, specs) = (filter.pred(), filter.specs());
            let specs = specs.clone();
            html! {
                <Select<LocPred>
                    selected=Some(selected)
                    options=LocPred::all()
                    onchange=model.link.callback(
                        move |kind| update(Ok(LocFilter::new(kind, specs.clone())))
                    )
                />
            }
        }

        ///
        pub fn add_new<Update>(
            model: &Model,
            filter: &LocFilter,
            update: Update,
            index: usize,
        ) -> Html
        where
            Update: Fn(Res<LocFilter>) -> Msg + Clone + 'static,
        {
            let slf = filter.clone();
            html! {
                <code
                    class=style::class::filter::line::ADD_LABEL
                    onclick=model.link.callback(move |_| {
                        let mut filter = slf.clone();
                        filter.insert(
                            index, LocSpec::default()
                        );
                        update(Ok(filter))
                    })
                >{"+"}</code>
            }
        }

        fn render_spec<Update>(model: &Model, spec: &LocSpec, update: Update) -> Html
        where
            Update: Fn(Res<LocSpec>) -> Msg + 'static,
        {
            html! {
                <input
                    type="text"
                    class=style::class::filter::line::TEXT_VALUE
                    value=spec.to_string()
                    onchange=model.link.callback(move |data: ChangeData| update(
                        match data {
                            yew::html::ChangeData::Value(txt) => LocSpec::new(txt),
                            err @ yew::html::ChangeData::Select(_) |
                            err @ yew::html::ChangeData::Files(_) => {
                                Err(err::Err::from(
                                    format!("unexpected text field update {:?}", err)
                                ))
                            }
                        }
                    ))
                />
            }
        }
    }
}
