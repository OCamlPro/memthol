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
    /// Renders a spec.
    fn render(&self) -> Html;
}

impl FilterSpecExt for FilterSpec {
    fn render(&self) -> Html {
        html! {
            <>
            </>
        }
    }
}
