//! Data filtering.
//!
//! All types in this module implement `serde`'s `Serialize` and `Deserialize` traits.

prelude! {}

pub mod label;
pub mod loc;
pub mod ord;
mod spec;
pub mod stats;
pub mod string_like;
pub mod sub;

#[cfg(any(test, feature = "server"))]
pub mod gen;

#[cfg(any(test, feature = "server"))]
pub use gen::FilterGen;
pub use label::LabelFilter;
pub use loc::LocFilter;
use ord::OrdFilter;
pub use spec::FilterSpec;
pub use sub::SubFilter;

/// A filter over allocation sizes.
pub type SizeFilter = OrdFilter<u32>;

/// A filter over allocation lifetimes.
pub type LifetimeFilter = OrdFilter<time::Lifetime>;
impl LifetimeFilter {
    /// Applies the filter to an allocation w.r.t. a diff timestamp.
    ///
    /// It should always be the case that `alloc_toc <= timestamp`.
    pub fn apply_at(&self, timestamp: &time::SinceStart, alloc_toc: &time::SinceStart) -> bool {
        debug_assert!(alloc_toc <= timestamp);
        let lt = (timestamp - alloc_toc).to_lifetime();
        self.apply(&lt)
    }
}

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
    Ord(ord::Pred),
    /// Label comparison.
    Label(label::LabelPred),
    /// Location comparison.
    Loc(loc::LocPred),
}
impl CmpKind {
    /// Ordered comparison constructor.
    pub fn new_ord(kind: ord::Pred) -> Self {
        Self::Ord(kind)
    }
    /// Label comparison constructor.
    pub fn new_label(kind: label::LabelPred) -> Self {
        Self::Label(kind)
    }
    /// Location comparison constructor.
    pub fn new_loc(kind: loc::LocPred) -> Self {
        Self::Loc(kind)
    }
}
impl fmt::Display for CmpKind {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Ord(kind) => write!(fmt, "{}", kind),
            Self::Label(kind) => write!(fmt, "{}", kind),
            Self::Loc(kind) => write!(fmt, "{}", kind),
        }
    }
}

/// Filter kind.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum FilterKind {
    /// Size filter.
    Size,
    /// Lifetime filter.
    Lifetime,
    /// Label filter.
    Label,
    /// Location filter.
    Loc,
}
impl fmt::Display for FilterKind {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Size => write!(fmt, "size"),
            Self::Lifetime => write!(fmt, "lifetime"),
            Self::Label => write!(fmt, "labels"),
            Self::Loc => write!(fmt, "locations"),
        }
    }
}

impl FilterKind {
    /// List of all the different filter kinds.
    pub fn all() -> Vec<FilterKind> {
        base::debug_do! {
            // If this fails, it means you added/removed a variant to/from `FilterKind`. The vector
            // below, which yields all variants, must be updated.
            match Self::Size {
                Self::Size => (),
                Self::Lifetime => (),
                Self::Label => (),
                Self::Loc => (),
            }
        }

        // Lists all `FilterKind` variants.
        vec![
            FilterKind::Size,
            FilterKind::Lifetime,
            FilterKind::Label,
            FilterKind::Loc,
        ]
    }
}

/// A list of filters.
///
/// Aggregates the following:
///
/// - a "catch all" [`FilterSpec`], the specification for the points that no filter catches;
/// - a "everything" [`FilterSpec`], the specification for all the points, regardless of
///     user-defined filters;
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
    /// The specification of the "catch-all" filter.
    catch_all: FilterSpec,
    /// The specification of the "everything" filter.
    everything: FilterSpec,
    /// The actual list of filters.
    filters: Vec<Filter>,
    /// Remembers which filter is responsible for an allocation.
    memory: BTMap<uid::Alloc, uid::Filter>,
}

impl Filters {
    /// Constructor.
    pub fn new() -> Self {
        Filters {
            filters: vec![],
            catch_all: FilterSpec::new_catch_all(),
            everything: FilterSpec::new_everything(),
            memory: BTMap::new(),
        }
    }

    /// Specification of the `catch_all` filter.
    pub fn catch_all(&self) -> &FilterSpec {
        &self.catch_all
    }
    /// Specification of the `everything` filter.
    pub fn everything(&self) -> &FilterSpec {
        &self.everything
    }
    /// Specifications of the custom filters.
    pub fn filters(&self) -> &Vec<Filter> {
        &self.filters
    }

    /// Runs filter generation.
    ///
    /// Returns the number of filter generated.
    #[cfg(any(test, feature = "server"))]
    pub fn auto_gen(
        &mut self,
        data: &data::Data,
        generator: impl Into<filter::gen::FilterGen>,
    ) -> Res<usize> {
        let generator = generator.into();
        let filters = generator.run(data)?;
        let count = filters.len();
        self.filters.extend(filters);
        Ok(count)
    }

    /// Length of the list of filters.
    pub fn len(&self) -> usize {
        self.filters.len()
    }

