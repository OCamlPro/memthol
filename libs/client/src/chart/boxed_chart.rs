//! A chart where the axes are abstract.

use stdweb::js;

use crate::base::*;

use chart::axis;

pub fn default<Str: AsRef<str>>(id: Str) -> BChart {
    new::x_time::y_size_sum(id.as_ref())
}

pub trait AbstractChart {
    /// Registers some allocation data.
    ///
    /// Does **not** update the actual chart.
    fn add_alloc(&mut self, alloc: &Alloc);
    /// Registers that some data has died.
    ///
    /// Does **not** update the actual chart.
    fn add_death(&mut self, uid: &AllocUid, tod: AllocDate);
    /// Updates the actual chart.
    fn update(&mut self);
}

pub type BChart = Box<dyn AbstractChart + 'static>;

pub mod new {
    use super::*;

    pub mod x_time {
        use super::*;

        pub fn y_size_sum(id: &str) -> BChart {
            let chart: ChartData<axis::XTime, axis::YSizeSum> = ChartData::init(id);
            // chart.set_x_range(axis::Range::sliding(Duration::new(1, 0).into()));
            Box::new(chart)
        }
    }

    pub mod x_size {
        use super::*;

        pub fn y_size_sum(id: &str) -> BChart {
            let chart: ChartData<axis::XSize, axis::YSizeSum> = ChartData::init(id);
            Box::new(chart)
        }
    }
}

pub struct ChartData<X, Y>
where
    X: axis::XAxis,
    Y: axis::YAxis,
{
    data: Map<X::Value, Y::Acc>,
    live: Map<AllocUid, (X::LiveInfo, Y::LiveInfo)>,
    chart: Value,
    x_axis: X,
    y_axis: Y,
    filter: data::Filter,
    y_acc: Y::Acc,
}
impl<X, Y> ChartData<X, Y>
where
    X: axis::XAxis,
    X::Value: axis::CloneSubOrd,
    Y: axis::YAxis,
    Y::Value: axis::CloneSubOrd,
{
    /// Constructor.
    fn new(x_axis: X, y_axis: Y, chart: Value) -> Self {
        let mut data = Map::new();
        match (X::origin_value(), Y::origin_value()) {
            (Some(x), Some(y)) => {
                let prev = data.insert(x, y);
                debug_assert! { prev.is_none() }
            }
            _ => (),
        }
        Self {
            data,
            live: Map::new(),
            chart,
            x_axis,
            y_axis,
            filter: data::Filter::new(),
            y_acc: Y::init_acc(),
        }
    }

    // /// X-axis accessor.
    // pub fn x_axis(&self) -> &X {
    //     &self.x_axis
    // }
    // /// Y-axis accessor.
    // pub fn y_axis(&self) -> &Y {
    //     &self.y_axis
    // }

    // /// Sets the range for the x-axis.
    // pub fn set_x_range(&mut self, range: axis::Range<X::Value>) {
    //     self.x_axis.set_range(range)
    // }

    fn get_mut(&mut self, x: X::Value) -> &mut Y::Acc {
        self.data.entry(x).or_insert_with(Y::init_acc)
    }

    // /// The min and max x-values.
    // fn x_min_max(&self) -> Option<(&X::Value, &X::Value)> {
    //     self.data
    //         .iter()
    //         .next()
    //         .and_then(|(min, _)| self.data.iter().next_back().map(|(max, _)| (min, max)))
    // }

    /// Extracts the new points.
    pub fn points(&mut self) -> Value {
        let vec = js! { return [] };

        let mut acc = self.y_acc.clone();
        for (x, y_acc) in self.data.iter() {
            acc = Y::combine_acc(&acc, y_acc);
            let y = Y::value_of_acc(&acc);
            info! { "x: {}, y: {} ({})", x, y, *x }
            js! {
                @{&vec}.push(
                    {
                        x: @{X::js_of_value(x)},
                        y: @{Y::js_of_value(&y)},
                    }
                )
            }
        }
        self.y_acc = acc;
        if X::clear_map {
            self.data.clear()
        }
        vec
    }

    // /// Adds the new points since the last call to the chart.
    // pub fn update_new_points(&mut self) {
    //     js! {
    //         var chart = @{&self.chart};
    //         let points = @{self.points()};
    //         while (points.length > 0) {
    //             let point = points.pop();

    //         }
    //     }
    // }

    /// Adds the new points since the last call to the chart.
    pub fn append_new_points(&mut self) {
        js! {
            var chart = @{&self.chart};
            let points = @{self.points()};
            points.forEach(
                function(point) {
                    if (chart.data[point.x] === undefined) {
                        console.info("adding   x: " + point.x + ", y: " + point.y);
                        chart.addData(point)
                    } else {
                        console.info("updating x: " + point.x + ", y: " + point.y + "(" + chart.data[point.x] + ")");
                        chart.data[point.x] = point.y;
                        chart.invalidateRawData();
                    }
                }
            );
            // chart.addData(points);
            // @{&self.chart}.update();
        }
    }

    /// Initializes a chart.
    pub fn init(id: &str) -> Self {
        let x_axis = X::default();
        let y_axis = Y::default();
        let chart = js! {
            am4core.useTheme(am4themes_animated);
            var chart = am4core.create(@{id}, am4charts.XYChart);

            chart.data = [];

            var x_axis = chart.xAxes.push(new am4charts.ValueAxis());
            var y_axis = chart.yAxes.push(new am4charts.ValueAxis());

            x_axis.renderer.inside = true;
            x_axis.renderer.axisFills.template.disabled = true;
            x_axis.renderer.ticks.template.disabled = true;

            y_axis.renderer.axisFills.template.disabled = true;
            y_axis.renderer.ticks.template.disabled = true;
            // y_axis.interpolationDuration = 500;
            // y_axis.rangeChangeDuration = 500;
            y_axis.renderer.inside = true;

            // x_axis.interpolationDuration = 200;
            // x_axis.rangeChangeDuration = 200;
            // y_axis.interpolationDuration = 200;
            // y_axis.rangeChangeDuration = 200;

            var series = chart.series.push(new am4charts.LineSeries());
            series.dataFields.valueX = "x";
            series.dataFields.valueY = "y";
            series.tooltipText = "{y}";
            series.strokeWidth = 2;
            series.minBulletDistance = 15;

            series.tooltip.background.cornerRadius = 20;
            series.tooltip.background.strokeOpacity = 0;
            series.tooltip.pointerOrientation = "vertical";
            series.tooltip.label.minWidth = 40;
            series.tooltip.label.minHeight = 40;
            series.tooltip.label.textAlign = "middle";
            series.tooltip.label.textValign = "middle";

            series.interpolationDuration = 500;
            series.defaultState.transitionDuration = 0;
            // series.tensionX = 0.8;

            // Make bullets grow on hover.
            var bullet = series.bullets.push(new am4charts.CircleBullet());
            bullet.circle.strokeWidth = 2;
            bullet.circle.radius = 4;
            bullet.circle.fill = am4core.color("#fff");

            var bullethover = bullet.states.create("hover");
            bullethover.properties.scale = 1.3;

            // Make a panning cursor.
            chart.cursor = new am4charts.XYCursor();
            chart.cursor.behavior = "panXY";
            chart.cursor.xAxis = x_axis;
            chart.cursor.snapToSeries = series;

            // Create vertical scrollbar and place it before the y-axis.
            chart.scrollbarY = new am4core.Scrollbar();
            chart.scrollbarY.parent = chart.leftAxesContainer;
            chart.scrollbarY.toBack();

            // Create a horizontal scrollbar with previe and place it underneath the x-axis.
            chart.scrollbarX = new am4charts.XYChartScrollbar();
            chart.scrollbarX.series.push(series);
            chart.scrollbarX.parent = chart.bottomAxesContainer;

            // chart.events.on("datavalidated", function () {
            //     x_axis.zoom({ start: 1 / 15, end: 1.2 }, false, true);
            // });

            return chart;
        };
        Self::new(x_axis, y_axis, chart)
    }
}

