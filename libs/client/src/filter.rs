//! Filter-handling.

prelude! {}

pub use charts::filter::{stats::AllFilterStats, Filter, FilterSpec, SubFilter};

/// Stores filter states.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FilterStates {
    /// Catch-all filter.
    pub catch_all: FilterSpec,
    /// Everything filter.
    pub everything: FilterSpec,
    /// Custom filters.
    pub filters: Vec<Filter>,
}
impl Default for FilterStates {
    fn default() -> Self {
        Self {
            catch_all: FilterSpec::new_catch_all(),
            everything: FilterSpec::new_everything(),
            filters: vec![],
        }
    }
}
impl FilterStates {
    /// Returns the current index and state for a filter from its UID.
    pub fn get_filter(&self, uid: uid::Filter) -> Res<(usize, &Filter)> {
        self.filters
            .iter()
            .enumerate()
            .find_map(|(index, filter)| {
                if filter.uid() == uid {
                    Some((index, filter))
                } else {
                    None
                }
            })
            .ok_or_else(|| format!("unknown filter uid #{}", uid).into())
    }
    /// Returns the current index and mutable state for a filter from its UID.
    pub fn get_filter_mut(&mut self, uid: uid::Filter) -> Res<(usize, &mut Filter)> {
        self.filters
            .iter_mut()
            .enumerate()
            .find_map(|(index, filter)| {
                if filter.uid() == uid {
                    Some((index, filter))
                } else {
                    None
                }
            })
            .ok_or_else(|| format!("unknown filter uid #{}", uid).into())
    }

    /// Returns the current index and state for a filter from its UID.
    pub fn get(&self, uid: uid::Line) -> Res<(Option<usize>, &FilterSpec)> {
        match uid {
            uid::Line::CatchAll => Ok((None, &self.catch_all)),
            uid::Line::Everything => Ok((None, &self.everything)),
            uid::Line::Filter(uid) => self
                .get_filter(uid)
                .map(|(idx, filter)| (Some(idx), filter.spec())),
        }
    }
    /// Returns the current index and mutable state for a filter from its UID.
    pub fn get_mut(&mut self, uid: uid::Line) -> Res<(Option<usize>, &mut FilterSpec)> {
        match uid {
            uid::Line::CatchAll => Ok((None, &mut self.catch_all)),
            uid::Line::Everything => Ok((None, &mut self.everything)),
            uid::Line::Filter(uid) => self
                .get_filter_mut(uid)
                .map(|(idx, filter)| (Some(idx), filter.spec_mut())),
        }
    }
    /// The filters to render.
    pub fn filters_to_render(&self) -> (&FilterSpec, Option<(&FilterSpec, &[Filter])>) {
        let has_user_filters = !self.filters.is_empty();
        (
            &self.everything,
            if has_user_filters {
                Some((&self.catch_all, &self.filters))
            } else {
                None
            },
        )
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

    /// Applies a function to all specification filters.
    ///
    /// Typically used when refreshing filters for a chart.
    pub fn specs_apply<F>(&self, mut f: F) -> Res<()>
    where
        F: FnMut(&FilterSpec) -> Res<()>,
    {
        for spec in self.specs_iter() {
            f(spec)?
        }
        Ok(())
    }
}

/// Enforces a type-level distinction between current and reference filters.
#[derive(Debug, Clone, Copy)]
pub struct F<'a, T> {
    states: &'a FilterStates,
    _phantom: std::marker::PhantomData<T>,
}
base::implement! {
    impl F<'a, T>, with ('a, T) {
        Deref {
            to &'a FilterStates => |&self| &self.states,
        }
    }
}
/// Type marker for current filters.
#[derive(Debug, Clone, Copy)]
pub enum FCurrent {}
/// Current filter states.
pub type Current<'a> = F<'a, FCurrent>;
/// Type marker for reference filters.
#[derive(Debug, Clone, Copy)]
pub enum FReference {}
/// Reference filter states.
pub type Reference<'a> = F<'a, FReference>;

/// Stores the current/old filter states, as well as the filter statistics.
pub struct FilterInfo {
    /// Link to the model.
    pub link: Link,
    /// Current and reference states.
    pub states: Memory<FilterStates>,
    /// Filter statistics for the reference filter states.
    pub reference_stats: AllFilterStats,
}

impl FilterInfo {
    /// Constructor.
    pub fn new(link: Link) -> Self {
        let states = Memory::new(FilterStates::default());
        Self {
            link,
            states,
            reference_stats: AllFilterStats::new(),
        }
    }

