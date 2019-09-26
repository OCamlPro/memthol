//! Data filtering.
//!
//! All types in this module implement `serde`'s `Serialize` and `Deserialize` traits.

use crate::base::*;

pub mod label;
pub mod ord;
mod spec;
pub mod sub;

pub use label::LabelFilter;
use ord::OrdFilter;
pub use spec::FilterSpec;
pub use sub::SubFilter;
pub use uid::{FilterUid, SubFilterUid};

/// A filter over allocation sizes.
pub type SizeFilter = OrdFilter<usize>;

/// Function(s) a filter must implement.
pub trait FilterExt<Data>: Sized
where
    Data: ?Sized,
{
    /// Applies the filter to some allocation data.
    fn apply(&self, alloc_data: &Data) -> bool;
}

/// Filter comparison kind.
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum CmpKind {
    /// Ordered comparison.
    Ord(ord::Kind),
    /// Label comparison.
    Label(label::Kind),
}
impl CmpKind {
    /// Ordered comparison constructor.
    pub fn new_ord(kind: ord::Kind) -> Self {
        Self::Ord(kind)
    }
    /// Label comparison constructor.
    pub fn new_label(kind: label::Kind) -> Self {
        Self::Label(kind)
    }
}
impl fmt::Display for CmpKind {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Ord(kind) => write!(fmt, "{}", kind),
            Self::Label(kind) => write!(fmt, "{}", kind),
        }
    }
}

/// Filter kind.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum FilterKind {
    /// Size filter.
    Size,
    /// Label filter.
    Label,
}
impl fmt::Display for FilterKind {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Size => write!(fmt, "size"),
            Self::Label => write!(fmt, "labels"),
        }
    }
}

impl FilterKind {
    pub fn all() -> Vec<FilterKind> {
        vec![FilterKind::Size, FilterKind::Label]
    }
}

/// A list of filters.
///
/// Aggregates the following:
///
/// - a "catch all" [`FilterSpec`], the specification of the points that no filter catches;
/// - a list of [`Filter`]s;
/// - a memory from allocation UIDs to filter UIDs that tells which filter takes care of some
///     allocation.
///
/// The point of the memory is that it is not possible to know which filter takes care of a given
/// allocation after the first time we saw that allocation. Which we want to know when registering
/// the death of an allocation. The reason we don't know is that new filters might have been
/// introduced or some filters may have changed. Hence the filter assigned for this allocation a
/// while ago may not be the one we would assign now.
///
/// [`FilterSpec`]: struct.FilterSpec.html (The FilterSpec struct)
/// [`Filter`]: struct.Filter.html (The Filter struct)
#[derive(Debug, Clone)]
pub struct Filters {
    /// The specification of the catch-all filter.
    catch_all: FilterSpec,
    /// The actual list of filters.
    filters: Vec<Filter>,
    /// Remembers which filter is responsible for an allocation.
    memory: Map<AllocUid, FilterUid>,
}

impl Filters {
    /// Constructor.
    pub fn new() -> Self {
        Filters {
            filters: vec![],
            catch_all: FilterSpec::new_catch_all(),
            memory: Map::new(),
        }
    }

    /// Length of the list of filters.
    pub fn len(&self) -> usize {
        self.filters.len()
    }

    /// The list of active filters.
    pub fn filters(&self) -> &Vec<Filter> {
        &self.filters
    }

    /// Filter specification mutable accessor.
    pub fn get_spec_mut(&mut self, uid: Option<FilterUid>) -> Res<&mut FilterSpec> {
        if let Some(uid) = uid {
            self.get_mut(uid).map(|(_, filter)| filter.spec_mut())
        } else {
            Ok(&mut self.catch_all)
        }
    }

    /// Filter mutable accessor.
    ///
    /// - returns the index of the filter and the filter itself;
    /// - fails if the filter UID is unknown.
    pub fn get_mut(&mut self, uid: FilterUid) -> Res<(usize, &mut Filter)> {
        for (index, filter) in self.filters.iter_mut().enumerate() {
            if filter.uid() == uid {
                return Ok((index, filter));
            }
        }
        bail!("cannot access filter with unknown UID #{}", uid)
    }

    /// Iterator over the filters.
    pub fn iter(&self) -> impl Iterator<Item = (index::Filter, &Filter)> {
        self.filters
            .iter()
            .enumerate()
            .map(|(index, filter)| (index::Filter::new(index), filter))
    }

    /// Mutable iterator over the filters.
    pub fn iter_mut(&mut self) -> impl Iterator<Item = (index::Filter, &mut Filter)> {
        self.filters
            .iter_mut()
            .enumerate()
            .map(|(index, filter)| (index::Filter::new(index), filter))
    }

    /// Overwrites a filter.
    pub fn set(&mut self, index: index::Filter, filter: Filter) {
        self.filters[*index.deref()] = filter
    }

    /// Remembers that an allocation is handled by some filter.
    fn remember(memory: &mut Map<AllocUid, FilterUid>, alloc: AllocUid, filter: FilterUid) {
        let prev = memory.insert(alloc, filter);
        if prev.is_some() {
            panic!("filter memory collision")
        }
    }

    /// Searches for a filter that matches on the input allocation.
    pub fn find_match(&mut self, alloc: &Alloc) -> Option<uid::FilterUid> {
        for filter in &self.filters {
            if filter.apply(alloc) {
                Self::remember(&mut self.memory, alloc.uid().clone(), filter.uid());
                return Some(filter.uid());
            }
        }
        None
    }

