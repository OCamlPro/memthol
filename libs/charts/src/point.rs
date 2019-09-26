//! Point representation.

use crate::base::*;

pub use chart::time::TimePoints;

/// A point value.
///
/// Stores a value for each filter, and the value for the catch-all filter.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PointVal<Val> {
    /// Values for filter lines.
    pub filtered: Map<uid::FilterUid, Val>,
    /// Catch-all value.
    pub rest: Val,
}
impl<Val> PointVal<Val> {
    /// Constructor.
    pub fn new(default: Val, filters: &filter::Filters) -> Self
    where
        Val: Clone,
    {
        let mut filtered = Map::new();
        for filter in filters.filters() {
            filtered.insert(filter.uid(), default.clone());
        }
        Self {
            filtered,
            rest: default,
        }
    }

    /// Immutable ref over some value.
    pub fn get_mut_or(&mut self, filtered_index: Option<uid::FilterUid>, default: Val) -> &mut Val {
        let val = match filtered_index {
            None => &mut self.rest,
            Some(uid) => {
                let entry = self.filtered.entry(uid);
                entry.or_insert(default)
            }
        };
        val
    }

    /// Mutable ref over some value.
    pub fn get_mut(&mut self, filtered_index: Option<uid::FilterUid>) -> Res<&mut Val> {
        let val = match filtered_index {
            None => &mut self.rest,
            Some(uid) => match self.filtered.get_mut(&uid) {
                Some(val) => val,
                None => bail!("unknown filter UID #{} (get_mut)", uid),
            },
        };
        Ok(val)
    }

    /// Map over all values.
    pub fn map<F, Out>(self, mut f: F) -> Res<PointVal<Out>>
    where
        F: FnMut(Option<uid::FilterUid>, Val) -> Res<Out>,
    {
        let mut filtered = Map::new();
        for (uid, val) in self.filtered {
            filtered.insert(uid, f(Some(uid), val)?);
        }
        let res = PointVal {
            filtered,
            rest: f(None, self.rest)?,
        };
        Ok(res)
    }
}

/// A abstract point.
///
/// A point is a `key`, which is the x-value of the point, and the y-values for all the filters.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Point<Key, Val> {
    /// X-value.
    pub key: Key,
    /// Y-values.
    pub vals: PointVal<Val>,
}
impl<Key, Val> Point<Key, Val> {
    /// Constructor.
    pub fn new(key: Key, vals: PointVal<Val>) -> Self {
        Self { key, vals }
    }
}
impl<Key, Val> fmt::Display for Point<Key, Val>
where
    Key: fmt::Display,
    Val: fmt::Display,
{
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        let Point {
            key,
            vals: PointVal { filtered, rest },
        } = self;
        write!(fmt, "{{ x: {}, y: {}", key, rest)?;
        for (uid, val) in filtered.iter() {
            write!(fmt, ", y_{}: {}", uid, val)?
        }
        write!(fmt, "}}")
    }
}

/// Some points for a particular chart type.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Points {
    /// Points for a time chart.
    Time(TimePoints),
}

impl Points {
    /// True if there are no points.
    pub fn is_empty(&self) -> bool {
        match self {
            Self::Time(points) => points.is_empty(),
        }
    }
}

impl<T> From<T> for Points
where
    T: Into<TimePoints>,
{
    fn from(points: T) -> Self {
        Self::Time(points.into())
    }
}

/// Some points for all the charts.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChartPoints {
    /// The actual points.
    points: Map<uid::ChartUid, Points>,
}
impl ChartPoints {
    /// Constructor.
    pub fn new() -> Self {
        Self { points: Map::new() }
    }

    /// True if there are no points.
    pub fn is_empty(&self) -> bool {
        self.points.iter().all(|(_uid, points)| points.is_empty())
    }
}

impl Deref for ChartPoints {
    type Target = Map<uid::ChartUid, Points>;
    fn deref(&self) -> &Self::Target {
        &self.points
    }
}
impl DerefMut for ChartPoints {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.points
    }
}
