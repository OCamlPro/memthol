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
        let uid = if let Some(uid) = filter.uid() {
            uid
        } else {
            bail!("trying to push a catch-all filter as a regular filter")
        };
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

    /// Applies a function to all the filters, including the deleted filters.
    ///
    /// The function is given a boolean flag that's true when the filter was deleted.
    pub fn iter_apply<F>(&self, mut f: F) -> Res<()>
    where
        F: FnMut(&Filter, bool) -> Res<()>,
    {
        for filter in self.filters.values() {
            f(filter, false)?
        }
        for filter in &self.deleted {
            f(filter, true)?
        }
        Ok(())
    }
}

/// # Message updates
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

    /// Applies an operation from the server.
    pub fn server_update(&mut self, msg: msg::from_server::FiltersMsg) -> Res<ShouldRender> {
        use msg::from_server::FiltersMsg::*;
        match msg {
            Rm(uid) => {
                self.remove(uid)?;
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
            spec.set_name(new_name)
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
            spec.set_color(new_color)
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

/// # Rendering
impl Filters {
    /// Renders the tabs for each filter.
    pub fn render_tabs(&self) -> Html {
        html! {
            <>
                // Actual filters.
                { for self.filters.values().map(|filter| {
                    let active = filter.uid() == self.active;
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
        html!(
            <>
                <ul class=style::class::filter::LINE>
                    <li class=style::class::filter::BUTTONS/>
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
}
