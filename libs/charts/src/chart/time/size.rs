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
        self.last = None;
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

        let time_window = time_windopt.to_time_window(|| *data.current_time());
        let min_time_spacing = data.current_time().clone() / (resolution.width / 5);

        debug_assert!(self.points.is_empty());
        if init {
            self.reset(filters);
        }

        self.points.push(Point::new(
            self.last_time_stamp.unwrap_or_else(|| {
                if let Some(lb) = time_windopt.lbound {
                    lb
                } else {
                    time::SinceStart::zero()
                }
            }),
            self.size.clone(),
        ));
        let points = &mut self.points;

        let (last_time_stamp, last_size, last) =
            (&mut self.last_time_stamp, &mut self.size, self.last.clone());

        macro_rules! map_do {
            ($f_uid:expr, _ => |ref mut $val:pat| $action:expr) => {{
                let $val = last_size
                    .map
                    .entry($f_uid)
                    .or_insert_with(|| INIT_SIZE_VALUE.into());
                $action;
                let $val = last_size
                    .map
                    .entry(uid::Line::Everything)
                    .or_insert_with(|| INIT_SIZE_VALUE.into());
                $action;
            }};
            ($f_uid:expr, $map:expr => |ref mut $val:pat| $action:expr) => {{
                let $val = $map.entry($f_uid).or_insert_with(|| INIT_SIZE_VALUE.into());
                $action;
                let $val = $map
                    .entry(uid::Line::Everything)
                    .or_insert_with(|| INIT_SIZE_VALUE.into());
                $action;
                map_do!($f_uid, _ => |ref mut $val| $action)
            }};
        }

        data.iter_new_events(last, |new_or_dead| {
            let (timestamp, size, add, alloc) = new_or_dead.as_ref().either(
                |alloc| (alloc.toc, alloc.real_size, true, alloc),
                |(tod, alloc)| (*tod, alloc.real_size, false, alloc),
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
                        &mut last.vals.map
                    } else {
                        points.push(Point::new(timestamp, last_size.clone()));
                        let last = points
                            .last_mut()
                            .expect("`last_mut` after `push` cannot fail");
                        &mut last.vals.map
                    };

                    map_do!(
                        f_uid, last_map => |ref mut val| if add {
                            val.size += size
                        } else {
                            val.size -= size
                        }
                    );

                    debug_assert!(points.len() == 1);
                    Ok(true)
                }

                // Inside the time-window.
                base::RangeCmp::Inside => {
                    let adjusted_timestamp = if let Some(last_time_stamp) = last_time_stamp.as_mut()
                    {
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
                        if last.key == adjusted_timestamp {
                            (&mut last.vals.map, true)
                        } else {
                            let mut repeat = Point::new(adjusted_timestamp, PointVal::empty());
                            let (last_val, last_everything_val) = (
                                last_size
                                    .map
                                    .get(&f_uid)
                                    .ok_or_else(|| format!("unexpected filter uid `{}`", f_uid))?
                                    .clone(),
                                last_size
                                    .map
                                    .get(&uid::Line::Everything)
                                    .ok_or_else(|| format!("unexpected filter uid `{}`", f_uid))?
                                    .clone(),
                            );
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
                        points.push(Point::new(adjusted_timestamp, last_size.clone()));
                        let last = points
                            .last_mut()
                            .expect("`last_mut` after `push` cannot fail");
                        (&mut last.vals.map, false)
                    };

                    map_do! {
                        f_uid, vals => |ref mut val| if add {
                            val.size += size
                        } else {
                            val.size -= size
                        }
                    }

                    if repeat_previous && points.len() >= 2 {
                        let penultimate = points.len() - 2;
                        if points[penultimate].vals.map.get(&f_uid).is_none() {
                            let prev = points[penultimate].vals.map.insert(
                                f_uid,
                                last_size
                                    .map
                                    .get(&f_uid)
                                    .ok_or_else(|| format!("unexpected filter uid `{}`", f_uid))?
                                    .clone(),
                            );
                            debug_assert_eq!(prev, None)
                        }
                    }

                    Ok(true)
                }

                // Above the range: generate the very last point and early exit.
                base::RangeCmp::Above => {
                    let end_time = time_window.ubound;
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
                let point = Point::new(*data.current_time(), self.size.clone());
                points.push(point)
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