    /// Filter specification mutable accessor.
    pub fn get_spec_mut(&mut self, uid: Option<uid::Filter>) -> Res<&mut FilterSpec> {
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
    pub fn get_mut(&mut self, uid: uid::Filter) -> Res<(usize, &mut Filter)> {
        for (index, filter) in self.filters.iter_mut().enumerate() {
            if filter.uid() == uid {
                return Ok((index, filter));
            }
        }
        bail!("cannot access filter with unknown UID #{}", uid)
    }

    /// Iterator over the filters.
    pub fn iter(&self) -> impl Iterator<Item = &Filter> {
        self.filters.iter()
    }

    /// Mutable iterator over the filters.
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Filter> {
        self.filters.iter_mut()
    }

    /// Remembers that an allocation is handled by some filter.
    fn remember(
        memory: &mut BTMap<uid::Alloc, uid::Filter>,
        alloc: uid::Alloc,
        filter: uid::Filter,
    ) {
        let prev = memory.insert(alloc, filter);
        let collision = prev.map(|uid| uid != filter).unwrap_or(false);
        if collision {
            panic!("filter memory collision")
        }
    }

    /// Searches for a filter that matches on the input allocation.
    pub fn find_match(
        &mut self,
        timestamp: &time::SinceStart,
        alloc: &Alloc,
    ) -> Option<uid::Filter> {
        for filter in &self.filters {
            if filter.apply(timestamp, alloc) {
                Self::remember(&mut self.memory, alloc.uid().clone(), filter.uid());
                return Some(filter.uid());
            }
        }
        None
    }

    /// Searches for a filter that matches on the input allocation, for its death.
    pub fn find_dead_match(&mut self, alloc: &uid::Alloc) -> Option<uid::Filter> {
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
            UpdateAll {
                everything,
                filters,
                catch_all,
            } => (self.update_all(everything, filters, catch_all), true),
        };
        res.map(|msgs| (msgs, should_reload))
    }

    /// Sends all the filters to the client.
    pub fn revert(&self) -> Res<msg::to_client::Msgs> {
        let catch_all = self.catch_all.clone();
        let everything = self.everything.clone();
        let filters = self.filters.clone();
        Ok(vec![msg::to_client::FiltersMsg::revert(
            everything, filters, catch_all,
        )])
    }

    /// Updates all the filters.
    pub fn update_all(
        &mut self,
        everything: FilterSpec,
        filters: Vec<Filter>,
        catch_all: FilterSpec,
    ) -> Res<msg::to_client::Msgs> {
        self.catch_all = catch_all;
        self.everything = everything;
        self.filters = filters;
        Ok(vec![])
    }

    /// Adds a new filter.
    pub fn add_new(&mut self) -> Res<msg::to_client::Msgs> {
        let spec = FilterSpec::new(Color::random());
        let filter = Filter::new(spec).chain_err(|| "while creating new filter")?;
        let msg = msg::to_client::FiltersMsg::add(filter);
        Ok(vec![msg])
    }

    /// Extract filter statistics.
    #[cfg(any(test, feature = "server"))]
    pub fn filter_stats(&self) -> Res<stats::AllFilterStats> {
        let mut stats = stats::AllFilterStats::new();
        let mut registered = 0;

        for (_, filter) in &self.memory {
            registered += 1;
            stats.stats_do((*filter).into(), |stats| stats.inc())
        }

        let total = data::alloc_count()?;
        if registered > total {
            bail!(
                "inconsistent state, extracted filter stats for {} allocation, \
                but allocation count is {}",
                registered,
                total,
            )
        }

        stats.stats_do(uid::Line::CatchAll, |stats| {
            stats.alloc_count = total - registered
        });

        Ok(stats)
    }
}

/// A filter that combines `SubFilter`s.
///
/// Also contains a [`FilterSpec`](struct.FilterSpec.html).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Filter {
    /// Actual list of filters.
    subs: BTMap<uid::SubFilter, SubFilter>,
    /// Filter specification.
    spec: FilterSpec,
}

impl Filter {
    /// Constructor.
    pub fn new(spec: FilterSpec) -> Res<Filter> {
        if spec.uid().filter_uid().is_none() {
            bail!("trying to construct a filter with no UID")
        }
        let slf = Self {
            subs: BTMap::new(),
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

    /// Name of the filter.
    pub fn name(&self) -> &str {
        self.spec().name()
    }

    /// UID accessor.
    pub fn uid(&self) -> uid::Filter {
        self.spec()
            .uid()
            .filter_uid()
            .expect("invariant violation, found a filter with no UID")
    }

    /// Applies the filters to an allocation.
    pub fn apply(&self, timestamp: &time::SinceStart, alloc: &Alloc) -> bool {
        for filter in self.subs.values() {
            if !filter.apply(timestamp, alloc) {
                return false;
            }
        }
        true
    }

    /// Removes a subfilter.
    pub fn remove(&mut self, sub_uid: uid::SubFilter) -> Res<()> {
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
    pub fn insert(&mut self, sub: impl Into<SubFilter>) -> Res<()> {
        let sub = sub.into();
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
