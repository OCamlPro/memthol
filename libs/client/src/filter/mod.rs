//! Filter-handling.

use crate::base::*;

use charts::filter::{Filter, FilterSpec, FilterUid, SubFilterUid};

/// Stores all the filters.
pub struct Filters {
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
    pub fn new() -> Self {
        Filters {
            active: None,
            catch_all: FilterSpec::new_catch_all(),
            filters: Map::new(),
            deleted: vec![],
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
        html! {
            <>
            </>
        }
    }
}

/// Extension trait for `FilterSpec`.
pub trait FilterSpecExt {
    /// Renders a spec as a tab.
    fn render_tab(&self, active: bool) -> Html;
}

impl FilterSpecExt for FilterSpec {
    fn render_tab(&self, active: bool) -> Html {
        let uid = self.uid();
        let (class, colorize) = style::class::tabs::footer_get(active, self.color());
        html! {
            <li class={ style::class::tabs::li::get(false) }>
                <a
                    class={class}
                    style={colorize}
                    // onclick=|_| msg::FooterMsg::toggle_tab(tab)
                > {
                    self.name()
                } </a>
            </li>
        }
    }
}
