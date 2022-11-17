/*<LICENSE>
    This file is part of Memthol.

    Copyright (C) 2020 OCamlPro.

    Memthol is free software: you can redistribute it and/or modify
    it under the terms of the GNU General Public License as published by
    the Free Software Foundation, either version 3 of the License, or
    (at your option) any later version.

    Memthol is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
    GNU General Public License for more details.

    You should have received a copy of the GNU General Public License
    along with Memthol.  If not, see <https://www.gnu.org/licenses/>.
*/

//! Sub filters.
//!
//! A sub filter is what [`Filter`]s are made of.

prelude! {}
use filter::*;

/// An allocation filter.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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

#[cfg(not(feature = "server"))]
static CREATOR_FLAG: bool = true;
#[cfg(feature = "server")]
static CREATOR_FLAG: bool = false;

/// A sub-filter: a [`RawSubFilter`] with a UID.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SubFilter {
    /// The UID.
    uid: uid::SubFilter,
    /// Actual subfilter.
    raw: RawSubFilter,
    /// True if it was created by the client.
    from_client: bool,
}
impl SubFilter {
    fn from(uid: uid::SubFilter, raw: RawSubFilter) -> Self {
        Self {
            uid,
            raw,
            from_client: CREATOR_FLAG,
        }
    }
}

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
    pub fn new(uid: uid::SubFilter, raw: RawSubFilter) -> Self {
        Self::from(uid, raw)
    }

    /// Subfilter UID.
    pub fn uid(&self) -> uid::SubFilter {
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
            self.uid = uid::SubFilter::fresh()
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
    impl SubFilter {
        Display {
            |&self, fmt| write!(fmt, "{}({})", self.uid, self.raw)
        }

        Default {
            Self::from(uid::SubFilter::fresh(), RawSubFilter::default()),
        }

        From {
            from FilterKind => |kind| Self::from(
                uid::SubFilter::fresh(), RawSubFilter::from(kind)
            ),
            from SizeFilter => |filter| Self::from(
                uid::SubFilter::fresh(), RawSubFilter::from(filter)
            ),
            from LifetimeFilter => |filter| Self::from(
                uid::SubFilter::fresh(), RawSubFilter::from(filter)
            ),
            from LabelFilter => |filter| Self::from(
                uid::SubFilter::fresh(), RawSubFilter::from(filter)
            ),
            from LocFilter => |filter| Self::from(
                uid::SubFilter::fresh(), RawSubFilter::from(filter)
            ),
            from RawSubFilter => |filter| Self::from(
                uid::SubFilter::fresh(), filter
            ),
        }

        Deref {
            to RawSubFilter => |&self| &self.raw
        }
        DerefMut {
            |&mut self| &mut self.raw
        }
    }

    impl RawSubFilter {
        Display {
            |&self, fmt| match self {
                Self::Size(filter) => write!(fmt, "size {}", filter),
                Self::Lifetime(filter) => write!(fmt, "lifetime {}", filter),
                Self::Label(filter) => write!(fmt, "labels {}", filter),
                Self::Loc(filter) => write!(fmt, "callstack {}", filter),
            }
        }

        Default {
            SizeFilter::default().into()
        }

        From {
            from FilterKind => |kind| match kind {
                FilterKind::Size => SizeFilter::default().into(),
                FilterKind::Lifetime => LifetimeFilter::default().into(),
                FilterKind::Label => LabelFilter::default().into(),
                FilterKind::Loc => LocFilter::default().into(),
            },
            from SizeFilter => |filter| Self::Size(filter),
            from LifetimeFilter => |filter| Self::Lifetime(filter),
            from LabelFilter => |filter| Self::Label(filter),
            from LocFilter => |filter| Self::Loc(filter),
        }
    }

    impl Update {
        Display {
            |&self, fmt| match self {
                Self::Size(update) => update.fmt(fmt),
                Self::Lifetime(update) => update.fmt(fmt),
                Self::Label(update) => update.fmt(fmt),
                Self::Loc(update) => update.fmt(fmt),
            }
        }
    }
}
