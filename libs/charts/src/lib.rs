//! Server-side chart handling.
//!
//! Note that most types in this crate implement (de)serialization to/from json. These types
//! implement the [`Json`] trait which provides straightforward conversion functions. This is used
//! heavily for server/client exchanges.
//!
//! # Basic Workflow
//!
//! All allocation-related data is stored in a global state in the [`data`] module. It features a
//! [`Watcher`] type which, after [`start`]ing it, will monitor a directory for init and diff files.
//!
//! [`Json`]: ./base/trait.Json.html (The Json trait)
//! [`Watcher`]: ./data/struct.Watcher.html (The Watcher struct in module data)
//! [`start`]: ./data/fn.start.html (The start function in module data)

pub extern crate alloc_data;
pub extern crate palette;
pub extern crate plotters;

#[macro_use]
pub mod prelude;

pub mod chart;
pub mod color;
#[cfg(any(test, feature = "server"))]
pub mod data;
pub mod filter;
pub mod msg;
pub mod point;

#[cfg(any(test, feature = "server"))]
pub use chart::Chart;

#[cfg(any(test, feature = "server"))]
prelude! {}

/// Trait implemented by all charts.
#[cfg(any(test, feature = "server"))]
pub trait ChartExt {
    /// Generates the new points of the chart.
    fn new_points(&mut self, filters: &mut Filters, init: bool) -> Res<Points>;

    /// Resets the chart.
    fn reset(&mut self, filters: &Filters);
}

/// Aggregates some charts.
#[cfg(any(test, feature = "server"))]
pub struct Charts {
    /// List of active charts.
    charts: Vec<Chart>,
    /// List of filters.
    filters: Filters,
    /// Start time of the run.
    ///
    /// This is used to check whether we need to detect that the init file of the run has changed
    /// and that we need to reset the charts.
    start_time: Option<time::Date>,
    to_client_msgs: msg::to_client::Msgs,
}

#[cfg(any(test, feature = "server"))]
impl Charts {
    /// Constructor.
    pub fn new() -> Self {
        Self {
            charts: vec![],
            filters: Filters::new(),
            start_time: None,
            to_client_msgs: msg::to_client::Msgs::with_capacity(7),
        }
    }

    /// All the charts.
    pub fn charts(&self) -> &Vec<Chart> {
        &self.charts
    }
    /// All the filters.
    pub fn filters(&self) -> &Filters {
        &self.filters
    }
    /// Start time.
    pub fn start_time(&self) -> Option<&time::Date> {
        self.start_time.as_ref()
    }

    /// Runs filter generation.
    ///
    /// Returns the number of filter generated.
    #[cfg(any(test, feature = "server"))]
    pub fn auto_gen(&mut self, generator: impl Into<filter::gen::FilterGen>) -> Res<usize> {
        self.filters.auto_gen(&*data::get()?, generator)
    }

    /// Pushes a new chart.
    pub fn push(&mut self, chart: Chart) {
        self.charts.push(chart)
    }

    /// Chart mutable accessor.
    pub fn get_mut(&mut self, uid: uid::Chart) -> Res<&mut Chart> {
        for chart in self.charts.iter_mut() {
            if chart.uid() == uid {
                return Ok(chart);
            }
        }
        bail!("cannot access chart with unknown UID #{}", uid)
    }
}

#[cfg(any(test, feature = "server"))]
impl Charts {
    /// Restarts the charts and the filters if needed.
    fn restart_if_needed(&mut self) -> Res<bool> {
        let data = data::get();
        let start_time = data
            .and_then(|data| data.start_time())
            .chain_err(|| "while checking if the charts should be restarted")?;
        if self.start_time != Some(start_time) {
            self.start_time = Some(start_time);
            for chart in &mut self.charts {
                chart.reset(&self.filters)
            }
            self.filters.reset();
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Extracts the new points for the different charts.
    ///
    /// The boolean indicates whether the points should overwrite existing points. It is typically
    /// true when the init file of the run has changed (the run was restarted).
    pub fn new_points(&mut self, init: bool) -> Res<(point::ChartPoints, bool)> {
        let restarted = self.restart_if_needed()?;
        let mut points = point::ChartPoints::new();
        for chart in &mut self.charts {
            if let Some(chart_points) = chart.new_points(&mut self.filters, restarted || init)? {
                let prev = points.insert(chart.uid(), chart_points);
                debug_assert!(prev.is_none())
            }
        }
        Ok((points, restarted || init))
    }

    pub fn handle_chart_msg(&mut self, msg: msg::to_server::ChartsMsg) -> Res<()> {
        debug_assert!(self.to_client_msgs.is_empty());

        match msg {
            msg::to_server::ChartsMsg::New(x_axis, y_axis) => {
                let nu_chart = chart::Chart::new(&mut self.filters, x_axis, y_axis)
                    .chain_err(|| "while creating new chart")?;

                // Chart creation message.
                self.to_client_msgs
                    .push(msg::to_client::ChartsMsg::new_chart(
                        nu_chart.spec().clone(),
                        nu_chart.settings().clone(),
                    ));
                // // Initial points message.
                // let points = nu_chart.new_points(&mut self.filters, true).chain_err(|| {
                //     format!(
                //         "while generating the initial points for new chart #{}",
                //         nu_chart.uid()
                //     )
                // })?;
                // to_client_msgs.push(msg::to_client::ChartMsg::new_points(nu_chart.uid(), points));

                self.charts.push(nu_chart)
            }

            msg::to_server::ChartsMsg::Reload => {
                let msg = self.reload_points(None, false)?;
                self.to_client_msgs.push(msg)
            }

            msg::to_server::ChartsMsg::ChartUpdate { uid, msg } => {
                let reload = self.get_mut(uid)?.update(msg);
                if reload {
                    let msg = self.reload_points(Some(uid), false)?;
                    self.to_client_msgs.push(msg)
                }
            }
        }

        Ok(())
    }

    /// Recomputes all the points, and returns them as a message for the client.
    pub fn reload_points(
        &mut self,
        uid: Option<uid::Chart>,
        refresh_filters: bool,
    ) -> Res<msg::to_client::Msg> {
        let mut new_points = point::ChartPoints::new();
        for chart in &mut self.charts {
            if let Some(uid) = uid {
                if chart.uid() != uid {
                    continue;
                }
            }
            chart.reset(&self.filters);
            self.filters.reset();
            let points_opt = chart
                .new_points(&mut self.filters, true)
                .chain_err(|| format!("while generating points for chart #{}", chart.uid()))?;
            if let Some(points) = points_opt {
                let prev = new_points.insert(chart.uid(), points);
                if prev.is_some() {
                    bail!("chart UID collision on #{}", chart.uid())
                }
            }
        }
        Ok(msg::to_client::ChartsMsg::new_points(
            new_points,
            refresh_filters,
        ))
    }

    /// Handles a message from the client.
    pub fn handle_msg<'me>(
        &'me mut self,
        msg: msg::to_server::Msg,
    ) -> Res<impl Iterator<Item = msg::to_client::Msg> + 'me> {
        use msg::to_server::Msg::*;

        match msg {
            Charts(msg) => self.handle_chart_msg(msg)?,
            Filters(msg) => {
                let (mut msgs, should_reload) = self.filters.update(msg)?;
                if should_reload {
                    msgs.push(self.reload_points(None, true)?)
                }
                self.to_client_msgs.extend(msgs)
            }
        };

        Ok(self.to_client_msgs.drain(0..))
    }
}
