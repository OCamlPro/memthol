use stdweb::{js, Value};

use crate::base::*;

struct Update {
    to_add: usize,
    to_sub: usize,
}
impl Default for Update {
    fn default() -> Self {
        Update::new()
    }
}
impl Update {
    fn new() -> Self {
        Update {
            to_add: 0,
            to_sub: 0,
        }
    }
    fn add(&mut self, n: usize) {
        self.to_add += n
    }
    fn sub(&mut self, n: usize) {
        self.to_sub += n
    }
    fn update(&self, size: usize) -> usize {
        size + self.to_add - self.to_sub
    }
}

pub struct TimeTotalSizeChart {
    start_date: Option<AllocNuDate>,
    current_size: usize,
    builder: Map<AllocNuDate, Update>,
    live: Map<AllocUid, usize>,
    chart: Option<Value>,
    uid: chart::ChartUid,
    id: String,
}
impl TimeTotalSizeChart {
    pub fn new(uid: chart::ChartUid, id: String) -> Self {
        Self {
            start_date: None,
            current_size: 0,
            builder: Map::new(),
            live: Map::new(),
            chart: None,
            uid,
            id,
        }
    }

    pub fn date_init(&mut self, date: AllocNuDate) {
        if self.start_date.is_some() {
            panic!("trying to initializes the date, but it is already set")
        }
        self.start_date = Some(date)
    }

    /// Renders the chart as HTML.
    pub fn render(&self) -> Html {
        html! {
            <center>
                <div id={&self.id}
                    style="width: 100%; height: 600px;"
                />
            </center>
        }
    }

    pub fn init(&mut self) {
        let chart = js! {
            am4core.useTheme(am4themes_animated);
            var chart = am4core.create(@{&self.id}, am4charts.XYChart);

            chart.data = [];

            chart.padding(0, 0, 0, 0);

            var dateAxis = chart.xAxes.push(new am4charts.DateAxis());
            dateAxis.dateFormats.setKey("second", "ss");
            dateAxis.dateFormats.setKey("millisecond", "nnn");
            dateAxis.periodChangeDateFormats.setKey("second", "[bold]h:mm a");
            dateAxis.periodChangeDateFormats.setKey("minute", "[bold]h:mm a");
            dateAxis.periodChangeDateFormats.setKey("hour", "[bold]h:mm a");

            var valueAxis = chart.yAxes.push(new am4charts.ValueAxis());

            var series = chart.series.push(new am4charts.LineSeries());
            series.dataFields.dateX = "x";
            series.dataFields.valueY = "y";
            series.tooltipText = "{y}";
            series.interpolationDuration = 100;
            series.defaultState.transitionDuration = 0;
            series.strokeWidth = 2;
            series.minBulletDistance = 15;

            // Create vertical scrollbar and place it before the value axis
            // chart.scrollbarY = new am4core.Scrollbar();
            // chart.scrollbarY.parent = chart.leftAxesContainer;
            // chart.scrollbarY.toBack();

            // Create a horizontal scrollbar with previe and place it underneath the date axis
            chart.scrollbarX = new am4charts.XYChartScrollbar();
            chart.scrollbarX.series.push(series);
            chart.scrollbarX.parent = chart.bottomAxesContainer;

            // bullet at the front of the line
            var bullet = series.createChild(am4charts.CircleBullet);
            bullet.circle.radius = 5;
            bullet.fillOpacity = 1;
            bullet.fill = chart.colors.getIndex(0);
            bullet.isMeasured = false;

            series.events.on("validated", function() {
                if (series.dataItems.last !== undefined) {
                    bullet.moveTo(series.dataItems.last.point);
                    bullet.validatePosition()
                }
            });

            return chart;
        };
        self.chart = Some(chart)
    }

    pub fn date_of(&self, date: AllocDate) -> AllocNuDate {
        let mut res = if let Some(start_date) = self.start_date.clone() {
            start_date
        } else {
            info!("trying to convert a duration but no start date is set");
            panic!("[bug] illegal workflow")
        };

        // info! { "adding {} to {}", date, res }

        res.add(date);
        // info! { "=> {}", res }
        res
    }

    pub fn add_alloc(&mut self, alloc: &Alloc) {
        let size = alloc.size();
        let toc = self.date_of(alloc.toc());
        self.builder.entry(toc).or_default().add(size);
        if let Some(tod) = alloc.tod() {
            let tod = self.date_of(tod);
            self.builder.entry(tod).or_default().sub(size)
        } else {
            let prev = self.live.insert(alloc.uid().clone(), size);
            assert! { prev.is_none() }
        }
    }

    pub fn add_death(&mut self, uid: &AllocUid, tod: alloc_data::Date) {
        let tod = self.date_of(tod);
        let size = if let Some(size) = self.live.remove(&uid) {
            size
        } else {
            panic!("unknown allocation UID `{}`", uid)
        };

        self.builder.entry(tod).or_default().sub(size)
    }

    pub fn update(&mut self) {
        info!("updating");
        let mut acc = self.current_size;
        for (date, update) in &self.builder {
            acc = update.update(acc);
            info!("date: {}", date);
            let (h, m, s, mi) = date.time_info();
            info!("      {}h{}m{}s{}mi", h, m, s, mi);
            js! {
                let point = { "x": @{date.as_js()}, "y": @{acc.to_string()} };
                console.log("x: " + point.x + ", y: " + point.y);
                @{self.chart.as_ref().unwrap()}.addData(point)
            }
        }
        self.current_size = acc;
        self.builder.clear()
    }
}
