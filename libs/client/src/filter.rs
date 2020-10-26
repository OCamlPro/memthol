//! Filter-handling.

prelude! {}

pub use charts::filter::{Filter, FilterSpec, SubFilter};

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

/// Type of the reference filters.
///
/// Reference filters are the filters as they exist server-side.
pub type ReferenceFilters = FiltersExt<()>;
/// Type for filters.
///
/// Contains the reference filters.
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
            reference: ReferenceFilters::new_reference(to_model),
        }
    }

    /// Reference filters.
    pub fn reference_filters(&self) -> &ReferenceFilters {
        &self.reference
    }

    /// True if some modification took place w.r.t. the reference.
    pub fn edited(&self) -> bool {
        let reference = self.reference_filters();
        self.catch_all != reference.catch_all
            || self.everything != reference.everything
            || self.filters != reference.filters
    }

    /// True if the filter has been edited in some way.
    ///
    /// Returns true if the filter is unknown.
    pub fn is_filter_edited(&self, uid: uid::Line) -> bool {
        let reference = self.reference_filters();
        match uid {
            uid::Line::Everything => self.everything != reference.everything,
            uid::Line::CatchAll => self.catch_all != reference.catch_all,
            uid::Line::Filter(uid) => {
                let mut pair_opt: Option<(usize, &Filter)> = None;
                for (index, filter) in self.reference.filters.iter().enumerate() {
                    if filter.uid() == uid {
                        pair_opt = Some((index, filter))
                    }
                }

                let (ref_index, ref_filter) = if let Some((index, filter)) = pair_opt {
                    (index, filter)
                } else {
                    // Reference does not know this filter, necessarily new, => edited.
                    return true;
                };

                for (index, filter) in self.filters.iter().enumerate() {
                    if filter.uid() == uid {
                        return ref_index != index || ref_filter != filter;
                    }
                }

                // Filter is known by the reference, but not the current filters.
                return true;
            }
        }
    }
}

impl<T> FiltersExt<T> {
    /// Retrieves a filter from its UID.
    pub fn get_filter(&self, uid: uid::Filter) -> Res<(usize, &Filter)> {
        for (index, filter) in self.filters.iter().enumerate() {
            if filter.uid() == uid {
                return Ok((index, filter));
            }
        }
        bail!("unknown filter uid #{}", uid)
    }

    /// Retrieves a filter from its UID, mutable version.
    fn get_filter_mut(&mut self, uid: uid::Filter) -> Res<(usize, &mut Filter)> {
        for (index, filter) in self.filters.iter_mut().enumerate() {
            if filter.uid() == uid {
                return Ok((index, filter));
            }
        }
        bail!("unknown filter uid #{}", uid)
    }

    /// Gives mutable access to a filter specification.
    pub fn get_spec_mut(&mut self, uid: uid::Line) -> Res<&mut FilterSpec> {
        match uid {
            uid::Line::CatchAll => Ok(&mut self.catch_all),
            uid::Line::Everything => Ok(&mut self.everything),
            uid::Line::Filter(uid) => self
                .get_filter_mut(uid)
                .map(|(_index, filter)| filter.spec_mut()),
        }
    }

    /// Gives mutable access to a filter specification.
    pub fn get_spec(&self, uid: uid::Line) -> Res<&FilterSpec> {
        match uid {
            uid::Line::CatchAll => Ok(&self.catch_all),
            uid::Line::Everything => Ok(&self.everything),
            uid::Line::Filter(uid) => self.get_filter(uid).map(|(_index, filter)| filter.spec()),
        }
    }

    /// Pushes a filter.
    pub fn push(&mut self, filter: Filter) {
        self.filters.push(filter)
    }

    /// Removes a filter.
    ///
    /// Returns the uid of
    /// - the filter after `uid`, if any
    /// - otherwise the filter before `uid`, if any.
    pub fn remove(&mut self, uid: uid::Filter) -> Res<Option<uid::Filter>> {
        let (index, _) = self.get_filter(uid)?;
        self.filters.remove(index);
        if index < self.filters.len() {
            Ok(Some(self.filters[index].uid()))
        } else if 0 < index {
            debug_assert!(index - 1 < self.filters.len());
            Ok(Some(self.filters[index - 1].uid()))
        } else {
            Ok(None)
        }
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

    /// Yields an iterator ovec the filter specifications.
    pub fn specs_iter(&self) -> impl Iterator<Item = &FilterSpec> + Clone {
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
                let catch_all = self.catch_all.clone();

                let everything = self.everything.clone();

                let mut filters = Vec::with_capacity(self.filters.len());

                for filter in &mut self.filters {
                    filters.push(filter.clone());
                }
                self.to_model.emit(
                    msg::to_server::FiltersMsg::update_all(everything, filters, catch_all).into(),
                );

                self.save_all();

                self.to_model.emit(msg::ChartsMsg::refresh_filters());

                Ok(true)
            }

            Rm(uid) => self.rm_filter(uid),

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
    pub fn change_name(&mut self, uid: uid::Line, new_name: ChangeData) -> Res<()> {
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

        Ok(())
    }

    /// Changes the color of a filter.
    pub fn change_color(&mut self, uid: uid::Line, new_color: ChangeData) -> Res<()> {
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
                self.reference.catch_all = catch_all.clone();
                self.reference.everything = everything.clone();
                self.reference.filters = filters.clone();
                self.catch_all = catch_all;
                self.everything = everything;
                for filter in &self.filters {
                    let uid = filter.uid();
                    if filters.iter().all(|filter| filter.uid() != uid) {
                        self.to_model
                            .emit(msg::FooterMsg::toggle_tab(uid::Line::Everything));
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
    pub fn add_filter(&mut self, filter: Filter) -> Res<ShouldRender> {
        let uid = filter.uid();
        self.filters.push(filter);
        self.to_model
            .emit(msg::FooterMsg::toggle_tab(footer::FooterTab::filter(
                uid::Line::Filter(uid),
            )));
        Ok(true)
    }

    /// Removes a filter from the map.
    pub fn rm_filter(&mut self, uid: uid::Filter) -> Res<ShouldRender> {
        let now_active = self.remove(uid)?;
        self.to_model.emit(msg::FooterMsg::toggle_tab(
            now_active
                .map(uid::Line::from)
                .unwrap_or(uid::Line::Everything),
        ));
        Ok(true)
    }

    /// Updates the specifications of the filters in the map.
    ///
    /// - triggers a refresh of the filters of all the charts.
    pub fn update_specs(&mut self, mut specs: BTMap<uid::Line, FilterSpec>) -> Res<ShouldRender> {
        // Update "everything" filter.
        if let Some(spec) = specs.remove(&uid::Line::Everything) {
            self.everything = spec
        }
        // Update "catch-all" filter.
        if let Some(spec) = specs.remove(&uid::Line::CatchAll) {
            self.catch_all = spec
        }
        // Check all filters for changes.
        for filter in &mut self.filters {
            if let Some(spec) = specs.remove(&uid::Line::Filter(filter.uid())) {
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

/// Applies an update to a filter.
fn filter_update(filter: &mut Filter, msg: msg::FilterMsg) -> Res<ShouldRender> {
    use msg::FilterMsg::*;
    match msg {
        AddNew => {
            let sub = SubFilter::default();
            filter.insert(sub)?;
            Ok(true)
        }
        Sub(sub) => {
            filter.replace(sub)?;
            Ok(true)
        }
        RmSub(uid) => {
            filter.remove(uid)?;
            Ok(true)
        }
    }
}
