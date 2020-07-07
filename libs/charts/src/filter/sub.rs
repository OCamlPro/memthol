//! Sub filters.
//!
//! A sub filter is what [`Filter`]s are made of.
//!
//! [`Filter`]: ../struct.Filter.html (The Filter struct).

prelude! {}
use filter::*;

/// An allocation filter.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RawSubFilter {
    /// Filter over allocation sizes.
    Size(SizeFilter),
    /// Filter over labels.
    Label(LabelFilter),
    /// Filter over locations.
    Loc(LocFilter),
}
impl RawSubFilter {
    /// Default filter for some filter kind.
    pub fn of_kind(kind: FilterKind) -> Self {
        match kind {
            FilterKind::Size => SizeFilter::default().into(),
            FilterKind::Label => LabelFilter::default().into(),
            FilterKind::Loc => LocFilter::default().into(),
        }
    }

    /// Filter kind of a filter.
    pub fn kind(&self) -> FilterKind {
        match self {
            Self::Size(_) => FilterKind::Size,
            Self::Label(_) => FilterKind::Label,
            Self::Loc(_) => FilterKind::Loc,
        }
    }

    /// Applies the filter to an allocation.
    pub fn apply(&self, alloc: &Alloc) -> bool {
        match self {
            RawSubFilter::Size(filter) => filter.apply(&alloc.size),
            RawSubFilter::Label(filter) => filter.apply(&alloc.labels()),
            RawSubFilter::Loc(filter) => filter.apply(&alloc.trace()),
        }
    }

    /// Changes the filter kind of a sub-filter.
    ///
    /// Returns `true` iff the filter actually changed.
    pub fn change_kind(&mut self, kind: FilterKind) -> bool {
        if self.kind() == kind {
            return false;
        }

        *self = Self::of_kind(kind);
        true
    }

    /// Updates a sub-filter.
    pub fn update(&mut self, update: Update) -> Res<bool> {
        match self {
            Self::Size(filter) => match update {
                Update::Size(update) => filter.update(update),
                update => bail!("cannot apply update `{}` to filter `{}`", update, self),
            },
            Self::Label(filter) => match update {
                Update::Label(update) => filter.update(update),
                update => bail!("cannot apply update `{}` to filter `{}`", update, self),
            },
            Self::Loc(filter) => match update {
                Update::Loc(update) => filter.update(update),
                update => bail!("cannot apply update `{}` to filter `{}`", update, self),
            },
        }
    }
}

impl fmt::Display for RawSubFilter {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Size(filter) => write!(fmt, "size {}", filter),
            Self::Label(filter) => write!(fmt, "labels {}", filter),
            Self::Loc(filter) => write!(fmt, "callstack {}", filter),
        }
    }
}

impl From<SizeFilter> for RawSubFilter {
    fn from(filter: SizeFilter) -> Self {
        Self::Size(filter)
    }
}
impl From<LabelFilter> for RawSubFilter {
    fn from(filter: LabelFilter) -> Self {
        Self::Label(filter)
    }
}
impl From<LocFilter> for RawSubFilter {
    fn from(filter: LocFilter) -> Self {
        Self::Loc(filter)
    }
}
impl Default for RawSubFilter {
    fn default() -> Self {
        SizeFilter::default().into()
    }
}

#[cfg(target_arch = "wasm32")]
static CREATOR_FLAG: bool = true;
#[cfg(not(target_arch = "wasm32"))]
static CREATOR_FLAG: bool = false;

/// A sub-filter: a [`RawSubFilter`](enum.RawSubFilter.html) with a
/// [`SubFilterUid`](../uid/struct.SubFilterUid.html).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubFilter {
    /// The UID.
    uid: SubFilterUid,
    /// Actual subfilter.
    raw: RawSubFilter,
    /// True if it was created by the client.
    from_client: bool,
}
impl SubFilter {
    fn of(uid: SubFilterUid, raw: RawSubFilter) -> Self {
        Self {
            uid,
            raw,
            from_client: CREATOR_FLAG,
        }
    }
}

impl Deref for SubFilter {
    type Target = RawSubFilter;
    fn deref(&self) -> &RawSubFilter {
        &self.raw
    }
}
impl DerefMut for SubFilter {
    fn deref_mut(&mut self) -> &mut RawSubFilter {
        &mut self.raw
    }
}

impl From<SizeFilter> for SubFilter {
    fn from(filter: SizeFilter) -> Self {
        Self::of(SubFilterUid::fresh(), RawSubFilter::from(filter))
    }
}
impl From<LabelFilter> for SubFilter {
    fn from(filter: LabelFilter) -> Self {
        Self::of(SubFilterUid::fresh(), RawSubFilter::from(filter))
    }
}
impl Default for SubFilter {
    fn default() -> Self {
        Self::of(SubFilterUid::fresh(), RawSubFilter::default())
    }
}
impl std::cmp::PartialEq for SubFilter {
    fn eq(&self, other: &Self) -> bool {
        self.uid.eq(&other.uid)
    }
}
impl std::cmp::Eq for SubFilter {}
impl std::cmp::PartialOrd for SubFilter {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.uid.cmp(&other.uid))
    }
}
impl std::cmp::Ord for SubFilter {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.uid.cmp(&other.uid)
    }
}

impl SubFilter {
    /// Default filter for some filter kind.
    pub fn of_kind(kind: FilterKind) -> Self {
        Self::of(SubFilterUid::fresh(), RawSubFilter::of_kind(kind))
    }

    /// Constructor.
    pub fn new(uid: SubFilterUid, raw: RawSubFilter) -> Self {
        Self::of(uid, raw)
    }

    /// Subfilter UID.
    pub fn uid(&self) -> SubFilterUid {
        self.uid
    }

    /// Raw subfilter.
    pub fn raw(&self) -> &RawSubFilter {
        &self.raw
    }

    /// True if the subfilter was created from the client.
    pub fn is_from_client(&self) -> bool {
        self.from_client
    }

    /// Sanitizes a subfilter, must be called when getting subfilters from the client.
    ///
    /// Checks whether the subfilter is from the client. If it is, overwrites its UID with a fresh
    /// one.
    pub fn sanitize(&mut self) {
        if self.is_from_client() {
            self.uid = SubFilterUid::fresh()
        }
    }
}

/// Sub-filter update.
pub enum Update {
    /// Size filter update.
    Size(ord::SizeUpdate),
    /// Label filter update.
    Label(label::LabelUpdate),
    /// Location filter update.
    Loc(loc::LocUpdate),
}
impl fmt::Display for Update {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Size(update) => update.fmt(fmt),
            Self::Label(update) => update.fmt(fmt),
            Self::Loc(update) => update.fmt(fmt),
        }
    }
}
