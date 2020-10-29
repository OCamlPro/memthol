//! Total size over time chart.

prelude! {}

use point::{Size, TimeSizePoints};

/// Initial size value.
const INIT_SIZE_VALUE: u32 = 0;

/// Total size over time chart.
#[derive(Debug, Serialize, Deserialize)]
pub struct TimeSize {
    /// UID of the last allocation, and timestamp of the last deallocation.
    last: Option<(uid::Alloc, time::SinceStart)>,
    /// Current total size.
    size: PointVal<Size>,
    /// Optional last timestamp.
    last_time_stamp: Option<time::SinceStart>,
    /// Points.
    points: TimeSizePoints,
}

impl TimeSize {
    /// Default constructor.
    pub fn default(filters: &filter::Filters) -> Self {
        Self {
            last: None,
            size: Self::init_size_point(filters),
            last_time_stamp: None,
            points: TimeSizePoints::with_capacity(32),
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
        time_windopt: &TimeWindopt,
    ) -> Res<Option<Points>> {
        self.do_it(filters, init, resolution, time_windopt)
            .map(|opt| opt.map(Points::from))
    }

    /// Resets (drops) all its points and re-initializes itself for `filters`.
    pub fn reset(&mut self, filters: &filter::Filters) {
        self.last_time_stamp = None;
        self.size = Self::init_size_point(filters);
    }
}

impl TimeSize {
    /// Constructor.
    pub fn new(filters: &filter::Filters) -> Self {
        let size = PointVal::new(INIT_SIZE_VALUE.into(), filters);
        Self {
            last: None,
            size,
            last_time_stamp: None,
            points: TimeSizePoints::with_capacity(32),
        }
    }

