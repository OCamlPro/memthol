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
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Filters {
    /// The actual list of filters.
    filters: Vec<Filter>,
    /// The specification of the catch-all filter.
    catch_all: FilterSpec,
}

impl Filters {
    /// Constructor.
    pub fn new() -> Self {
        Filters {
            filters: vec![],
            catch_all: FilterSpec::new_catch_all(),
        }
    }

    /// Length of the list of filters.
    pub fn len(&self) -> usize {
        self.filters.len()
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

    /// Searches for a filter that matches on the input allocation.
    pub fn find_match(&self, alloc: &Alloc) -> Option<index::Filter> {
        for (index, filter) in self.filters.iter().enumerate() {
            if filter.apply(alloc) {
                return Some(index::Filter::new(index));
            }
        }
        None
    }
}

/// # Message handling
impl Filters {
    /// Applies a filter message.
    pub fn update(&mut self, msg: msg::to_server::FiltersMsg) -> Res<msg::to_client::Msgs> {
        use msg::to_server::FiltersMsg::*;
        match msg {
            Add(filter) => self.add(filter),
            Rm(uid) => self.remove(uid),
            UpdateSpec { uid, spec } => self.update_spec(uid, spec),
            Filter { uid, msg } => self.update_filter(uid, msg),
        }
    }

    /// Adds a filter.
    pub fn add(&mut self, filter: Filter) -> Res<msg::to_client::Msgs> {
        self.filters.push(filter);
        Ok(vec![])
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
        filter.update(msg);
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
/// # Invariants
///
/// - `self.uid().is_some()`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Filter {
    /// Actual list of filters.
    filters: Vec<SubFilter>,
    /// Filter specification.
    spec: FilterSpec,
}
impl Filter {
    /// Constructor.
    pub fn new(spec: FilterSpec) -> Res<Filter> {
        if spec.uid().is_none() {
            bail!("trying to construct a filter with no UID")
        }
        let slf = Self {
            filters: vec![],
            spec,
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

    /// UID accessor.
    pub fn uid(&self) -> FilterUid {
        self.spec()
            .uid()
            .expect("invariant violation, found a filter with no UID")
    }

    /// Applies the filters to an allocation.
    pub fn apply(&self, alloc: &Alloc) -> bool {
        for filter in &self.filters {
            if filter.apply(alloc) {
                return true;
            }
        }
        false
    }

    /// Applies a filter message.
    pub fn update(&mut self, msg: msg::to_server::FilterMsg) {
        use msg::to_server::FilterMsg::*;

        match msg {
            Add { filter } => self.filters.push(filter),
            Rm { index } => {
                self.filters.remove(*index);
                ()
            }
            Update { index, filter } => self.filters[*index] = filter,
        }
    }

    /// Iterator over the sub-filters.
    pub fn iter(&self) -> impl Iterator<Item = (index::SubFilter, &SubFilter)> {
        self.filters
            .iter()
            .enumerate()
            .map(|(index, filter)| (index::SubFilter::new(index), filter))
    }

    /// Mutable iterator over the sub-filters.
    pub fn iter_mut(&mut self) -> impl Iterator<Item = (index::SubFilter, &mut SubFilter)> {
        self.filters
            .iter_mut()
            .enumerate()
            .map(|(index, filter)| (index::SubFilter::new(index), filter))
    }

    /// Overwrites a sub-filter.
    pub fn set(&mut self, index: index::SubFilter, sub_filter: SubFilter) {
        self.filters[*index.deref()] = sub_filter
    }
}

impl std::ops::Index<index::SubFilter> for Filter {
    type Output = SubFilter;
    fn index(&self, index: index::SubFilter) -> &SubFilter {
        &self.filters[*index.deref()]
    }
}
impl std::ops::IndexMut<index::SubFilter> for Filter {
    fn index_mut(&mut self, index: index::SubFilter) -> &mut SubFilter {
        &mut self.filters[*index.deref()]
    }
}
