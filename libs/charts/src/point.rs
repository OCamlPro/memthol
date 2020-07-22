//! Point representation.

prelude! {}

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

    pub fn get(&self, uid: uid::LineUid) -> Res<&Val> {
        self.map
            .get(&uid)
            .ok_or_else(|| format!("unknown line uid `{}`", uid).into())
    }

    /// Retrieves the value for the *everything* filter.
    pub fn get_everything_val(&self) -> Res<&Val> {
        self.get(uid::LineUid::Everything)
    }

    /// Map over all values.
    pub fn map<Out>(self, mut f: impl FnMut(uid::LineUid, Val) -> Res<Out>) -> Res<PointVal<Out>> {
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
    fn zero() -> Self::Coord;
    fn is_zero(val: &Self::Coord) -> bool;
    fn default_min() -> Self::Coord;
    fn default_max() -> Self::Coord;
}
impl CoordExt for Date {
    type Coord = time::chrono::Duration;
    type Range = plotters::coord::RangedDuration;
    fn zero() -> time::chrono::Duration {
        time::chrono::Duration::seconds(0)
    }
    fn is_zero(dur: &time::chrono::Duration) -> bool {
        dur.is_zero()
    }
    fn default_min() -> time::chrono::Duration {
        time::chrono::Duration::seconds(0)
    }
    fn default_max() -> time::chrono::Duration {
        time::chrono::Duration::seconds(5)
    }
}
impl CoordExt for u32 {
    type Coord = u32;
    type Range = plotters::coord::RangedCoordu32;
    fn zero() -> u32 {
        0
    }
    fn is_zero(val: &u32) -> bool {
        *val == 0
    }
    fn default_min() -> u32 {
        0
    }
    fn default_max() -> u32 {
        5
    }
}
impl CoordExt for f32 {
    type Coord = f32;
    type Range = plotters::coord::RangedCoordf32;
    fn zero() -> f32 {
        0.0
    }
    fn is_zero(val: &f32) -> bool {
        *val == 0.0
    }
    fn default_min() -> f32 {
        0.0
    }
    fn default_max() -> f32 {
        5.0
    }
}

pub trait RatioExt {
    /// Returns the percentage (between `0` and `100`) of the ratio between `self` and `max`.
    fn ratio_wrt(&self, max: &Self) -> Res<f32>;
}
impl RatioExt for u32 {
    fn ratio_wrt(&self, max: &Self) -> Res<f32> {
        let (slf, max) = (*self, *max);
        if max == 0 || slf > max {
            bail!("cannot compute u32 ratio of {} w.r.t. {}", slf, max)
        }
        Ok(((slf * 100) as f32) / (max as f32))
    }
}
impl RatioExt for time::chrono::Duration {
    fn ratio_wrt(&self, max: &Self) -> Res<f32> {
        let (slf, max) = match (self.num_nanoseconds(), max.num_nanoseconds()) {
            (Some(slf), Some(max)) if max != 0 && slf <= max => (slf, max),
            _ => bail!("cannot compute Duration ratio of {} w.r.t. {}", self, max),
        };
        Ok(((slf * 100) as f32) / (max as f32))
    }
}

pub trait RangesExt<X, Y> {
    fn ranges(&self, is_active: impl Fn(uid::LineUid) -> bool) -> Ranges<Option<X>, Option<Y>>;
}

pub trait PointValExt<Val>
where
    Val: CoordExt,
{
    fn val_range_processor(range: Range<Option<Val>>) -> Res<Range<Val>>;
    fn val_coord_range_processor(range: &Range<Val>) -> Res<Range<Val::Coord>>;
    fn val_coord_processor(range: &Range<Val>, x: &Val) -> Val::Coord;
    fn val_label_formatter(val: &Val::Coord) -> String;
}

impl<X, Y> ChartRender<X, Y> for PolyPoints<X, Y>
where
    X: CoordExt,
    Y: CoordExt,
    Self: RangesExt<X, Y> + PointValExt<X> + PointValExt<Y>,
{
    fn points(&self) -> std::slice::Iter<Point<X, Y>> {
        self.iter()
    }
}

pub trait StyleExt {
    fn mesh_conf<X, Y, DB>(
        &self,
        configure_mesh: &mut plotters::chart::MeshStyle<X::Range, Y::Range, DB>,
    ) where
        X: CoordExt,
        Y: CoordExt,
        DB: plotters::drawing::DrawingBackend;
    fn shape_conf(&self, color: &Color) -> plotters::style::ShapeStyle;
}

