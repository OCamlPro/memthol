//! Total size over time chart.

use crate::base::*;

/// A total-size-over-time point.
pub type TimeSizePoint = super::TimePoint<usize>;
/// Some total-size-over-time points.
pub type TimeSizePoints = Vec<TimeSizePoint>;

/// Total size over time chart.
#[derive(Debug, Serialize, Deserialize)]
pub struct TimeSize {
    /// Timestamp of the last allocation registered.
    timestamp: SinceStart,
    /// Current total size.
    size: PointVal<usize>,
    /// Map used to construct the points.
    map: Map<Date, PointVal<(usize, usize)>>,
}

impl Default for TimeSize {
    fn default() -> Self {
        Self {
            timestamp: SinceStart::zero(),
            size: Self::init_size(),
            map: Map::new(),
        }
    }
}

impl ChartExt for TimeSize {
    fn new_points(&mut self, filters: &Filters, init: bool) -> Res<Points> {
        self.get_allocs(filters, init)?;
        Ok(self.generate_points()?.into())
    }

    fn reset(&mut self) {
        self.timestamp = SinceStart::zero();
        self.size = Self::init_size();
        self.map.clear()
    }
}

impl TimeSize {
    /// Constructor.
    pub fn new() -> Self {
        let timestamp = SinceStart::zero();
        let size = PointVal::new(0, 0);
        let map = Map::new();
        Self {
            timestamp,
            size,
            map,
        }
    }

    /// Initial size.
    fn init_size() -> PointVal<usize> {
        PointVal::new(0, 0)
    }
}

macro_rules! map {
    (entry $map:expr, with $filters:expr => at $date:expr) => {
        $map.entry($date)
            .or_insert(PointVal::new((0, 0), $filters.len()))
    };
}

/// # Helpers for point generation
impl TimeSize {
    /// Generates points from what's in `self.map`.
    ///
    /// This function should only be called right after `get_allocs`.
    ///
    /// - clears `self.map`.
    fn generate_points(&mut self) -> Res<TimeSizePoints> {
        let map = std::mem::replace(&mut self.map, Map::new());

        let mut points = Vec::with_capacity(self.map.len());

        for (time, point_val) in map {
            let point_val = point_val.map(|index_opt, (to_add, to_sub)| match index_opt {
                None => (self.size.rest + to_add) - to_sub,
                Some(idx) => (self.size.filtered[idx] + to_add) - to_sub,
            });
            self.size = point_val.clone();
            points.push(Point::new(time, point_val))
        }

        Ok(points)
    }

    /// Retrieves all allocations since its internal timestamp.
    ///
    /// - assumes `self.map.is_empty()`;
    /// - new allocations will be in `self.map`;
    /// - updates the internal timestamp.
    fn get_allocs(&mut self, filters: &Filters, init: bool) -> Res<()> {
        debug_assert!(self.map.is_empty());
        debug_assert_eq!(self.size.filtered.len(), filters.len());

        let data = data::get()
            .chain_err(|| "while retrieving allocations")
            .chain_err(|| "while building new points")?;
        let (my_map, timestamp) = (&mut self.map, &self.timestamp);
        let start_time = data
            .start_time()
            .chain_err(|| "while building new points")?;
        let as_date = |duration: SinceStart| start_time.copy_add(duration);

        if init {
            map!(entry my_map, with filters => at as_date(SinceStart::zero()));
            ()
        }

        data.iter_new_since(
            timestamp,
            // New allocation.
            |alloc| {
                let index = filters.find_match(alloc);

                let toc_point_val =
                    map!(entry my_map, with filters => at as_date(alloc.toc.clone()));
                toc_point_val.get_mut(index).0 += alloc.size;
                Ok(())
            },
        )?;

        data.iter_dead_since(
            timestamp,
            // New dead allocation.
            |uids, tod| {
                for uid in uids {
                    let alloc = data
                        .get_alloc(uid)
                        .chain_err(|| "while handling new dead allocations")?;
                    let index = filters.find_match(alloc);

                    let toc_point_val = map!(entry my_map, with filters => at as_date(tod.clone()));
                    toc_point_val.get_mut(index).1 += alloc.size;
                }
                Ok(())
            },
        )?;

        self.timestamp = data.current_time().clone();

        Ok(())
    }
}
