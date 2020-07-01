//! Filter-handling.

use crate::common::*;

pub use charts::filter::{Filter, FilterSpec};

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

pub type ReferenceFilters = FiltersExt<()>;
pub type Filters = FiltersExt<ReferenceFilters>;

/// Stores all the filters.
pub struct FiltersExt<Reference> {
    /// Sends messages to the model.
    to_model: Callback<Msg>,
    /// Catch-all filter.
    pub catch_all: FilterSpec,
    /// Everything filter.
    pub everything: FilterSpec,
    /// Actual filters.
    pub filters: Vec<Filter>,
    /// True if a filter was (re)moved and the server was not notified.
    edited: bool,
    /// Reference filters, if any.
    reference: Reference,
}

impl ReferenceFilters {
    /// Constructor.
    fn new_reference(to_model: Callback<Msg>) -> Self {
        FiltersExt {
            to_model,
            catch_all: FilterSpec::new_catch_all(),
            everything: FilterSpec::new_everything(),
            filters: Vec::new(),
            edited: false,
            reference: (),
        }
    }
}

impl Filters {
    /// Constructor.
    pub fn new(to_model: Callback<Msg>) -> Self {
        Filters {
            to_model: to_model.clone(),
            catch_all: FilterSpec::new_catch_all(),
            everything: FilterSpec::new_everything(),
            filters: Vec::new(),
            edited: false,
            reference: ReferenceFilters::new_reference(to_model),
        }
    }

    /// Reference filters.
    pub fn reference_filters(&self) -> &ReferenceFilters {
        &self.reference
    }
}

impl<T> FiltersExt<T> {
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
    pub fn get_filter(&self, uid: FilterUid) -> Res<(usize, &Filter)> {
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

    /// Gives mutable access to a filter specification.
    pub fn get_spec(&self, uid: LineUid) -> Res<&FilterSpec> {
        match uid {
            LineUid::CatchAll => Ok(&self.catch_all),
            LineUid::Everything => Ok(&self.everything),
            LineUid::Filter(uid) => self.get_filter(uid).map(|(_index, filter)| filter.spec()),
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

    pub fn specs_iter(&self) -> impl Iterator<Item = &FilterSpec> {
        Some(&self.everything)
            .into_iter()
            .chain(self.filters.iter().map(Filter::spec))
            .chain(
                if self.filters.is_empty() {
                    None
                } else {
                    Some(&self.catch_all)
                }
                .into_iter(),
            )
    }

    /// True if there are no filters besides the built-in ones.
    pub fn has_user_filters(&self) -> bool {
        !self.filters.is_empty()
    }

    /// The filters to render.
    pub fn filters_to_render(&self) -> (&FilterSpec, Option<(&FilterSpec, &[Filter])>) {
        (
            &self.everything,
            if self.has_user_filters() {
                Some((&self.catch_all, &self.filters))
            } else {
                None
            },
        )
    }
}

impl Filters {
    /// Propagates all filters to the reference filters.
    fn save_all(&mut self) {
        let reference = &mut self.reference;
        reference.everything = self.everything.clone();
        reference.catch_all = self.catch_all.clone();
        reference.filters = self.filters.clone();
    }

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

                self.save_all();

                self.to_model.emit(msg::ChartsMsg::refresh_filters());

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
}

/// # Internal message handling
impl<T> FiltersExt<T> {
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
                self.edited = false;
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
}

impl<T> FiltersExt<T> {
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
