//! Point representation.

prelude! {}

/// A point value.
///
/// Stores a value for each filter, and the value for the catch-all filter.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PointVal<Val> {
    /// Values for filter lines.
    pub map: BTMap<uid::Line, Val>,
}
impl<Val> PointVal<Val> {
    /// Constructor.
    pub fn new(default: Val, filters: &filter::Filters) -> Self
    where
        Val: Clone,
    {
        let mut map = BTMap::new();
        map.insert(uid::Line::CatchAll, default.clone());
        map.insert(uid::Line::Everything, default.clone());
        for filter in filters.filters() {
            map.insert(uid::Line::Filter(filter.uid()), default.clone());
        }
        Self { map }
    }

    /// Empty constructor.
    pub fn empty() -> Self {
        Self { map: BTMap::new() }
    }

    /// True if the inner map is empty.
    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    /// Immutable ref over some value.
    pub fn get_mut_or(&mut self, uid: uid::Line, default: Val) -> &mut Val {
        self.map.entry(uid).or_insert(default)
    }

    /// Mutable ref over some value.
    pub fn get_mut(&mut self, uid: uid::Line) -> Res<&mut Val> {
        self.map
            .get_mut(&uid)
            .ok_or_else(|| format!("unknown line uid `{}`", uid).into())
    }

    /// Retrieves the value for a filter.
    pub fn get(&self, uid: uid::Line) -> Res<&Val> {
        self.map
            .get(&uid)
            .ok_or_else(|| format!("unknown line uid `{}`", uid).into())
    }

    /// Retrieves the value for the *everything* filter.
    pub fn get_everything_val(&self) -> Res<&Val> {
        self.get(uid::Line::Everything)
    }

    /// Map over all values.
    pub fn map<Out>(self, mut f: impl FnMut(uid::Line, Val) -> Res<Out>) -> Res<PointVal<Out>> {
        let mut map = BTMap::new();
        for (uid, val) in self.map {
            map.insert(uid, f(uid, val)?);
        }
        let res = PointVal { map };
        Ok(res)
    }

