//! Total size over time chart.

prelude! {}

#[cfg(any(test, feature = "server"))]
use point::TimeSizePoints;

use point::Size;

/// Initial size value.
const INIT_SIZE_VALUE: u32 = 0;

/// Total size over time chart.
#[derive(Debug, Serialize, Deserialize)]
pub struct TimeSize {
    /// UID of the last allocation, and timestamp of the last deallocation.
    last: Option<(uid::Alloc, time::SinceStart)>,
    /// Current total size.
    size: PointVal<Size>,
    /// Map used to construct the points.
    map: BTMap<time::Date, PointVal<(u32, u32)>>,
    /// Optional last timestamp.
    last_time_stamp: Option<time::SinceStart>,
}

impl TimeSize {
    /// Default constructor.
    pub fn default(filters: &filter::Filters) -> Self {
        Self {
            last: None,
            size: Self::init_size_point(filters),
            map: BTMap::new(),
            last_time_stamp: None,
        }
    }
}

#[cfg(any(test, feature = "server"))]
impl TimeSize {
    /// Retrieves the new points since the last time it was called.
    pub fn new_points(
        &mut self,
        filters: &mut Filters,
        init: bool,
        resolution: chart::settings::Resolution,
    ) -> Res<Option<Points>> {
        let new_stuff = self.get_allocs(filters, init, resolution)?;
        Ok(if new_stuff {
            Some(self.generate_points()?.into())
        } else {
            None
        })
    }

    /// Resets (drops) all its points and re-initializes itself for `filters`.
    pub fn reset(&mut self, filters: &filter::Filters) {
        self.last_time_stamp = None;
        self.size = Self::init_size_point(filters);
        self.map.clear()
    }
}

impl TimeSize {
    /// Constructor.
    pub fn new(filters: &filter::Filters) -> Self {
        let size = PointVal::new(INIT_SIZE_VALUE.into(), filters);
        let map = BTMap::new();
        Self {
            last: None,
            size,
            map,
            last_time_stamp: None,
        }
    }

    /// Initial size.
    fn init_size_point(filters: &filter::Filters) -> PointVal<Size> {
        PointVal::new(INIT_SIZE_VALUE.into(), filters)
    }
}

