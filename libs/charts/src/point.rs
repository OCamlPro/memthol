//! Point representation.

use crate::base::*;

pub use time::TimePoint;

/// A point value.
///
/// Stores a value for each filter, and the value for the catch-all filter.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PointVal<Val> {
    /// Values for filter lines.
    pub filtered: Vec<Val>,
    /// Catch-all value.
    pub rest: Val,
}
impl<Val> PointVal<Val> {
    /// Constructor.
    pub fn new(default: Val, filtered_len: usize) -> Self
    where
        Val: Clone,
    {
        let mut filtered = Vec::with_capacity(filtered_len);
        for _ in 0..filtered_len {
            filtered.push(default.clone())
        }
        Self {
            filtered,
            rest: default,
        }
    }

    /// Mutable ref over some value.
    pub fn get_mut(&mut self, filtered_index: Option<index::Filter>) -> &mut Val {
        match filtered_index {
            None => &mut self.rest,
            Some(idx) => &mut self.filtered[*idx],
        }
    }

    /// Map over all values.
    pub fn map<F, Out>(self, mut f: F) -> PointVal<Out>
    where
        F: FnMut(Option<usize>, Val) -> Out,
    {
        PointVal {
            filtered: self
                .filtered
                .into_iter()
                .enumerate()
                .map(|(index, val)| f(Some(index), val))
                .collect(),
            rest: f(None, self.rest),
        }
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

/// Some points for a particular chart type.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Points {
    /// Points for time chart.
    Time(time::TimePoints),
}

impl<T> From<T> for Points
where
    T: Into<time::TimePoints>,
{
    fn from(points: T) -> Self {
        Self::Time(points.into())
    }
}
