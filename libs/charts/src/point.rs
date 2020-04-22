//! Point representation.

use crate::common::*;

pub use chart::time::TimePoints;

/// A point value.
///
/// Stores a value for each filter, and the value for the catch-all filter.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PointVal<Val> {
    /// Values for filter lines.
    pub map: Map<uid::LineUid, Val>,
}
impl<Val> PointVal<Val> {
    /// Constructor.
    pub fn new(default: Val, filters: &filter::Filters) -> Self
    where
        Val: Clone,
    {
        let mut map = Map::new();
        map.insert(uid::LineUid::CatchAll, default.clone());
        map.insert(uid::LineUid::Everything, default.clone());
        for filter in filters.filters() {
            map.insert(uid::LineUid::Filter(filter.uid()), default.clone());
        }
        Self { map }
    }

    /// Immutable ref over some value.
    pub fn get_mut_or(&mut self, uid: uid::LineUid, default: Val) -> &mut Val {
        self.map.entry(uid).or_insert(default)
    }

    /// Mutable ref over some value.
    pub fn get_mut(&mut self, uid: uid::LineUid) -> Res<&mut Val> {
        self.map
            .get_mut(&uid)
            .ok_or_else(|| format!("unknown line uid `{}`", uid).into())
    }

    /// Map over all values.
    pub fn map<F, Out>(self, mut f: F) -> Res<PointVal<Out>>
    where
        F: FnMut(uid::LineUid, Val) -> Res<Out>,
    {
        let mut map = Map::new();
        for (uid, val) in self.map {
            map.insert(uid, f(uid, val)?);
        }
        let res = PointVal { map };
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
            vals: PointVal { map },
        } = self;
        write!(fmt, "{{ x: {}", key)?;
        for (uid, val) in map.iter() {
            write!(fmt, ", {}: {}", uid.y_axis_key(), val)?
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
