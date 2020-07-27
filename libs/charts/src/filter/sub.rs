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
    /// Filter over lifetime.
    Lifetime(LifetimeFilter),
    /// Filter over labels.
    Label(LabelFilter),
    /// Filter over locations.
    Loc(LocFilter),
}

impl RawSubFilter {
    /// Filter kind of a filter.
    pub fn kind(&self) -> FilterKind {
        match self {
            Self::Size(_) => FilterKind::Size,
            Self::Lifetime(_) => FilterKind::Lifetime,
            Self::Label(_) => FilterKind::Label,
            Self::Loc(_) => FilterKind::Loc,
        }
    }

    /// Applies the filter to an allocation.
    pub fn apply(&self, timestamp: &time::SinceStart, alloc: &Alloc) -> bool {
        match self {
            RawSubFilter::Size(filter) => filter.apply(&alloc.size),
            RawSubFilter::Lifetime(filter) => {
                let timestamp = alloc
                    .tod()
                    .map(|tod| std::cmp::min(tod, *timestamp))
                    .unwrap_or(*timestamp);
                filter.apply_at(&timestamp, &alloc.toc())
            }
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

        *self = Self::from(kind);
        true
    }

    /// Updates a sub-filter.
    pub fn update(&mut self, update: Update) -> Res<bool> {
        macro_rules! fail {
            () => {
                bail!("cannot apply update `{}` to filter `{}`", update, self)
            };
        }
        match self {
            Self::Size(filter) => match update {
                Update::Size(update) => filter.update(update),
                _ => fail!(),
            },
            Self::Lifetime(filter) => match update {
                Update::Lifetime(update) => filter.update(update),
                _ => fail!(),
            },
            Self::Label(filter) => match update {
                Update::Label(update) => filter.update(update),
                _ => fail!(),
            },
            Self::Loc(filter) => match update {
                Update::Loc(update) => filter.update(update),
                _ => fail!(),
            },
        }
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
    /// Lifetime filter update.
    Lifetime(ord::LifetimeUpdate),
    /// Label filter update.
    Label(label::LabelUpdate),
    /// Location filter update.
    Loc(loc::LocUpdate),
}

base::implement! {
    Display {
        SubFilter => |&self, fmt| write!(fmt, "{}({})", self.uid, self.raw),

        RawSubFilter => |&self, fmt| match self {
            Self::Size(filter) => write!(fmt, "size {}", filter),
            Self::Lifetime(filter) => write!(fmt, "lifetime {}", filter),
            Self::Label(filter) => write!(fmt, "labels {}", filter),
            Self::Loc(filter) => write!(fmt, "callstack {}", filter),
        },

        Update => |&self, fmt| match self {
            Self::Size(update) => update.fmt(fmt),
            Self::Lifetime(update) => update.fmt(fmt),
            Self::Label(update) => update.fmt(fmt),
            Self::Loc(update) => update.fmt(fmt),
        },
    }

    From {
        RawSubFilter, from FilterKind => |kind| match kind {
            FilterKind::Size => SizeFilter::default().into(),
            FilterKind::Lifetime => LifetimeFilter::default().into(),
            FilterKind::Label => LabelFilter::default().into(),
            FilterKind::Loc => LocFilter::default().into(),
        },
        RawSubFilter, from SizeFilter => |filter| Self::Size(filter),
        RawSubFilter, from LifetimeFilter => |filter| Self::Lifetime(filter),
        RawSubFilter, from LabelFilter => |filter| Self::Label(filter),
        RawSubFilter, from LocFilter => |filter| Self::Loc(filter),

        SubFilter, from FilterKind => |kind| Self::of(
            SubFilterUid::fresh(), RawSubFilter::from(kind)
        ),
        SubFilter, from SizeFilter => |filter| Self::of(
            SubFilterUid::fresh(), RawSubFilter::from(filter)
        ),
        SubFilter, from LifetimeFilter => |filter| Self::of(
            SubFilterUid::fresh(), RawSubFilter::from(filter)
        ),
        SubFilter, from LabelFilter => |filter| Self::of(
            SubFilterUid::fresh(), RawSubFilter::from(filter)
        ),
        SubFilter, from LocFilter => |filter| Self::of(
            SubFilterUid::fresh(), RawSubFilter::from(filter)
        ),
        SubFilter, from RawSubFilter => |filter| Self::of(
            SubFilterUid::fresh(), filter
        ),
    }

    Default {
        RawSubFilter => SizeFilter::default().into(),
        SubFilter => Self::of(SubFilterUid::fresh(), RawSubFilter::default()),
    }
}