pub trait ChartRender<X, Y>
where
    X: CoordExt,
    Y: CoordExt,
    Self: RangesExt<X, Y> + PointValExt<X> + PointValExt<Y>,
{
    fn ranges_processor(ranges: Ranges<Option<X>, Option<Y>>) -> Res<Ranges<X, Y>> {
        Ok(Ranges {
            x: Self::val_range_processor(ranges.x)?,
            y: Self::val_range_processor(ranges.y)?,
        })
    }

    fn coord_ranges_processor(ranges: &Ranges<X, Y>) -> Res<Ranges<X::Coord, Y::Coord>> {
        Ok(Ranges {
            x: Self::val_coord_range_processor(&ranges.x)?,
            y: Self::val_coord_range_processor(&ranges.y)?,
        })
    }

    fn x_coord_processor(x_range: &Range<X>, x: &X) -> X::Coord {
        Self::val_coord_processor(x_range, x)
    }
    fn y_coord_processor(y_range: &Range<Y>, y: &Y) -> Y::Coord {
        Self::val_coord_processor(y_range, y)
    }

    fn x_label_formatter(val: &X::Coord) -> String {
        <Self as PointValExt<X>>::val_label_formatter(val)
    }
    fn y_label_formatter(val: &Y::Coord) -> String {
        <Self as PointValExt<Y>>::val_label_formatter(val)
    }

    fn points(&self) -> std::slice::Iter<Point<X, Y>>;

    fn chart_render<'spec, DB>(
        &self,
        mut chart_builder: plotters::prelude::ChartBuilder<DB>,
        style_conf: &impl StyleExt,
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
            style_conf.mesh_conf::<X, Y, DB>(&mut mesh);

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
                        Self::x_coord_processor(&raw_ranges.x, &point.key),
                        Self::y_coord_processor(&raw_ranges.y, val),
                    )
                })
            });

            let style = style_conf.shape_conf(filter_spec.color());

            chart_cxt
                .draw_series(LineSeries::new(points, style))
                .map_err(|e| e.to_string())?;
        }

        Ok(())
    }

    fn stacked_area_chart_render<'spec, DB>(
        &self,
        mut chart_builder: plotters::prelude::ChartBuilder<DB>,
        style_conf: &impl StyleExt,
        is_active: impl Fn(uid::LineUid) -> bool,
        active_filters: impl Iterator<Item = &'spec filter::FilterSpec> + Clone,
    ) -> Res<()>
    where
        DB: plotters::prelude::DrawingBackend,
        Y::Coord: RatioExt
            + std::ops::Add<Output = Y::Coord>
            + std::ops::Sub<Output = Y::Coord>
            + Clone
            + PartialOrd
            + Ord
            + PartialEq,
        Self: ChartRender<X, Y>,
    {
        let opt_ranges = self.ranges(&is_active);
        let raw_ranges = Self::ranges_processor(opt_ranges)?;
        let ranges = Self::coord_ranges_processor(&raw_ranges)?;

        use plotters::prelude::*;

        let x_range: X::Range = (ranges.x.min..ranges.x.max).into();
        let y_range: RangedCoordf32 = (0. ..100.).into();

        // Alright, time to build the actual chart context used for drawing.
        let mut chart_cxt: ChartContext<DB, RangedCoord<X::Range, RangedCoordf32>> = chart_builder
            .build_ranged(x_range, y_range)
            .map_err(|e| e.to_string())?;

        // Mesh configuration.
        {
            let mut mesh = chart_cxt.configure_mesh();

            // Apply caller's configuration.
            style_conf.mesh_conf::<X, f32, DB>(&mut mesh);

            // Set x/y formatters and draw this thing.
            mesh.x_label_formatter(&Self::x_label_formatter)
                // .y_label_formatter(&Self::y_label_formatter)
                .draw()
                .map_err(|e| e.to_string())?;
        }

        // Stores all the maximum values and the sum of the values for each filter that has been
        // processed so far (used in the loop below).
        let mut memory: Vec<_> = self
            .points()
            .map(|point| {
                let mut max = Y::zero();
                for (uid, val) in &point.vals.map {
                    if !uid.is_everything() && is_active(*uid) {
                        max = max + Self::y_coord_processor(&raw_ranges.y, val);
                    }
                }
                (max, 0.0f32)
            })
            .collect();

        // Invariant: `memory.len()` is the same length as `self.points()`.

        // Time to add some points.
        for filter_spec in active_filters.clone() {
            let f_uid = filter_spec.uid();
            if f_uid.is_everything() {
                continue;
            }

            let points = self
                .points()
                .enumerate()
                .filter_map(|(point_index, point)| {
                    let (ref max, ref mut sum) = memory[point_index];
                    point.vals.map.get(&f_uid).map(|val| {
                        let y_val = Self::y_coord_processor(&raw_ranges.y, val);
                        assert!(*max >= y_val);

                        let y_val = if Y::is_zero(max) {
                            f32::zero()
                        } else {
                            y_val.ratio_wrt(max).expect(
                                "\
                                    logical error, maximum value for stacked area chart is not \
                                    compatible with one of the individual values\
                                ",
                            )
                        };
                        println!("y_val: {}, sum: {}", y_val, sum);
                        *sum = *sum + y_val;
                        (Self::x_coord_processor(&raw_ranges.x, &point.key), *sum)
                    })
                });

            let style = style_conf.shape_conf(filter_spec.color()).filled();

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

impl<Y> PointValExt<Date> for PolyPoints<Date, Y> {
    fn val_range_processor(range: Range<Option<Date>>) -> Res<Range<Date>> {
        match (range.min, range.max) {
            (Some(min), Some(max)) => Ok(Range { min, max }),
            (min, max) => bail!("failed to compute x-range: {:?}, {:?}", min, max),
        }
    }
    fn val_coord_range_processor(range: &Range<Date>) -> Res<Range<<Date as CoordExt>::Coord>> {
        let min = time::chrono::Duration::seconds(0);
        let max = range.max.date().clone() - range.min.date().clone();
        Ok(Range { min, max })
    }
    fn val_coord_processor(range: &Range<Date>, x: &Date) -> <Date as CoordExt>::Coord {
        x.date().clone() - range.min.date().clone()
    }
    fn val_label_formatter(date: &<Date as CoordExt>::Coord) -> String {
        let date = date.to_std().unwrap();
        let mut secs = date.as_secs();
        let mut mins = secs / 60;
        secs = secs - mins * 60;
        let hours = mins / 60;
        mins = mins - hours * 60;
        let mut s = String::with_capacity(10);
        use std::fmt::Write;
        if hours > 0 {
            write!(&mut s, "{}h", hours).unwrap()
        }
        if mins > 0 {
            write!(&mut s, "{}m", mins).unwrap()
        }
        write!(&mut s, "{}", secs).unwrap();
        let millis = date.subsec_millis();
        if millis != 0 {
            write!(&mut s, ".{}", millis).unwrap()
        }
        write!(&mut s, "s").unwrap();
        s.shrink_to_fit();
        s
    }
}

impl<X> PointValExt<u32> for PolyPoints<X, u32> {
    fn val_range_processor(range: Range<Option<u32>>) -> Res<Range<u32>> {
        Ok(Range {
            min: range.min.unwrap_or_else(u32::default_min),
            max: range.max.unwrap_or_else(u32::default_max),
        })
    }
    fn val_coord_range_processor(range: &Range<u32>) -> Res<Range<<u32 as CoordExt>::Coord>> {
        let default_max = u32::default_max();
        Ok(Range {
            min: range.min,
            max: if range.max < default_max {
                default_max
            } else {
                range.max
            },
        })
    }
    fn val_coord_processor(_range: &Range<u32>, x: &u32) -> <u32 as CoordExt>::Coord {
        *x
    }
    fn val_label_formatter(val: &<u32 as CoordExt>::Coord) -> String {
        num_fmt::str_do(val, |str| str.to_string())
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

    pub fn chart_render<'spec, DB>(
        &self,
        chart_builder: plotters::prelude::ChartBuilder<DB>,
        style_conf: &impl StyleExt,
        is_active: impl Fn(uid::LineUid) -> bool,
        active_filters: impl Iterator<Item = &'spec filter::FilterSpec>,
    ) -> Res<()>
    where
        DB: plotters::prelude::DrawingBackend,
    {
        match self {
            Self::Size(points) => {
                points.chart_render(chart_builder, style_conf, is_active, active_filters)
            }
        }
    }

    pub fn stacked_area_chart_render<'spec, DB>(
        &self,
        chart_builder: plotters::prelude::ChartBuilder<DB>,
        style_conf: &impl StyleExt,
        is_active: impl Fn(uid::LineUid) -> bool,
        active_filters: impl Iterator<Item = &'spec filter::FilterSpec> + Clone,
    ) -> Res<()>
    where
        DB: plotters::prelude::DrawingBackend,
    {
        match self {
            Self::Size(points) => points.stacked_area_chart_render(
                chart_builder,
                style_conf,
                is_active,
                active_filters,
            ),
        }
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

    pub fn chart_render<'spec, DB>(
        &self,
        chart_builder: plotters::prelude::ChartBuilder<DB>,
        style_conf: &impl StyleExt,
        is_active: impl Fn(uid::LineUid) -> bool,
        active_filters: impl Iterator<Item = &'spec filter::FilterSpec>,
    ) -> Res<()>
    where
        DB: plotters::prelude::DrawingBackend,
    {
        match self {
            Self::Time(points) => {
                points.chart_render(chart_builder, style_conf, is_active, active_filters)
            }
        }
    }

    pub fn stacked_area_chart_render<'spec, DB>(
        &self,
        chart_builder: plotters::prelude::ChartBuilder<DB>,
        style_conf: &impl StyleExt,
        is_active: impl Fn(uid::LineUid) -> bool,
        active_filters: impl Iterator<Item = &'spec filter::FilterSpec> + Clone,
    ) -> Res<()>
    where
        DB: plotters::prelude::DrawingBackend,
    {
        match self {
            Self::Time(points) => points.stacked_area_chart_render(
                chart_builder,
                style_conf,
                is_active,
                active_filters,
            ),
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
