//! Total size over time chart.

use crate::base::*;

/// Total size over time chart.
pub struct TimeSize {
    /// Timestamp of the last allocation registered.
    timestamp: SinceStart,
    /// Current total size.
    size: usize,
    /// Map used to construct the points.
    map: Map<SinceStart, (usize, usize)>,
}
impl TimeSize {
    /// Constructor.
    pub fn new() -> Self {
        let timestamp = SinceStart::zero();
        let size = 0;
        let map = Map::new();
        Self {
            timestamp,
            size,
            map,
        }
    }
}

// /// # Helpers for interacting with `self.map`
// impl TimeSize {
//     ///
// }

macro_rules! map {
    ($map:expr => at $date:expr, add $size:expr) => {
        $map.entry($date).or_insert((0, 0)).0 += $size
    };
    ($map:expr => at $date:expr, add $size:expr) => {
        $map.entry($date).or_insert((0, 0)).1 += $size
    };
}

/// # Helpers for point generation
impl TimeSize {
    /// Retrieves all allocations since its internal timestamp.
    ///
    /// - assumes `self.map.is_empty()`;
    /// - updates the internal timestamp.
    pub fn get_allocs(&mut self) -> Res<()> {
        debug_assert!(self.map.is_empty());

        let data = data::get().chain_err(|| "while retrieving allocations for chart")?;
        // The idea is to go through the time-ordered list of allocations in reverse order, and get
        // everything until we reach `self.timestamp`.
        for (_, alloc) in data.uid_map.iter().rev() {
            if alloc.toc < self.timestamp {
                map!(self.map => at alloc.toc, add alloc.size)
            } else {
                break;
            }
        }

        self.timestamp = data.current_time;

        Ok(())
    }
}
