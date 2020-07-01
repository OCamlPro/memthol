//! Point representation.

use crate::common::*;

pub struct Range<T> {
    pub min: T,
    pub max: T,
}
impl<T> Range<T> {
    pub fn new(min: T, max: T) -> Self {
        Self { min, max }
    }

    pub fn map<NewT>(self, f: impl Fn(T) -> NewT) -> Range<NewT> {
        Range {
            min: f(self.min),
            max: f(self.max),
        }
    }
}

pub struct Ranges<X, Y> {
    pub x: Range<X>,
    pub y: Range<Y>,
}
impl<X, Y> Ranges<X, Y> {
    pub fn new(x: Range<X>, y: Range<Y>) -> Self {
        Self { x, y }
    }

    pub fn map<NewX, NewY>(
        self,
        fx: impl Fn(X) -> NewX,
        fy: impl Fn(Y) -> NewY,
    ) -> Ranges<NewX, NewY> {
        Ranges {
            x: self.x.map(fx),
            y: self.y.map(fy),
        }
    }
}

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

pub trait CoordExt
where
    Self: Clone,
    Self::Coord: 'static + std::fmt::Debug + Clone,
    Self::Range: 'static
        + plotters::coord::Ranged<ValueType = Self::Coord>
        + From<std::ops::Range<Self::Coord>>,
{
    type Coord;
    type Range;
    fn default_min() -> Self::Coord;
    fn default_max() -> Self::Coord;
}
impl CoordExt for Date {
    type Coord = Duration;
    type Range = plotters::coord::RangedDuration;
    fn default_min() -> Duration {
        Duration::seconds(0)
    }
    fn default_max() -> Duration {
        Duration::seconds(5)
    }
}
impl CoordExt for u32 {
    type Coord = u32;
    type Range = plotters::coord::RangedCoordu32;
    fn default_min() -> u32 {
        0
    }
    fn default_max() -> u32 {
        5
    }
}

pub trait RangesExt<X, Y> {
    fn ranges(&self, is_active: impl Fn(uid::LineUid) -> bool) -> Ranges<Option<X>, Option<Y>>;
}

pub trait ChartRender<X, Y>
where
    X: CoordExt,
    Y: CoordExt,
    Self: RangesExt<X, Y>,
{
    fn ranges_processor(ranges: Ranges<Option<X>, Option<Y>>) -> Res<Ranges<X, Y>>;

    fn coord_ranges_processor(ranges: &Ranges<X, Y>) -> Res<Ranges<X::Coord, Y::Coord>>;

    fn point_x_coord_processor(x_range: &Range<X>, x: &X) -> X::Coord;
    fn point_y_coord_processor(y_range: &Range<Y>, y: &Y) -> Y::Coord;

    fn x_label_formatter(val: &X::Coord) -> String;
    fn y_label_formatter(val: &Y::Coord) -> String;

    fn points(&self) -> std::slice::Iter<Point<X, Y>>;

    fn chart_render<'spec, DB>(
        &self,
        mut chart_builder: plotters::prelude::ChartBuilder<DB>,
        configure_chart_cxt: impl Fn(&mut plotters::chart::MeshStyle<X::Range, Y::Range, DB>),
        configure_style: impl Fn(&Color) -> plotters::style::ShapeStyle,
        is_active: impl Fn(uid::LineUid) -> bool,
        active_filters: impl Iterator<Item = &'spec filter::FilterSpec>,
    ) -> Res<()>
    where
        DB: plotters::prelude::DrawingBackend,
    {
        let opt_ranges = self.ranges(is_active);
        let raw_ranges = Self::ranges_processor(opt_ranges)?;
        let ranges = Self::coord_ranges_processor(&raw_ranges)?;

        use plotters::prelude::*;

        let x_range: X::Range = (ranges.x.min..ranges.x.max).into();
        let y_range: Y::Range = (ranges.y.min..ranges.y.max).into();

        // Alright, time to build the actual chart context used for drawing.
        let mut chart_cxt: ChartContext<DB, RangedCoord<X::Range, Y::Range>> = chart_builder
            .build_ranged(x_range, y_range)
            .map_err(|e| e.to_string())?;

        // Mesh configuration.
        {
            let mut mesh = chart_cxt.configure_mesh();

            // Apply caller's configuration.
            configure_chart_cxt(&mut mesh);

            // Set x/y formatters and draw this thing.
            mesh.x_label_formatter(&Self::x_label_formatter)
                .y_label_formatter(&Self::y_label_formatter)
                .draw()
                .map_err(|e| e.to_string())?;
        }

        // Time to add some points.
        for filter_spec in active_filters {
            let f_uid = filter_spec.uid();

            let points = self.points().filter_map(|point| {
                point.vals.map.get(&f_uid).map(|val| {
                    (
                        Self::point_x_coord_processor(&raw_ranges.x, &point.key),
                        Self::point_y_coord_processor(&raw_ranges.y, val),
                    )
                })
            });

            let style = configure_style(filter_spec.color());

            chart_cxt
                .draw_series(LineSeries::new(points, style))
                .map_err(|e| e.to_string())?;
        }

        Ok(())
    }
}