    /// Stats accessor (stats are for the reference filters).
    pub fn ref_stats(&self) -> &AllFilterStats {
        &self.reference_stats
    }
    /// Updates the reference stats.
    pub fn update_ref_stats(&mut self, stats: AllFilterStats) {
        self.reference_stats = stats
    }

    /// Current filter accessor.
    pub fn current(&self) -> Current {
        F {
            states: &self.states.get(),
            _phantom: std::marker::PhantomData,
        }
    }
    /// Reference filter accessor.
    pub fn reference(&self) -> Reference {
        F {
            states: &self.states.reference(),
            _phantom: std::marker::PhantomData,
        }
    }

    /// True if the current filter states is different from the reference filter states.
    pub fn has_changed(&self) -> bool {
        self.states.has_changed()
    }

    /// True if the given filter has been edited in some way.
    ///
    /// Returns true if the filter is unknown in the reference states.
    pub fn is_filter_edited(&self, uid: uid::Line) -> bool {
        let (current, reference) = (self.states.get(), self.states.reference());
        match uid {
            uid::Line::Everything => current.everything != reference.everything,
            uid::Line::CatchAll => current.catch_all != reference.catch_all,
            uid::Line::Filter(uid) => {
                let (ref_index, ref_filter) = if let Ok((idx, filter)) = reference.get_filter(uid) {
                    (idx, filter)
                } else {
                    return true;
                };

                match current.get_filter(uid) {
                    Ok((cur_index, cur_filter)) => {
                        ref_index != cur_index || ref_filter != cur_filter
                    }
                    Err(_) => return true,
                }
            }
        }
    }

    /// Returns the current index and state for a filter from its UID.
    fn get_filter(&self, uid: uid::Filter) -> Res<(usize, &Filter)> {
        self.states
            .get()
            .filters
            .iter()
            .enumerate()
            .find_map(|(index, filter)| {
                if filter.uid() == uid {
                    Some((index, filter))
                } else {
                    None
                }
            })
            .ok_or_else(|| format!("unknown filter uid #{}", uid).into())
    }
    /// Returns the current index and mutable state for a filter from its UID.
    fn get_filter_mut(&mut self, uid: uid::Filter) -> Res<(usize, &mut Filter)> {
        self.states
            .get_mut()
            .filters
            .iter_mut()
            .enumerate()
            .find_map(|(index, filter)| {
                if filter.uid() == uid {
                    Some((index, filter))
                } else {
                    None
                }
            })
            .ok_or_else(|| format!("unknown filter uid #{}", uid).into())
    }
    /// Removes the filter and returns its index before removing.
    fn rm_filter(&mut self, uid: uid::Filter) -> Res<(usize, Filter)> {
        let (index, _) = self.get_filter(uid)?;
        let filter = self.states.get_mut().filters.remove(index);
        Ok((index, filter))
    }

    /// Returns the current index and state for a filter from its UID.
    fn _get(&self, uid: uid::Line) -> Res<(Option<usize>, &FilterSpec)> {
        match uid {
            uid::Line::CatchAll => Ok((None, &self.states.get().catch_all)),
            uid::Line::Everything => Ok((None, &self.states.get().everything)),
            uid::Line::Filter(uid) => self
                .get_filter(uid)
                .map(|(idx, filter)| (Some(idx), filter.spec())),
        }
    }
    /// Returns the current index and mutable state for a filter from its UID.
    fn get_mut(&mut self, uid: uid::Line) -> Res<(Option<usize>, &mut FilterSpec)> {
        match uid {
            uid::Line::CatchAll => Ok((None, &mut self.states.get_mut().catch_all)),
            uid::Line::Everything => Ok((None, &mut self.states.get_mut().everything)),
            uid::Line::Filter(uid) => self
                .get_filter_mut(uid)
                .map(|(idx, filter)| (Some(idx), filter.spec_mut())),
        }
    }
}

impl FilterInfo {
    /// Removes a filter from the current filter states.
    fn remove(&mut self, uid: uid::Filter) -> Res<ShouldRender> {
        // Find the index of the filter to remove.
        let (index, _) = self.rm_filter(uid)?;

        let current = self.states.get_mut();

        // Find the uid of the filter after the one we removed; if none, find the filter before the
        // one we removed, if any.
        let new_active = if index < current.filters.len() {
            Some(current.filters[index].uid())
        } else if 0 < index {
            debug_assert!(index - 1 < current.filters.len());
            Some(current.filters[index - 1].uid())
        } else {
            None
        };

        self.link.send_message(msg::FooterMsg::toggle_tab(
            new_active
                .map(uid::Line::from)
                .unwrap_or(uid::Line::Everything),
        ));
        Ok(true)
    }

