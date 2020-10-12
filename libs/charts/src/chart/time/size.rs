//! Total size over time chart.

prelude! {}

#[cfg(any(test, feature = "server"))]
use point::TimeSizePoints;

/// Initial size value.
const INIT_SIZE_VALUE: u32 = 0;

/// Total size over time chart.
#[derive(Debug, Serialize, Deserialize)]
pub struct TimeSize {
    /// Timestamp of the last allocation registered.
    timestamp: time::SinceStart,
    /// Current total size.
    size: PointVal<u32>,
    /// Map used to construct the points.
    map: Map<Date, PointVal<(u32, u32)>>,
}

impl TimeSize {
    /// Default constructor.
    pub fn default(filters: &filter::Filters) -> Self {
        Self {
            timestamp: time::SinceStart::zero(),
            size: Self::init_size_point(filters),
            map: Map::new(),
        }
    }
}

#[cfg(any(test, feature = "server"))]
impl ChartExt for TimeSize {
    fn new_points(&mut self, filters: &mut Filters, init: bool) -> Res<Points> {
        self.get_allocs(filters, init)?;
        Ok(self.generate_points()?.into())
    }

    fn reset(&mut self, filters: &filter::Filters) {
        self.timestamp = time::SinceStart::zero();
        self.size = Self::init_size_point(filters);
        self.map.clear()
    }
}

impl TimeSize {
    /// Constructor.
    pub fn new(filters: &filter::Filters) -> Self {
        let timestamp = time::SinceStart::zero();
        let size = PointVal::new(0, filters);
        let map = Map::new();
        Self {
            timestamp,
            size,
            map,
        }
    }

    /// Initial size.
    fn init_size_point(filters: &filter::Filters) -> PointVal<u32> {
        PointVal::new(INIT_SIZE_VALUE, filters)
    }
}

#[cfg(any(test, feature = "server"))]
macro_rules! map {
    (entry $map:expr, with $filters:expr => at $date:expr) => {
        $map.entry($date).or_insert(PointVal::new((0, 0), $filters))
    };
}

/// # Helpers for point generation
#[cfg(any(test, feature = "server"))]
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
            let point_val = point_val.map(|uid, (to_add, to_sub)| {
                let new_val = (*self.size.get_mut_or(uid, INIT_SIZE_VALUE) + to_add) - to_sub;
                Ok(new_val)
            })?;
            self.size = point_val.clone();
            let point = Point::new(time, point_val);
            points.push(point)
        }

        Ok(points)
    }

    /// Retrieves all allocations since its internal timestamp.
    ///
    /// - assumes `self.map.is_empty()`;
    /// - new allocations will be in `self.map`;
    /// - updates the internal timestamp.
    fn get_allocs(&mut self, filters: &mut Filters, init: bool) -> Res<()> {
        debug_assert!(self.map.is_empty());

        let data = data::get()
            .chain_err(|| "while retrieving allocations")
            .chain_err(|| "while building new points")?;
        let (my_map, timestamp) = (&mut self.map, &self.timestamp);
        let start_time = data
            .start_time()
            .chain_err(|| "while building new points")?;
        let as_date = |duration: time::SinceStart| start_time.copy_add(duration);

        if init {
            map!(entry my_map, with filters => at as_date(time::SinceStart::zero()));
            ()
        }

        data.iter_new_since(
            timestamp,
            // New allocation.
            |alloc| {
                // Filter UID that matches the allocation, or catch-all.
                let uid = if let Some(uid) = filters.find_match(data.current_time(), alloc) {
                    uid::LineUid::Filter(uid)
                } else {
                    uid::LineUid::CatchAll
                };

                let toc_point_val =
                    map!(entry my_map, with filters => at as_date(alloc.toc.clone()));

                // Update the filter that matches the allocation.
                toc_point_val
                    .get_mut_or(uid, (INIT_SIZE_VALUE, INIT_SIZE_VALUE))
                    .0 += alloc.size;
                // Update the everything line.
                toc_point_val
                    .get_mut_or(uid::LineUid::Everything, (INIT_SIZE_VALUE, INIT_SIZE_VALUE))
                    .0 += alloc.size;
                Ok(())
            },
        )?;

        data.iter_dead_since(
            timestamp,
            // New dead allocation.
            |uids, tod| {
                for uid in uids {
                    // Potentially update the map, some filters are time-sensitive so matches can
                    // change.
                    // let uid = if let Some(uid) = filters.find_match(data.current_time(), alloc) {
                    //     uid::LineUid::Filter(uid)
                    // } else {
                    //     uid::LineUid::CatchAll
                    // };
                    let alloc = data
                        .get_alloc(uid)
                        .chain_err(|| "while handling new dead allocations")?;

                    let uid = if let Some(uid) = filters.find_dead_match(uid) {
                        uid::LineUid::Filter(uid)
                    } else {
                        uid::LineUid::CatchAll
                    };

                    let toc_point_val = map!(entry my_map, with filters => at as_date(tod.clone()));

                    toc_point_val.get_mut(uid)?.1 += alloc.size;
                    toc_point_val.get_mut(uid::LineUid::Everything)?.1 += alloc.size
                }
                Ok(())
            },
        )?;

        self.timestamp = data.current_time().clone();

        Ok(())
    }
}