/// A list of points.
pub type PolyPoints<X, Y> = Vec<Point<X, Y>>;

impl<X, Y> RangesExt<X, Y> for PolyPoints<X, Y>
where
    X: PartialOrd + Clone,
    Y: PartialOrd + Clone,
{
    fn ranges(&self, is_active: impl Fn(uid::LineUid) -> bool) -> Ranges<Option<X>, Option<Y>> {
        let mut ranges: Ranges<Option<&X>, Option<&Y>> =
            Ranges::new(Range::new(None, None), Range::new(None, None));

        for point in self {
            // Update x-range.
            {
                if ranges.x.min.is_none() {
                    ranges.x.min = Some(&point.key)
                }

                // Points must be ordered by their key, let's make sure that's the case.
                debug_assert!(ranges.x.min.is_some());
                debug_assert!(ranges.x.min.map(|min| min <= &point.key).unwrap());
                // Now for max.
                debug_assert!(ranges.x.max.map(|max| max <= &point.key).unwrap_or(true));

                // Update max.
                ranges.x.max = Some(&point.key);
            }

            // Update y-range.
            for (_, val) in point.vals.map.iter().filter(|(uid, _)| is_active(**uid)) {
                // Update min.
                if let Some(min) = &mut ranges.y.min {
                    if val < min {
                        *min = val
                    }
                } else {
                    ranges.y.min = Some(val)
                }

                // Update max.
                if let Some(max) = &mut ranges.y.max {
                    if *max < val {
                        *max = val
                    }
                } else {
                    ranges.y.max = Some(val)
                }
            }
        }

        ranges.map(|x| x.cloned(), |y| y.cloned())
    }
}

/// Points representing size over time.
pub type TimeSizePoints = PolyPoints<Date, u32>;

/// Some points for a time chart.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TimePoints {
    Size(TimeSizePoints),
}

impl TimePoints {
    /// True if there are no points.
    pub fn is_empty(&self) -> bool {
        match self {
            Self::Size(points) => points.is_empty(),
        }
    }

    /// Extends some points with other points, returns `true` iff new points were added.
    ///
    /// Fails if the two kinds of points are not compatible.
    pub fn extend(&mut self, other: &mut Self) -> Res<bool> {
        let new_stuff = match (self, other) {
            (Self::Size(self_points), Self::Size(points)) => {
                let new_stuff = !points.is_empty();
                self_points.extend(points.drain(0..));
                new_stuff
            }
        };
        Ok(new_stuff)
    }
}

impl From<TimeSizePoints> for TimePoints {
    fn from(points: TimeSizePoints) -> Self {
        Self::Size(points)
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

    /// Extends some points with other points, returns `true` iff new points were added.
    ///
    /// Fails if the two kinds of points are not compatible.
    pub fn extend(&mut self, other: &mut Self) -> Res<bool> {
        match (self, other) {
            (Self::Time(self_points), Self::Time(points)) => self_points.extend(points),
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