    /// Changes the name of a filter.
    fn change_name(&mut self, uid: uid::Line, new_name: ChangeData) -> Res<()> {
        let new_name = match new_name {
            yew::html::ChangeData::Value(txt) => txt,
            err @ yew::html::ChangeData::Select(_) | err @ yew::html::ChangeData::Files(_) => {
                bail!("unexpected text field update {:?}", err)
            }
        };
        if new_name.is_empty() {
            bail!("filter names cannot be empty")
        }
        let (_, spec) = self
            .get_mut(uid)
            .chain_err(|| "while updating a filter's name")?;
        spec.set_name(new_name);

        Ok(())
    }

    /// Changes the color of a filter.
    fn change_color(&mut self, uid: uid::Line, new_color: ChangeData) -> Res<()> {
        let new_color = match new_color {
            yew::html::ChangeData::Value(new_color) => charts::color::Color::from_str(new_color)
                .chain_err(|| "while changing the color of a filter")?,
            err @ yew::html::ChangeData::Select(_) | err @ yew::html::ChangeData::Files(_) => {
                bail!("unexpected text field update {:?}", err)
            }
        };

        let (_, spec) = self
            .get_mut(uid)
            .chain_err(|| "while updating the color of a filter")?;
        spec.set_color(new_color);

        Ok(())
    }

    /// Applies an update to a filter.
    fn filter_update(filter: &mut Filter, msg: FilterMsg) -> Res<ShouldRender> {
        match msg {
            FilterMsg::Sub(sub) => {
                filter.replace(sub)?;
                Ok(true)
            }
            FilterMsg::RmSub(uid) => {
                filter.remove(uid)?;
                Ok(true)
            }
        }
    }
}

