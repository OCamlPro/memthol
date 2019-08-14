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
    fn update(&self);
}

pub type BChart = Box<dyn AbstractChart + 'static>;

pub mod new {
    use super::*;

    pub mod x_time {
        use super::*;

        pub fn y_size_sum(id: &str) -> BChart {
            let mut chart: ChartData<axis::XTime, axis::YSizeSum> = ChartData::init(id);
            chart.set_x_range(axis::Range::sliding(Duration::new(1, 0).into()));
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
    config: Value,
    chart: Value,
    x_axis: X,
    y_axis: Y,
    filter: data::Filter,
}
impl<X, Y> ChartData<X, Y>
where
    X: axis::XAxis,
    X::Value: axis::CloneSubOrd,
    Y: axis::YAxis,
    Y::Value: axis::CloneSubOrd,
{
    /// Constructor.
    fn new(x_axis: X, y_axis: Y, config: Value, chart: Value) -> Self {
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
            config,
            chart,
            x_axis,
            y_axis,
            filter: data::Filter::new(),
        }
    }

    /// X-axis accessor.
    pub fn x_axis(&self) -> &X {
        &self.x_axis
    }
    /// Y-axis accessor.
    pub fn y_axis(&self) -> &Y {
        &self.y_axis
    }

    /// Sets the range for the x-axis.
    pub fn set_x_range(&mut self, range: axis::Range<X::Value>) {
        self.x_axis.set_range(range)
    }

    fn get_mut(&mut self, x: X::Value) -> &mut Y::Acc {
        self.data.entry(x).or_insert_with(Y::init_acc)
    }

    /// The min and max x-values.
    fn x_min_max(&self) -> Option<(&X::Value, &X::Value)> {
        self.data
            .iter()
            .next()
            .and_then(|(min, _)| self.data.iter().next_back().map(|(max, _)| (min, max)))
    }

    /// Extracts the points.
    pub fn points(&self) -> Value {
        let vec = js! { return [] };

        let (x_min, x_max) = if let Some((min, max)) = self.x_min_max() {
            (min, max)
        } else {
            // Returning, there's no points to draw anyways.
            return vec;
        };

        let x_range = self.x_axis.axis().range();

        let mut last_out_of_range: Option<(X::Value, Y::Value)> = None;

        let mut acc = Y::init_acc();
        for (x, y_acc) in self.data.iter() {
            acc = Y::combine_acc(&acc, y_acc);
            let y = Y::value_of_acc(&acc);
            if x_range.contains(&x, &x_min, &x_max) {
                if let Some((x, y)) = std::mem::replace(&mut last_out_of_range, None) {
                    js! {
                        @{&vec}.push(
                            {
                                x: @{X::js_of_value(&x)},
                                y: @{Y::js_of_value(&y)},
                            }
                        )
                    }
                }
                info! { "x: {}, y: {} ({})", x, y, *x }
                js! {
                    @{&vec}.push(
                        {
                            x: @{X::js_of_value(x)},
                            y: @{Y::js_of_value(&y)},
                        }
                    )
                }
            } else {
                last_out_of_range = Some((x.clone(), y));
            }
        }
        x_range.apply_to_axis(
            x_min,
            x_max,
            &js!(return @{&self.config}.options.scales.xAxes[0]),
        );
        vec
    }

    /// Sets the data in a chart.
    pub fn update(&self) {
        js! {
            var config = @{&self.config};
            let points = @{self.points()};
            config.data.datasets[0].data = points;
            @{&self.chart}.update();
        }
    }

    /// Initializes a chart.
    pub fn init(id: &str) -> Self {
        let x_axis = X::default();
        let y_axis = Y::default();
        let config = js! {
            var data = [];
            const x = data.map(point => point.x);
            const y = data.map(point => point.y);

            var chartOptions = {
                showLine: true,
                tooltips: {
                    enabled: true,
                    mode: "nearest",
                    intersect: true,
                },
                scales: {
                    xAxes: [ @{x_axis.axis().as_js()} ],
                    yAxes: [ @{y_axis.axis().as_js()} ],
                },
                hover: {
                    mode: "nearest",
                    axis: "xy",
                },
                // animation: {
                //     duration: 100,
                //     easing: "easeOutExpo",
                // },
                elements: {
                    point: {
                        radius: 0
                    }
                }
            };

            return {
                type: "scatter",
                data: {
                    datasets: [
                        {
                            label: "dataset label",
                            showLine: true,
                            fill: true,
                            tension: 0,
                            borderColor: "#3e95cd",
                        }
                    ],
                },
                options: chartOptions,
            };
        };
        let chart = js! {
            var cxt = document.getElementById(@{id});
            return new Chart(cxt, @{&config});
        };
        Self::new(x_axis, y_axis, config, chart)
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

    fn update(&self) {
        self.update()
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