/// Filter-related functions.
impl<X, Y> ChartData<X, Y>
where
    X: axis::XAxis,
    Y: axis::YAxis,
{
    /// Adds a new filter.
    pub fn add_filter<F>(&mut self, filter: F)
    where
        F: data::filter::AllocFilter + 'static,
    {
        self.filter.add(filter)
    }
}

impl<X, Y> AbstractChart for ChartData<X, Y>
where
    X: axis::XAxis,
    X::Value: axis::CloneSubOrd,
    Y: axis::YAxis,
    Y::Value: axis::CloneSubOrd,
{
    fn add_alloc(&mut self, alloc: &Alloc) {
        if self.filter.apply(alloc) {
            let uid = alloc.uid().clone();
            let x_value: X::Value = X::value_of_alloc(alloc);
            let y_value: Y::Value = Y::value_of_alloc(alloc);

            let y_acc = self.get_mut(x_value);
            *y_acc = Y::combine_value(y_acc.clone(), y_value);

            let x_info: X::LiveInfo = X::info_of_alloc(alloc);
            let y_info: Y::LiveInfo = Y::info_of_alloc(alloc);
            let prev = self.live.insert(uid, (x_info, y_info));
            debug_assert! { prev.is_none() }

            match alloc.tod() {
                None => (),
                Some(tod) => self.add_death(alloc.uid(), tod),
            }
        }
    }

    fn add_death(&mut self, uid: &AllocUid, tod: AllocDate) {
        if let Some((x_info, y_info)) = self.live.remove(uid) {
            let x_val = X::value_of_info(x_info, &tod);
            let y_acc = self.get_mut(x_val);
            *y_acc = Y::combine_info(y_acc.clone(), y_info, &tod);
        }
    }

    fn update(&mut self) {
        self.append_new_points()
    }
}

impl<X, Y> ChartData<X, Y>
where
    X: axis::XAxis,
    Y: axis::YAxis,
    X::Value: fmt::Display,
    Y::Value: fmt::Display,
    Y::Acc: fmt::Display,
{
    pub fn log(&self) {
        info! { "chart data:" }
        self.data.iter().fold(Y::init_acc(), |acc, (x, y_acc)| {
            let acc = Y::combine_acc(&acc, y_acc);
            let y = Y::value_of_acc(&acc);
            info! { "    x: {}, y_acc: {}, y: {}", x, y_acc, y }
            acc
        });
    }
}