/// Retrieves an entry in a `TimeSize` map, introduces a new empty point if needed.
#[cfg(any(test, feature = "server"))]
macro_rules! map {
    (entry $map:expr, with $filters:expr => at $date:expr) => {
        $map.entry($date).or_insert(PointVal::empty())
    };
    (entry $map:expr, with $filters:expr, $val:expr => at $date:expr) => {
        $map.entry($date).or_insert(PointVal::new($val, $filters))
    };
    (augment add $map:expr $(, $uid:expr => $val:expr)* $(,)?) => {{
        $(
            $map.get_mut_or($uid, (INIT_SIZE_VALUE, INIT_SIZE_VALUE)).0 += $val;
        )*
    }};
    (augment sub $map:expr $(, $uid:expr => $val:expr)* $(,)?) => {{
        $(
            $map.get_mut_or($uid, (INIT_SIZE_VALUE, INIT_SIZE_VALUE)).1 += $val;
        )*
    }};
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
        let map = std::mem::replace(&mut self.map, BTMap::new());

        let mut points: TimeSizePoints = Vec::with_capacity(self.map.len());

        let map_len = map.len();
        for (idx, (time, point_val)) in map.into_iter().enumerate() {
            let (is_first, is_last) = (idx == 0, idx + 1 == map_len);

            if !is_first {
                let pre_point = point_val.clone().filter_map(|uid, (to_add, to_sub)| {
                    if to_add + to_sub > 0 {
                        let val = self.size.get_mut_or(uid, INIT_SIZE_VALUE.into()).size;
                        Ok(Some(val.into()))
                    } else {
                        Ok(None)
                    }
                })?;
                if !pre_point.is_empty() {
                    points.push(Point::new(time, pre_point));
                }
            }

            let point_val = point_val.filter_map(|uid, (to_add, to_sub)| {
                if is_first || is_last || to_add > 0 || to_sub > 0 {
                    let old_value: &mut Size = self.size.get_mut_or(uid, INIT_SIZE_VALUE.into());
                    let sum = old_value.size + to_add;
                    let new_val = sum - to_sub;
                    old_value.size = new_val.clone();
                    Ok(Some(new_val.into()))
                } else {
                    Ok(None)
                }
            })?;
            if !point_val.is_empty() {
                points.push(Point::new(time, point_val))
            }
        }

        if points.len() == 1 {
            let mut before = points[0].clone();
            for value in before.vals.map.values_mut() {
                value.size = 0
            }
            let mut after = before.clone();

            after.key = after.key + time::SinceStart::one_sec();
            before.key = before.key - time::SinceStart::one_sec();

            points.insert(0, before);
            points.push(after)
        }

        Ok(points)
    }

    /// Retrieves all allocations since its internal timestamp.
    ///
    /// - assumes `self.map.is_empty()`;
    /// - new allocations will be in `self.map`;
    /// - updates the internal timestamp.
    fn get_allocs(
        &mut self,
        filters: &mut Filters,
        init: bool,
        resolution: chart::settings::Resolution,
    ) -> Res<bool> {
        debug_assert!(self.map.is_empty());
        debug_assert!(init == self.last.is_none());

        let data = data::get()
            .chain_err(|| "while retrieving allocations")
            .chain_err(|| "while building new points")?;
        let (my_map, last_time_stamp) = (&mut self.map, &mut self.last_time_stamp);
        let start_time = data
            .start_time()
            .chain_err(|| "while building new points")?;
        let as_date = |duration: time::SinceStart| start_time + duration;
        let factor: u32 = convert(
            data.init()
                .map(|init| init.sampling_rate.factor * init.word_size / 8)
                .ok_or_else(|| "trying to construct points, but no data Init is available")?,
            "generate_points: factor",
        );

        if init {
            map!(entry my_map, with filters, (0, 0) => at as_date(time::SinceStart::zero()));
            ()
        }

        let min_time_spacing = data.current_time().clone() / (resolution.width / 5);
        *last_time_stamp = None;

        let mut nu_last_new = None;
        let mut nu_last_dead = None;

        data.iter_new_events(self.last.clone(), |new_or_dead| {
            match new_or_dead {
                Either::Left(alloc) => {
                    nu_last_new = Some(alloc);

                    let adjusted_toc = if let Some(last_time_stamp) = last_time_stamp.as_mut() {
                        if alloc.toc - *last_time_stamp < min_time_spacing {
                            last_time_stamp.clone()
                        } else {
                            *last_time_stamp = alloc.toc.clone();
                            alloc.toc.clone()
                        }
                    } else {
                        *last_time_stamp = Some(alloc.toc.clone());
                        alloc.toc.clone()
                    };

                    let toc_point_val =
                        map!(entry my_map, with filters => at as_date(adjusted_toc));

                    let uid = if let Some(uid) = filters.find_match(data.current_time(), alloc) {
                        uid::Line::Filter(uid)
                    } else {
                        uid::Line::CatchAll
                    };

                    map!(
                        augment add toc_point_val,
                            uid => factor * alloc.nsamples,
                            uid::Line::Everything => factor * alloc.nsamples,
                    )
                }
                Either::Right((tod, alloc)) => {
                    nu_last_dead = Some(tod);
                    let adjusted_tod = if let Some(last_time_stamp) = last_time_stamp.as_mut() {
                        if tod - *last_time_stamp < min_time_spacing {
                            last_time_stamp.clone()
                        } else {
                            *last_time_stamp = tod.clone();
                            tod.clone()
                        }
                    } else {
                        *last_time_stamp = Some(tod.clone());
                        tod.clone()
                    };

                    let uid = if let Some(uid) = filters.find_dead_match(&alloc.uid) {
                        uid::Line::Filter(uid)
                    } else {
                        uid::Line::CatchAll
                    };

                    let tod_point_val =
                        map!(entry my_map, with filters => at as_date(adjusted_tod));
                    map!(
                        augment sub tod_point_val,
                            uid => factor * alloc.nsamples,
                            uid::Line::Everything => factor * alloc.nsamples,
                    )
                }
            }

            Ok(())
        })?;

        map!(entry my_map, with filters, (0, 0) => at as_date(*data.current_time()));

        let mut new_stuff = true;

        self.last = match (nu_last_new, nu_last_dead, self.last) {
            (Some(new), Some(dead_time), _) => Some((new.uid, dead_time)),
            (Some(new), None, Some((_, dead_time))) => Some((new.uid, dead_time)),
            (None, Some(dead_time), Some((new, _))) => Some((new, dead_time)),
            (Some(new), None, None) => Some((new.uid, time::SinceStart::zero())),
            // Nothing new this time.
            (None, None, last) => {
                new_stuff = false;
                self.map.clear();
                last
            }
            // Errors.
            (None, Some(time), None) => bail!(
                "saw an allocation death at {} before seeing any allocation",
                time,
            ),
        };

        Ok(new_stuff)
    }
}