    /// Initial size.
    fn init_size_point(filters: &filter::Filters) -> PointVal<Size> {
        PointVal::new(INIT_SIZE_VALUE.into(), filters)
    }
}

/// # Helpers for point generation
#[cfg(any(test, feature = "server"))]
impl TimeSize {
    fn do_it(
        &mut self,
        filters: &mut Filters,
        init: bool,
        resolution: chart::settings::Resolution,
        time_windopt: &TimeWindopt,
    ) -> Res<Option<TimeSizePoints>> {
        let data = data::get()?;

        if !data.has_new_stuff_since(self.last.clone()) {
            return Ok(None);
        }

        let start_time = data.start_time()?;
        let time_window = time_windopt.to_time_window(|| *data.current_time());
        let min_time_spacing = data.current_time().clone() / (resolution.width / 5);
        let factor: u32 = convert(
            data.init()
                .map(|init| init.sampling_rate.factor * init.word_size / 8)
                .ok_or_else(|| "trying to construct points, but no data Init is available")?,
            "generate_points: factor",
        );

        debug_assert!(self.points.is_empty());
        if init {
            self.reset(filters);
        }
        self.points.push(Point::new(
            start_time + self.last_time_stamp.unwrap_or_else(time::SinceStart::zero),
            self.size.clone(),
        ));
        let points = &mut self.points;

        macro_rules! last_val_of {
            ($f_uid:expr) => {
                last_val_of!(@work(points.iter().rev()) $f_uid)
            };
            ($f_uid:expr, penultimate) => {{
                let mut points = points.iter().rev();
                let _ = points.next();
                last_val_of!(@work(points), $f_uid)
            }};
            (@work($rev_points:expr) $f_uid:expr) => {{
                let (mut last_val, mut last_everything_val) = (None, None);
                for point in $rev_points {
                    if last_val.is_none() {
                        last_val = point.vals.map.get($f_uid).cloned()
                    }
                    if last_everything_val.is_none() {
                        last_everything_val = point.vals.map.get(&uid::Line::Everything).cloned()
                    }
                    if last_val.is_some() && last_everything_val.is_some() {
                        break
                    }
                }
                (
                    last_val.unwrap_or_else(|| INIT_SIZE_VALUE.into()),
                    last_everything_val.unwrap_or_else(|| INIT_SIZE_VALUE.into()),
                )
            }};
        }

        let (last_time_stamp, last_size, last) =
            (&mut self.last_time_stamp, &self.size, self.last.clone());

        data.iter_new_events(last, |new_or_dead| {
            let (timestamp, size, add, alloc) = new_or_dead.as_ref().either(
                |alloc| (alloc.toc, factor * alloc.nsamples, true, alloc),
                |(tod, alloc)| (*tod, factor * alloc.nsamples, false, alloc),
            );
            let f_uid = if let Some(f_uid) = filters.find_match(data.current_time(), alloc) {
                uid::Line::Filter(f_uid)
            } else {
                uid::Line::CatchAll
            };

            match time_window.cmp(timestamp) {
                // Below the time-window, update the first point if any.
                base::RangeCmp::Below => {
                    debug_assert!(points.len() <= 1);

                    *last_time_stamp = Some(timestamp);
                    let last_map = if let Some(last) = points.last_mut() {
                        last.key = start_time + timestamp;
                        &mut last.vals.map
                    } else {
                        points.push(Point::new(start_time + timestamp, last_size.clone()));
                        let last = points
                            .last_mut()
                            .expect("`last_mut` after `push` cannot fail");
                        &mut last.vals.map
                    };

                    let val = last_map.entry(f_uid).or_insert(INIT_SIZE_VALUE.into());
                    if add {
                        val.size += size
                    } else {
                        val.size -= size
                    }

                    debug_assert!(points.len() == 1);
                    Ok(true)
                }

                // Inside the time-window.
                base::RangeCmp::Inside => {
                    let adjusted_date = start_time
                        + if let Some(last_time_stamp) = last_time_stamp.as_mut() {
                            if timestamp - *last_time_stamp < min_time_spacing {
                                last_time_stamp.clone()
                            } else {
                                *last_time_stamp = timestamp;
                                timestamp
                            }
                        } else {
                            *last_time_stamp = Some(timestamp);
                            timestamp
                        };

                    let (vals, repeat_previous) = if let Some(last) = points.last_mut() {
                        if last.key == adjusted_date {
                            (&mut last.vals.map, true)
                        } else {
                            let mut repeat = Point::new(adjusted_date, PointVal::empty());
                            let (last_val, last_everything_val) = last_val_of!(&f_uid);
                            let prev = repeat.vals.map.insert(f_uid, last_val);
                            debug_assert_eq!(prev, None);
                            let prev = repeat
                                .vals
                                .map
                                .insert(uid::Line::Everything, last_everything_val);
                            debug_assert_eq!(prev, None);

                            let new = repeat.clone();

                            points.push(repeat);

                            points.push(new);
                            let last = points
                                .last_mut()
                                .expect("`last_mut` after `push` cannot fail");
                            (&mut last.vals.map, true)
                        }
                    } else {
                        points.push(Point::new(adjusted_date, last_size.clone()));
                        let last = points
                            .last_mut()
                            .expect("`last_mut` after `push` cannot fail");
                        (&mut last.vals.map, false)
                    };

                    let val = vals.entry(f_uid).or_insert(INIT_SIZE_VALUE.into());
                    if add {
                        val.size += size
                    } else {
                        val.size -= size
                    }
                    let val = vals
                        .entry(uid::Line::Everything)
                        .or_insert(INIT_SIZE_VALUE.into());
                    if add {
                        val.size += size
                    } else {
                        val.size -= size
                    }

                    if repeat_previous && points.len() >= 2 {
                        let penultimate = points.len() - 2;
                        if points[penultimate].vals.map.get(&f_uid).is_none() {
                            let last_val = points[0..penultimate]
                                .iter()
                                .rev()
                                .find_map(|point| point.vals.map.get(&f_uid).cloned())
                                .unwrap_or_else(|| INIT_SIZE_VALUE.into());
                            let prev = points[penultimate].vals.map.insert(f_uid, last_val);
                            debug_assert_eq!(prev, None)
                        }
                    }

                    Ok(true)
                }

                // Above the range: generate the very last point and early exit.
                base::RangeCmp::Above => {
                    let end_time = start_time + time_window.ubound;
                    if let Some(last) = points.last() {
                        if last.key < end_time {
                            let mut last = last.clone();
                            last.key = end_time;
                            points.push(last)
                        }
                    }
                    Ok(false)
                }
            }
        })?;

        if let Some(ts) = last_time_stamp {
            if ts != data.current_time() {
                if let Some(mut last) = points.last().cloned() {
                    last.key = start_time + data.current_time();
                    points.push(last)
                }
            }
        }

        self.last = data.last_events();

        debug_assert!(!points.is_empty());
        // println!();
        // println!("points {{");
        // for point in points.iter() {
        //     print!("    {}:", point.key);
        //     for (uid, val) in &point.vals.map {
        //         print!(" {} -> {},", uid, val)
        //     }
        //     println!()
        // }
        // println!("}}");
        Ok(Some(points.drain(0..).collect()))
    }
}