    /// Filter-map over all values.
    pub fn filter_map<Out: PartialEq + fmt::Debug>(
        self,
        mut f: impl FnMut(uid::Line, Val) -> Res<Option<Out>>,
    ) -> Res<PointVal<Out>> {
        let mut map = BTMap::new();
        for (uid, val) in self.map {
            if let Some(res) = f(uid, val)? {
                let prev = map.insert(uid, res);
                debug_assert_eq!(prev, None);
            }
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

/// Some ranges for a x-axis/y-axis graph.
pub struct Ranges<X, Y> {
    /// X-axis range.
    pub x: Range<X>,
    /// Y-axis range.
    pub y: Range<Y>,
}
impl<X, Y> Ranges<X, Y> {
    /// Constructor.
    pub fn new(x: Range<X>, y: Range<Y>) -> Self {
        Self { x, y }
    }

    /// Map over the bounds of both ranges.
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

/// Size quantity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Size {
    /// Actual size value.
    pub size: u64,
}
impl Size {
    /// Constructor.
    pub fn new(size: impl Into<u64>) -> Self {
        Self { size: size.into() }
    }
}
base::implement! {
    impl Size {
        Display {
            |&self, fmt| self.size.fmt(fmt),
        }
        From {
            from u64 => |size| Self::new(size),
        }
    }
}

/// Extension trait for coordinates.
///
/// Note that the type of the values appearing in a point are not necessarily the same type expected
/// by the graph. Hence, this trait has a [`Coord`] associated type specifying the type of the
/// actual values as expected by a graph.
///
/// [`Coord`]: #associatedtype.Coord (Coord associated type)
pub trait CoordExt
where
    Self: Clone,
    Self::Coord: 'static + std::fmt::Debug + Clone,
    Self::Range: 'static
        + coord::ValueFormatter<Self::Coord>
        + coord::Ranged<ValueType = Self::Coord>
        + From<std::ops::Range<Self::Coord>>,
{
    /// Type of the values as they will be passed to the graph.
    type Coord;
    /// Type of coordinate ranges.
    type Range;

    /// Generates a default `Self` value.
    fn default_val() -> Self;

    /// Zero value for the actual coordinates.
    fn zero() -> Self::Coord;
    /// True if a coordinate value is zero.
    fn is_zero(val: &Self::Coord) -> bool;

    /// Default minimum value for a range of coordinates.
    fn default_min() -> Self::Coord;
    /// Default maximum value for a range of coordinates.
    fn default_max() -> Self::Coord;
}

impl CoordExt for time::Date {
    type Coord = time::chrono::Duration;
    type Range = coord::RangedDuration;
    fn default_val() -> Self {
        time::Date::from_timestamp(0, 0)
    }
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

impl CoordExt for time::SinceStart {
    type Coord = time::chrono::Duration;
    type Range = coord::RangedDuration;
    fn default_val() -> Self {
        time::SinceStart::zero()
    }
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

impl CoordExt for u64 {
    type Coord = u64;
    type Range = coord::RangedCoordu64;
    fn default_val() -> Self {
        0
    }
    fn zero() -> u64 {
        0
    }
    fn is_zero(val: &u64) -> bool {
        *val == 0
    }
    fn default_min() -> u64 {
        0
    }
    fn default_max() -> u64 {
        5
    }
}

impl CoordExt for Size {
    type Coord = u64;
    type Range = coord::RangedCoordu64;
    fn default_val() -> Self {
        0.into()
    }
    fn zero() -> u64 {
        0
    }
    fn is_zero(val: &u64) -> bool {
        *val == 0
    }
    fn default_min() -> u64 {
        0
    }
    fn default_max() -> u64 {
        5
    }
}

impl CoordExt for f32 {
    type Coord = f32;
    type Range = coord::RangedCoordf32;
    fn default_val() -> Self {
        0.0
    }
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

/// Extension trait allowing to compute ratios.
pub trait RatioExt {
    /// Returns the percentage (between `0` and `100`) of the ratio between `self` and `max`.
    fn ratio_wrt(&self, max: &Self) -> Res<f32>;
}
impl RatioExt for u64 {
    fn ratio_wrt(&self, max: &Self) -> Res<f32> {
        let (slf, max) = (*self, *max);
        if max == 0 || slf > max {
            bail!("cannot compute u64 ratio of {} w.r.t. {}", slf, max)
        }
        Ok(((slf * 100) as f32) / (max as f32))
    }
}
impl RatioExt for Size {
    fn ratio_wrt(&self, max: &Self) -> Res<f32> {
        let (slf, max) = (self.size, max.size);
        if max == 0 || slf > max {
            bail!("cannot compute u64 ratio of {} w.r.t. {}", slf, max)
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

/// Ranges extension trait.
pub trait RangesExt<X, Y> {
    /// Computes ranges from itself, given which filters are active.
    fn ranges(&self, is_active: impl Fn(uid::Line) -> bool) -> Ranges<Option<X>, Option<Y>>;
}

/// Extension trait for point values.
pub trait PointValExt<Val>
where
    Val: CoordExt,
{
    /// Processes a range over optional bounds to yield a range proper.
    fn val_range_processor(range: Range<Option<Val>>) -> Res<Range<Val>>;
    /// Turns a range over point values into a range over coordinates.
    fn val_coord_range_processor(range: &Range<Val>) -> Res<Range<Val::Coord>>;
    /// Processes a point value to yield a coordinate value.
    fn val_coord_processor(range: &Range<Val>, x: &Val) -> Val::Coord;
    /// Formatter for the axis labels.
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

/// Mesh configuration extension trait.
pub trait StyleExt {
    /// Applies a mesh configuration.
    fn mesh_conf<X, Y, DB>(
        &self,
        configure_mesh: &mut plotters::chart::MeshStyle<X::Range, Y::Range, DB>,
    ) where
        X: CoordExt,
        Y: CoordExt,
        DB: plotters::prelude::DrawingBackend;

    /// Creates a shape style.
    fn shape_conf(&self, color: &Color) -> plotters::style::ShapeStyle;
}

/// Chart-rendering trait.
pub trait ChartRender<X, Y>
where
    X: CoordExt,
    Y: CoordExt,
    Self: RangesExt<X, Y> + PointValExt<X> + PointValExt<Y>,
{
    /// Processes the ranges for both axis.
    fn ranges_processor(ranges: Ranges<Option<X>, Option<Y>>) -> Res<Ranges<X, Y>> {
        Ok(Ranges {
            x: Self::val_range_processor(ranges.x)?,
            y: Self::val_range_processor(ranges.y)?,
        })
    }

    /// Turns point-value ranges into coordinate ranges.
    fn coord_ranges_processor(ranges: &Ranges<X, Y>) -> Res<Ranges<X::Coord, Y::Coord>> {
        Ok(Ranges {
            x: Self::val_coord_range_processor(&ranges.x)?,
            y: Self::val_coord_range_processor(&ranges.y)?,
        })
    }

    /// Processes a x-axis point value to yield a x-axis coordinate.
    fn x_coord_processor(x_range: &Range<X>, x: &X) -> X::Coord {
        Self::val_coord_processor(x_range, x)
    }
    /// Processes a y-axis point value to yield a y-axis coordinate.
    fn y_coord_processor(y_range: &Range<Y>, y: &Y) -> Y::Coord {
        Self::val_coord_processor(y_range, y)
    }

    /// X-axis label formatter.
    fn x_label_formatter(val: &X::Coord) -> String {
        <Self as PointValExt<X>>::val_label_formatter(val)
    }
    /// Y-axis label formatter.
    fn y_label_formatter(val: &Y::Coord) -> String {
        <Self as PointValExt<Y>>::val_label_formatter(val)
    }

    /// Yields the actual points.
    fn points(&self) -> std::slice::Iter<Point<X, Y>>;

    /// Renders some points on a graph.
    fn render<'spec, DB>(
        &self,
        settings: &settings::Chart,
        chart_builder: plotters::prelude::ChartBuilder<DB>,
        style_conf: &impl StyleExt,
        is_active: impl Fn(uid::Line) -> bool,
        active_filters: impl Iterator<Item = &'spec filter::FilterSpec> + Clone,
    ) -> Res<()>
    where
        DB: plotters::prelude::DrawingBackend,
        X: fmt::Display,
        Y::Coord: RatioExt
            + std::ops::Add<Output = Y::Coord>
            + std::ops::Sub<Output = Y::Coord>
            + Clone
            + fmt::Display
            + PartialOrd
            + Ord
            + PartialEq,
    {
        use chart::settings::DisplayMode;
        match settings.display_mode() {
            DisplayMode::Normal => self.chart_render(
                settings,
                chart_builder,
                style_conf,
                is_active,
                active_filters,
            ),
            DisplayMode::StackedArea => self.chart_render_stacked_area(
                settings,
                chart_builder,
                style_conf,
                is_active,
                active_filters,
            ),
            DisplayMode::StackedAreaPercent => self.chart_render_stacked_area_percent(
                settings,
                chart_builder,
                style_conf,
                is_active,
                active_filters,
            ),
        }
    }

    /// Normal display mode rendering.
    fn chart_render<'spec, DB>(
        &self,
        _settings: &settings::Chart,
        mut chart_builder: plotters::prelude::ChartBuilder<DB>,
        style_conf: &impl StyleExt,
        is_active: impl Fn(uid::Line) -> bool,
        active_filters: impl Iterator<Item = &'spec filter::FilterSpec>,
    ) -> Res<()>
    where
        DB: plotters::prelude::DrawingBackend,
    {
        let opt_ranges = self.ranges(is_active);
        let raw_ranges = Self::ranges_processor(opt_ranges)?;
        let ranges = Self::coord_ranges_processor(&raw_ranges)?;

        use plotters::prelude::*;

        let x_range: X::Range = (ranges.x.lbound..ranges.x.ubound).into();
        let y_range: Y::Range = (ranges.y.lbound..ranges.y.ubound).into();

        // Alright, time to build the actual chart context used for drawing.
        let mut chart_cxt: ChartContext<DB, coord::Cartesian2d<X::Range, Y::Range>> = chart_builder
            .build_cartesian_2d(x_range, y_range)
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
    /// Stacked area rendering.
    fn chart_render_stacked_area<'spec, DB>(
        &self,
        settings: &settings::Chart,
        chart_builder: plotters::prelude::ChartBuilder<DB>,
        style_conf: &impl StyleExt,
        is_active: impl Fn(uid::Line) -> bool,
        active_filters: impl Iterator<Item = &'spec filter::FilterSpec> + Clone,
    ) -> Res<()>
    where
        DB: plotters::prelude::DrawingBackend,
        X: fmt::Display,
        Y::Coord: RatioExt
            + std::ops::Add<Output = Y::Coord>
            + std::ops::Sub<Output = Y::Coord>
            + Clone
            + fmt::Display
            + PartialOrd
            + Ord
            + PartialEq,
    {
        self.chart_render_stacked_area_custom(
            settings,
            chart_builder,
            style_conf,
            is_active,
            active_filters,
            Y::zero,
            |y_val, _y_max| y_val,
            |lbound, ubound| (lbound..ubound).into(),
            Self::y_label_formatter,
            Y::default_val,
        )
    }

    /// Percent stacked area rendering.
    fn chart_render_stacked_area_percent<'spec, DB>(
        &self,
        settings: &settings::Chart,
        chart_builder: plotters::prelude::ChartBuilder<DB>,
        style_conf: &impl StyleExt,
        is_active: impl Fn(uid::Line) -> bool,
        active_filters: impl Iterator<Item = &'spec filter::FilterSpec> + Clone,
    ) -> Res<()>
    where
        DB: plotters::prelude::DrawingBackend,
        X: fmt::Display,
        Y::Coord: RatioExt
            + std::ops::Add<Output = Y::Coord>
            + std::ops::Sub<Output = Y::Coord>
            + Clone
            + fmt::Display
            + PartialOrd
            + Ord
            + PartialEq,
    {
        self.chart_render_stacked_area_custom(
            settings,
            chart_builder,
            style_conf,
            is_active,
            active_filters,
            || 0.0f32,
            |y_val, y_max| {
                if Y::is_zero(y_max) {
                    f32::zero()
                } else {
                    y_val.ratio_wrt(y_max).expect(
                        "\
                                    logical error, maximum value for stacked area chart is not \
                                    compatible with one of the individual values\
                                ",
                    )
                }
            },
            |_min, _max| (0.0..100.0).into(),
            |val| format!("{:.2}%", val),
            f32::default_val,
        )
    }

    /// Stacked area rendering.
    fn chart_render_stacked_area_custom<'spec, DB, RealY: CoordExt>(
        &self,
        _settings: &settings::Chart,
        mut chart_builder: plotters::prelude::ChartBuilder<DB>,
        style_conf: &impl StyleExt,
        is_active: impl Fn(uid::Line) -> bool,
        active_filters: impl Iterator<Item = &'spec filter::FilterSpec> + Clone,
        zero: impl Fn() -> RealY::Coord,
        compute_from_val_and_max: impl Fn(Y::Coord, &Y::Coord) -> RealY::Coord,
        range_do: impl Fn(Y::Coord, Y::Coord) -> RealY::Range,
        label_formatter: impl Fn(&RealY::Coord) -> String,
        _type_inference_help: fn() -> RealY,
    ) -> Res<()>
    where
        DB: plotters::prelude::DrawingBackend,
        X: fmt::Display,
        Y::Coord: RatioExt
            + std::ops::Add<Output = Y::Coord>
            + std::ops::Sub<Output = Y::Coord>
            + Clone
            + fmt::Display
            + PartialOrd
            + Ord
            + PartialEq,
        RealY::Coord: ops::Add<RealY::Coord, Output = RealY::Coord> + fmt::Display,
    {
        let is_active = |uid: uid::Line| !uid.is_everything() && is_active(uid);
        let active_filters = active_filters.filter(|uid| !uid.is_everything());

        let opt_ranges = self.ranges(&is_active);
        let raw_ranges = Self::ranges_processor(opt_ranges)?;
        let ranges = Self::coord_ranges_processor(&raw_ranges)?;

        use plotters::prelude::*;

        let (y_min, mut y_max) = (ranges.y.lbound, Y::zero());

        // Stores all the maximum values and the result quantity for each filter that has been
        // processed so far (used in the loop below).
        let mut memory: Vec<_> = {
            // Remembers the last values for each filter.
            let mut last: HMap<_, _> = active_filters
                .clone()
                .map(|uid| (uid.uid(), Y::zero()))
                .collect();
            // Build the memory.
            self.points()
                .map(|point| {
                    let mut max = Y::zero();
                    for f in active_filters.clone() {
                        let f_uid = f.uid();
                        let y_val = if let Some(val) = point.vals.map.get(&f_uid) {
                            let y_val = Self::y_coord_processor(&raw_ranges.y, val);
                            let _prev = last.insert(f_uid, y_val.clone());
                            y_val
                        } else {
                            last.get(&f_uid)
                                .unwrap_or_else(|| panic!("unexpected filter UID {}", f_uid))
                                .clone()
                        };

                        max = max + y_val;
                    }
                    if max > y_max {
                        y_max = max.clone()
                    }
                    (max, zero())
                })
                .collect()
        };

        let x_range: X::Range = (ranges.x.lbound..ranges.x.ubound).into();
        let y_range = range_do(y_min, y_max);

        // Alright, time to build the actual chart context used for drawing.
        let mut chart_cxt: ChartContext<DB, coord::Cartesian2d<X::Range, RealY::Range>> =
            chart_builder
                .build_cartesian_2d(x_range, y_range)
                .map_err(|e| e.to_string())?;

        // Mesh configuration.
        {
            let mut mesh = chart_cxt.configure_mesh();

            // Apply caller's configuration.
            style_conf.mesh_conf::<X, RealY, DB>(&mut mesh);

            // Set x/y formatters and draw this thing.
            mesh.x_label_formatter(&Self::x_label_formatter)
                .y_label_formatter(&label_formatter)
                .draw()
                .map_err(|e| e.to_string())?;
        }

        // Invariant: `memory.len()` is the same length as `self.points()`.

        // Time to add some points.
        for filter_spec in active_filters.clone() {
            let f_uid = filter_spec.uid();
            if f_uid.is_everything() {
                continue;
            }
            let mut prev = Y::zero();

            let points = self.points().enumerate().map(|(point_index, point)| {
                let (ref max, ref mut sum) = memory[point_index];
                let y_val = point
                    .vals
                    .map
                    .get(&f_uid)
                    .map(|val| Self::y_coord_processor(&raw_ranges.y, val))
                    .unwrap_or_else(|| prev.clone());
                prev = y_val.clone();
                if *max < y_val {
                    log::error!("expected `max < y_val");
                    log::error!("max: {}, y_val: {}", max, y_val);
                    log::error!("@ {} on filter {}", point.key, f_uid);
                }
                assert!(*max >= y_val);
                *sum = sum.clone() + compute_from_val_and_max(y_val, max);
                (
                    Self::x_coord_processor(&raw_ranges.x, &point.key),
                    sum.clone(),
                )
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
    X: PartialOrd + Clone + std::fmt::Display,
    Y: PartialOrd + Clone,
{
    fn ranges(&self, is_active: impl Fn(uid::Line) -> bool) -> Ranges<Option<X>, Option<Y>> {
        let mut ranges: Ranges<Option<&X>, Option<&Y>> =
            Ranges::new(Range::new(None, None), Range::new(None, None));

        for point in self {
            // Update x-range.
            {
                if ranges.x.lbound.is_none() {
                    ranges.x.lbound = Some(&point.key)
                }

                // Points must be ordered by their key, let's make sure that's the case.
                debug_assert!(ranges.x.lbound.is_some());
                debug_assert!(ranges.x.lbound.map(|min| min <= &point.key).unwrap());
                // Now for max.
                debug_assert!(ranges.x.ubound.map(|max| max <= &point.key).unwrap_or(true));

                // Update max.
                ranges.x.ubound = Some(&point.key);
            }

            // Update y-range.
            for (_, val) in point.vals.map.iter().filter(|(uid, _)| is_active(**uid)) {
                // Update min.
                if let Some(min) = &mut ranges.y.lbound {
                    if val < min {
                        *min = val
                    }
                } else {
                    ranges.y.lbound = Some(val)
                }

                // Update max.
                if let Some(max) = &mut ranges.y.ubound {
                    if *max < val {
                        *max = val
                    }
                } else {
                    ranges.y.ubound = Some(val)
                }
            }
        }

        ranges.map(|x| x.cloned(), |y| y.cloned())
    }
}

impl<Y> PointValExt<time::Date> for PolyPoints<time::Date, Y> {
    fn val_range_processor(range: Range<Option<time::Date>>) -> Res<Range<time::Date>> {
        match (range.lbound, range.ubound) {
            (Some(min), Some(max)) => Ok(Range::new(min, max)),
            (min, max) => bail!("failed to compute x-range: {:?}, {:?}", min, max),
        }
    }
    fn val_coord_range_processor(
        range: &Range<time::Date>,
    ) -> Res<Range<<time::Date as CoordExt>::Coord>> {
        let min = time::chrono::Duration::seconds(0);
        let max = range.ubound.date().clone() - range.lbound.date().clone();
        Ok(Range::new(min, max))
    }
    fn val_coord_processor(
        range: &Range<time::Date>,
        x: &time::Date,
    ) -> <time::Date as CoordExt>::Coord {
        x.date().clone() - range.lbound.date().clone()
    }
    fn val_label_formatter(date: &<time::Date as CoordExt>::Coord) -> String {
        let mut res = time::SinceStart::from(date.to_std().unwrap())
            .display_millis()
            .to_string();
        'rm_trailing_zeros: while let Some(last) = res.pop() {
            if last == '0' {
                continue 'rm_trailing_zeros;
            } else {
                res.push(last);
                break 'rm_trailing_zeros;
            }
        }
        res.push('s');
        res
    }
}

impl<Y> PointValExt<time::SinceStart> for PolyPoints<time::SinceStart, Y> {
    fn val_range_processor(range: Range<Option<time::SinceStart>>) -> Res<Range<time::SinceStart>> {
        match (range.lbound, range.ubound) {
            (Some(min), Some(max)) => Ok(Range::new(min, max)),
            (min, max) => bail!("failed to compute x-range: {:?}, {:?}", min, max),
        }
    }
    fn val_coord_range_processor(
        range: &Range<time::SinceStart>,
    ) -> Res<Range<<time::SinceStart as CoordExt>::Coord>> {
        let min = range.lbound.to_chrono_duration();
        let max = range.ubound.to_chrono_duration();
        Ok(Range::new(min, max))
    }
    fn val_coord_processor(
        _range: &Range<time::SinceStart>,
        x: &time::SinceStart,
    ) -> <time::SinceStart as CoordExt>::Coord {
        x.to_chrono_duration()
    }
    fn val_label_formatter(date: &<time::SinceStart as CoordExt>::Coord) -> String {
        let mut res = time::SinceStart::from(date.to_std().unwrap())
            .display_millis()
            .to_string();
        'rm_trailing_zeros: while let Some(last) = res.pop() {
            if last == '0' {
                continue 'rm_trailing_zeros;
            } else {
                res.push(last);
                break 'rm_trailing_zeros;
            }
        }
        res.push('s');
        res
    }
}

impl<X> PointValExt<Size> for PolyPoints<X, Size> {
    fn val_range_processor(range: Range<Option<Size>>) -> Res<Range<Size>> {
        Ok(range.unwrap_or_else(|| u64::default_min().into(), || u64::default_max().into()))
    }
    fn val_coord_range_processor(range: &Range<Size>) -> Res<Range<<Size as CoordExt>::Coord>> {
        let default_max = Size::default_max();
        Ok(Range::new(
            range.lbound.size,
            if range.ubound.size < default_max {
                default_max
            } else {
                range.ubound.size
            },
        ))
    }
    fn val_coord_processor(_range: &Range<Size>, x: &Size) -> <Size as CoordExt>::Coord {
        x.size
    }
    fn val_label_formatter(val: &<Size as CoordExt>::Coord) -> String {
        let mut s = num_fmt::bin_str_do(*val as f64, base::identity);
        s.push('B');
        s
    }
}

impl<X> PointValExt<u64> for PolyPoints<X, u64> {
    fn val_range_processor(range: Range<Option<u64>>) -> Res<Range<u64>> {
        Ok(range.unwrap_or_else(u64::default_min, u64::default_max))
    }
    fn val_coord_range_processor(range: &Range<u64>) -> Res<Range<<u64 as CoordExt>::Coord>> {
        let default_max = u64::default_max();
        Ok(Range::new(
            range.lbound,
            if range.ubound < default_max {
                default_max
            } else {
                range.ubound
            },
        ))
    }
    fn val_coord_processor(_range: &Range<u64>, x: &u64) -> <u64 as CoordExt>::Coord {
        *x
    }
    fn val_label_formatter(val: &<u64 as CoordExt>::Coord) -> String {
        num_fmt::str_do(*val as f64, base::identity)
    }
}

/// Points representing size over time.
pub type TimeSizePoints = PolyPoints<time::SinceStart, Size>;

/// Some points for a time chart.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TimePoints {
    /// Size over time.
    Size(TimeSizePoints),
}

base::implement! {
    impl From for TimePoints {
        from TimeSizePoints => |points| Self::Size(points)
    }
}

impl TimePoints {
    /// True if there are no points.
    pub fn is_empty(&self) -> bool {
        match self {
            Self::Size(points) => points.is_empty(),
        }
    }

    /// Number of x-axis point ticks.
    ///
    /// Each x-axis tick can have several y-axis filter points.
    pub fn len(&self) -> usize {
        match self {
            Self::Size(points) => points.len(),
        }
    }
    /// Total number of points.
    pub fn point_count(&self) -> usize {
        match self {
            Self::Size(points) => points
                .iter()
                .fold(0, |acc, point| acc + point.vals.map.len()),
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

    /// Renders the points on a graph.
    pub fn render<'spec, DB>(
        &self,
        settings: &settings::Chart,
        chart_builder: plotters::prelude::ChartBuilder<DB>,
        style_conf: &impl StyleExt,
        is_active: impl Fn(uid::Line) -> bool,
        active_filters: impl Iterator<Item = &'spec filter::FilterSpec> + Clone,
    ) -> Res<()>
    where
        DB: plotters::prelude::DrawingBackend,
    {
        match self {
            Self::Size(points) => points.render(
                settings,
                chart_builder,
                style_conf,
                is_active,
                active_filters,
            ),
        }
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

    /// Number of x-axis ticks.
    ///
    /// Each x-axis tick can have several y-axis filter points.
    pub fn len(&self) -> usize {
        match self {
            Self::Time(points) => points.len(),
        }
    }
    /// Total number of points.
    pub fn point_count(&self) -> usize {
        match self {
            Self::Time(points) => points.point_count(),
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

    /// Renders the points on a graph.
    pub fn render<'spec, DB>(
        &self,
        settings: &settings::Chart,
        chart_builder: plotters::prelude::ChartBuilder<DB>,
        style_conf: &impl StyleExt,
        is_active: impl Fn(uid::Line) -> bool,
        active_filters: impl Iterator<Item = &'spec filter::FilterSpec> + Clone,
    ) -> Res<()>
    where
        DB: plotters::prelude::DrawingBackend,
    {
        match self {
            Self::Time(points) => points.render(
                settings,
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
    points: BTMap<uid::Chart, Points>,
}
impl ChartPoints {
    /// Constructor.
    pub fn new() -> Self {
        Self {
            points: BTMap::new(),
        }
    }

    /// True if there are no points.
    pub fn is_empty(&self) -> bool {
        self.points.iter().all(|(_uid, points)| points.is_empty())
    }
}

base::implement! {
    impl ChartPoints {
        Deref {
            to BTMap<uid::Chart, Points> => |&self| &self.points
        }
        DerefMut {
            |&mut self| &mut self.points
        }
    }
}