impl FilterInfo {
    /// Handles a message.
    pub fn update(&mut self, msg: Msg) -> Res<ShouldRender> {
        match msg {
            Msg::Save => {
                if !self.states.has_changed() {
                    return Ok(false);
                }

                // Send current version to the server.
                let current = self.states.get();
                self.link
                    .send_message(msg::to_server::FiltersMsg::update_all(
                        current.everything.clone(),
                        current.filters.clone(),
                        current.catch_all.clone(),
                    ));

                // Overwrite reference to be the current state.
                self.states.overwrite_reference();

                // Model must now refresh its filters.
                self.link.send_message(msg::ChartsMsg::refresh_filters());

                Ok(true)
            }

            Msg::Rm(uid) => self.remove(uid),

            Msg::FilterSpec {
                uid,
                msg: SpecMsg::ChangeName(new_name),
            } => {
                self.change_name(uid, new_name)?;
                Ok(true)
            }
            Msg::FilterSpec {
                uid,
                msg: SpecMsg::ChangeColor(new_color),
            } => {
                self.change_color(uid, new_color)?;
                Ok(true)
            }
            Msg::Filter { uid, msg } => {
                let (_index, filter) = self.get_filter_mut(uid)?;
                Self::filter_update(filter, msg)
            }
            Msg::Move { uid, left } => {
                let (index, _) = self.get_filter(uid)?;
                let current = self.states.get_mut();

                let to_swap = if current.filters.len() < 2 {
                    // There's at most one filter, no move to do.
                    //
                    // This first check guarantees that the third `if` below is legal.
                    None
                } else if left && index > 0 {
                    // Going left and not the left-most index, swap.
                    Some((index - 1, index))
                } else if !left && index < current.filters.len() - 1 {
                    // Going right and not the right-most index, swap.
                    Some((index, index + 1))
                } else {
                    // Everything else causes no move.
                    None
                };
                if let Some((i_1, i_2)) = to_swap {
                    current.filters.swap(i_1, i_2);
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
        }
    }

    /// Applies an operation from the server.
    pub fn server_update(&mut self, msg: msg::from_server::FiltersMsg) -> Res<ShouldRender> {
        use msg::from_server::FiltersMsg::*;
        match msg {
            Add(filter) => {
                let uid = filter.uid();
                self.states.get_mut().filters.push(filter);
                self.link
                    .send_message(msg::FooterMsg::toggle_tab(footer::FooterTab::filter(
                        uid::Line::Filter(uid),
                    )));
                Ok(true)
            }
            AddSub(uid, subfilter) => {
                let (_, filter) = self.states.get_mut().get_filter_mut(uid)?;
                filter.insert(subfilter)?;
                Ok(true)
            }
            Revert {
                everything,
                filters,
                catch_all,
            } => {
                for filter in &self.current().filters {
                    let uid = filter.uid();
                    if filters.iter().all(|filter| filter.uid() != uid) {
                        self.link
                            .send_message(msg::FooterMsg::toggle_tab(uid::Line::Everything));
                    }
                }
                self.states.set_both(FilterStates {
                    everything,
                    filters,
                    catch_all,
                });
                Ok(true)
            } // UpdateSpecs(specs) => self.update_specs(specs),
        }
    }
}

/// Operations over filters.
#[derive(Debug)]
pub enum Msg {
    /// Updates a filter on the server.
    Save,
    /// Removes a filter.
    Rm(uid::Filter),
    /// A message for a specific filter specification.
    FilterSpec {
        /// Uid of the filter.
        uid: uid::Line,
        /// Message.
        msg: SpecMsg,
    },
    /// A message for a specific filter.
    Filter {
        /// UID of the iflter.
        uid: uid::Filter,
        /// Message.
        msg: FilterMsg,
    },
    /// Moves a filter left or right.
    Move {
        /// Filter UID.
        uid: uid::Filter,
        /// Move left iff true.
        left: bool,
    },
}

impl Msg {
    /// Updates a filter on the server.
    pub fn save() -> Msg {
        Self::Save.into()
    }
    /// Removes a filter.
    pub fn rm(uid: uid::Filter) -> Msg {
        Self::Rm(uid).into()
    }
    /// A message for a specific filter specification.
    pub fn filter_spec(uid: uid::Line, msg: SpecMsg) -> Msg {
        Self::FilterSpec { uid, msg }.into()
    }
    /// A message for a specific filter.
    pub fn filter(uid: uid::Filter, msg: FilterMsg) -> Msg {
        Self::Filter { uid, msg }.into()
    }
    /// Moves a filter left or right.
    pub fn move_filter(uid: uid::Filter, left: bool) -> Msg {
        Self::Move { uid, left }.into()
    }
}

/// An action over the specification of a filter.
#[derive(Debug)]
pub enum SpecMsg {
    /// Changes the name of a filter.
    ChangeName(ChangeData),
    /// Changes the color of a filter.
    ChangeColor(ChangeData),
}
impl SpecMsg {
    /// Changes the name of a filter.
    pub fn change_name(uid: uid::Line, new_name: ChangeData) -> Msg {
        Msg::filter_spec(uid, Self::ChangeName(new_name)).into()
    }
    /// Changes the color of a filter.
    pub fn change_color(uid: uid::Line, new_color: ChangeData) -> Msg {
        Msg::filter_spec(uid, Self::ChangeColor(new_color)).into()
    }
}

/// A message for a specific filter.
#[derive(Debug)]
pub enum FilterMsg {
    /// Updates a subfilter.
    Sub(filter::SubFilter),
    /// Removes a subfilter.
    RmSub(uid::SubFilter),
}
impl FilterMsg {
    /// Updates a subfilter.
    pub fn update_sub(uid: uid::Filter, sub: filter::SubFilter) -> msg::Msg {
        Msg::filter(uid, Self::Sub(sub.into())).into()
    }
    /// Removes a subfilter.
    pub fn rm_sub(uid: uid::Filter, sub_uid: uid::SubFilter) -> msg::Msg {
        Msg::filter(uid, Self::RmSub(sub_uid)).into()
    }
}

base::implement! {
    impl msg::Msg {
        From {
            from Msg => |msg| msg::Msg::Filter(msg),
        }
    }

    impl Msg {
        Display {
            |&self, fmt| match self {
                Self::Save => write!(fmt, "save"),
                Self::Rm(f_uid) => write!(fmt, "rm {}", f_uid),
                Self::FilterSpec { uid, msg } => write!(fmt, "filter spec {}, {}", uid, msg),
                Self::Filter { uid, msg } => write!(fmt, "filter {}, {}", uid, msg),
                Self::Move { uid, left } => write!(fmt, "move {} ({})", uid, left),
            }
        }
    }
    impl SpecMsg {
        Display {
            |&self, fmt| match self {
                Self::ChangeName(_) => write!(fmt, "change name"),
                Self::ChangeColor(_) => write!(fmt, "change color"),
            }
        }
    }
    impl FilterMsg {
        Display {
            |&self, fmt| match self {
                Self::Sub(_) => write!(fmt, "subfilter update"),
                Self::RmSub(_) => write!(fmt, "remove subfilter"),
            }
        }
    }
}