    /// Searches for a filter that matches on the input allocation, for its death.
    pub fn find_dead_match(&mut self, alloc: &AllocUid) -> Option<uid::FilterUid> {
        self.memory.get(alloc).map(|uid| *uid)
    }

    /// Resets all the filters.
    pub fn reset(&mut self) {
        self.memory.clear()
    }
}

/// # Message handling
impl Filters {
    /// Applies a filter message.
    pub fn update(&mut self, msg: msg::to_server::FiltersMsg) -> Res<(msg::to_client::Msgs, bool)> {
        use msg::to_server::FiltersMsg::*;
        let (res, should_reload) = match msg {
            RequestNew => (self.add_new(), false),
            Revert => (self.revert(), false),
            UpdateAll { catch_all, filters } => (self.update_all(catch_all, filters), true),
        };
        res.map(|msgs| (msgs, should_reload))
    }

    /// Sends all the filters to the client.
    pub fn revert(&self) -> Res<msg::to_client::Msgs> {
        let catch_all = self.catch_all.clone();
        let filters = self.filters.clone();
        Ok(vec![msg::to_client::FiltersMsg::revert(catch_all, filters)])
    }

    /// Updates all the filters.
    ///
    /// # TODO
    ///
    /// - decide whether we need to re-compute all the points using the `edited()` flags. If only
    ///     filter specifications have change, there is no need to re-compute the points.
    pub fn update_all(
        &mut self,
        mut catch_all: FilterSpec,
        mut filters: Vec<Filter>,
    ) -> Res<msg::to_client::Msgs> {
        catch_all.unset_edited();
        self.catch_all = catch_all;
        for filter in &mut filters {
            filter.unset_edited();
            filter.spec_mut().unset_edited();
        }
        self.filters = filters;
        Ok(vec![])
    }

    /// Adds a new filter.
    pub fn add_new(&mut self) -> Res<msg::to_client::Msgs> {
        let spec = FilterSpec::new(Color::random(true));
        let filter = Filter::new(spec).chain_err(|| "while creating new filter")?;
        let msg = msg::to_client::FiltersMsg::add(filter);
        Ok(vec![msg])
    }
}

impl std::ops::Index<index::Filter> for Filters {
    type Output = Filter;
    fn index(&self, index: index::Filter) -> &Filter {
        &self.filters[*index.deref()]
    }
}
impl std::ops::IndexMut<index::Filter> for Filters {
    fn index_mut(&mut self, index: index::Filter) -> &mut Filter {
        &mut self.filters[*index.deref()]
    }
}

/// A filter that combines `SubFilter`s.
///
/// Also contains a [`FilterSpec`](struct.FilterSpec.html).
///
/// # Invariants
///
/// - `self.uid().is_some()`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Filter {
    /// Actual list of filters.
    subs: Map<SubFilterUid, SubFilter>,
    /// Filter specification.
    spec: FilterSpec,
    /// Edited flag, for the client.
    edited: bool,
}
impl Filter {
    /// Constructor.
    pub fn new(spec: FilterSpec) -> Res<Filter> {
        if spec.uid().is_none() {
            bail!("trying to construct a filter with no UID")
        }
        let slf = Self {
            subs: Map::new(),
            spec,
            edited: false,
        };
        Ok(slf)
    }

    /// Specification accessor.
    pub fn spec(&self) -> &FilterSpec {
        &self.spec
    }
    /// Specification mutable accessor.
    pub fn spec_mut(&mut self) -> &mut FilterSpec {
        &mut self.spec
    }

    /// Name of the filter.
    pub fn name(&self) -> &str {
        self.spec().name()
    }

    /// True if the filter itself, and not the spec, has been edited.
    pub fn edited(&self) -> bool {
        self.edited
    }
    /// Sets the edited flag to true.
    pub fn set_edited(&mut self) {
        self.edited = true
    }
    /// Sets the edited flag to false.
    pub fn unset_edited(&mut self) {
        self.edited = false
    }

    /// UID accessor.
    pub fn uid(&self) -> FilterUid {
        self.spec()
            .uid()
            .expect("invariant violation, found a filter with no UID")
    }

    /// Applies the filters to an allocation.
    pub fn apply(&self, alloc: &Alloc) -> bool {
        for filter in self.subs.values() {
            if filter.apply(alloc) {
                return true;
            }
        }
        false
    }

    /// Removes a subfilter.
    pub fn remove(&mut self, sub_uid: uid::SubFilterUid) -> Res<()> {
        let prev = self.subs.remove(&sub_uid);
        if prev.is_none() {
            bail!("failed to remove unknown subfilter UID #{}", sub_uid)
        }
        Ok(())
    }

    /// Iterator over the subfilters.
    pub fn iter(&self) -> impl Iterator<Item = &SubFilter> {
        self.subs.values()
    }

    /// Mutable iterator over the subfilters.
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut SubFilter> {
        self.subs.values_mut()
    }

    /// Inserts a subfilter.
    ///
    /// Fails if the subfilter is **not** new.
    pub fn insert(&mut self, sub: SubFilter) -> Res<()> {
        let prev = self.subs.insert(sub.uid(), sub);
        if let Some(prev) = prev {
            bail!("subfilter UID collision on #{}", prev.uid())
        }
        Ok(())
    }

    /// Replaces a subfilter.
    ///
    /// Fails if the subfilter **is** new.
    pub fn replace(&mut self, sub: SubFilter) -> Res<()> {
        let uid = sub.uid();
        let prev = self.subs.insert(sub.uid(), sub);
        if prev.is_none() {
            bail!("failed to replace subfilter with unknown UID #{}", uid)
        }
        Ok(())
    }
}
