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

    /// The list of filters.
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
    pub fn update(&mut self, msg: msg::to_server::FiltersMsg) -> Res<msg::to_client::Msgs> {
        use msg::to_server::FiltersMsg::*;
        match msg {
            AddNew => self.add_new(),
            Rm(uid) => self.remove(uid),
            UpdateSpec { uid, spec } => self.update_spec(uid, spec),
            Filter { uid, msg } => self.update_filter(uid, msg),
        }
    }

    /// Adds a new filter.
    pub fn add_new(&mut self) -> Res<msg::to_client::Msgs> {
        let spec = FilterSpec::new(Color::random(true));
        let filter = Filter::new(spec).chain_err(|| "while creating new filter")?;
        self.filters.push(filter.clone());
        let msg = msg::to_client::FiltersMsg::add(filter);
        Ok(vec![msg])
    }

    /// Updates the specification of a filter.
    pub fn update_spec(
        &mut self,
        uid: Option<FilterUid>,
        new_spec: FilterSpec,
    ) -> Res<msg::to_client::Msgs> {
        let spec = self
            .get_spec_mut(uid)
            .chain_err(|| "while updating a filter specification")?;
        *spec = new_spec;
        // Send it to the client.
        let catch_all = if uid.is_none() {
            Some(spec.clone())
        } else {
            None
        };
        let mut specs = Map::new();
        if let Some(uid) = uid {
            specs.insert(uid, spec.clone());
        }
        let msg = msg::to_client::FiltersMsg::update_specs(catch_all, specs);
        Ok(vec![msg])
    }

    /// Handles a message for a particular filter.
    pub fn update_filter(
        &mut self,
        uid: FilterUid,
        msg: msg::to_server::FilterMsg,
    ) -> Res<msg::to_client::Msgs> {
        let (_, filter) = self
            .get_mut(uid)
            .chain_err(|| format!("while handling filter message {:?}", msg))?;
        filter
            .update(msg)
            .chain_err(|| format!("while updating filter `{}`", filter.spec().name()))?;
        Ok(vec![])
    }

    /// Removes a filter.
    ///
    /// - returns a message for the client to drop that filter.
    pub fn remove(&mut self, uid: FilterUid) -> Res<msg::to_client::Msgs> {
        let (index, _) = self.get_mut(uid).chain_err(|| "while removing chart")?;
        self.filters.remove(index);
        Ok(vec![msg::to_client::FiltersMsg::rm(uid).into()])
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

    /// Applies a filter message.
    pub fn update(&mut self, msg: msg::to_server::FilterMsg) -> Res<()> {
        use msg::to_server::FilterMsg::*;

        match msg {
            ReplaceSubs(subs) => {
                self.subs.clear();
                for mut sub in subs {
                    sub.sanitize();
                    self.subs.insert(sub.uid(), sub);
                }
            }
            AddNew => {
                let sub = SubFilter::default();
                let prev = self.subs.insert(sub.uid(), sub);
                debug_assert!(prev.is_none())
            }
            Rm(uid) => {
                let prev = self.subs.remove(&uid);
                if prev.is_none() {
                    bail!("failed to remove subfilter with unknown UID #{}", uid)
                }
            }
            Update(sub) => self.replace(sub)?,
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
