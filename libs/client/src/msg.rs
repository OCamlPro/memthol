//! Messages of the client.

use crate::base::*;

pub use charts::msg::{to_client as from_server, to_server};

use charts::uid::ChartUid;

/// Internal model messages.
///
/// These messages are sent by the model's components to the model. The only exception is
/// `FromServer`, which contains a message sent by the server.
#[derive(Debug)]
pub enum Msg {
    /// A message from a server.
    FromServer(from_server::RawMsg),
    /// A message to send to the server (from sub-components).
    ToServer(to_server::Msg),
    /// Status notification for the connection with the server.
    ConnectionStatus(WebSocketStatus),

    /// Chart operations.
    Charts(ChartsMsg),
    /// Footer operations.
    Footer(FooterMsg),

    /// A message to print in the JS console.
    Msg(String),

    /// An error.
    Err(err::Err),
}

impl Msg {
    /// Error message constructor.
    pub fn err(e: err::Err) -> Self {
        Self::Err(e)
    }
}

impl From<String> for Msg {
    fn from(s: String) -> Self {
        Self::Msg(s)
    }
}
impl From<err::Err> for Msg {
    fn from(e: err::Err) -> Self {
        Self::err(e)
    }
}
impl From<from_server::RawMsg> for Msg {
    fn from(msg: from_server::RawMsg) -> Self {
        Self::FromServer(msg)
    }
}
impl From<to_server::Msg> for Msg {
    fn from(msg: to_server::Msg) -> Self {
        Self::ToServer(msg)
    }
}
impl From<ChartsMsg> for Msg {
    fn from(msg: ChartsMsg) -> Self {
        Self::Charts(msg)
    }
}
impl From<FooterMsg> for Msg {
    fn from(msg: FooterMsg) -> Self {
        Self::Footer(msg)
    }
}

/// Operations over charts.
#[derive(Debug)]
pub enum ChartsMsg {
    /// Builds a chart and attaches it to its container.
    ///
    /// This is typically sent after a chart is first render, thus creating the chart container. The
    /// message forces to build and bind the chart once the container exists.
    Build(ChartUid),
    /// Moves a chart up or down.
    Move {
        /// UID of the chart.
        uid: ChartUid,
        /// Move up if true, down otherwise.
        up: bool,
    },
    /// Toggles the visibility of a chart.
    ToggleVisible(ChartUid),
    /// Destroys a chart.
    Destroy(ChartUid),

    /// Sets the x-axis in the new chart element.
    NewChartSetX(chart::axis::XAxis),
    /// Sets the y-axis in the new chart element.
    NewChartSetY(chart::axis::YAxis),
}
impl ChartsMsg {
    /// Constructs a `Build` message.
    pub fn build(uid: ChartUid) -> Msg {
        Self::Build(uid).into()
    }
    /// Constructs a message to move a chart up.
    pub fn move_up(uid: ChartUid) -> Msg {
        Self::Move { uid, up: true }.into()
    }
    /// Constructs a message to move a chart down.
    pub fn move_down(uid: ChartUid) -> Msg {
        Self::Move { uid, up: true }.into()
    }
    /// Constructs a message to toggle the visibility of a chart.
    pub fn toggle_visible(uid: ChartUid) -> Msg {
        Self::ToggleVisible(uid).into()
    }
    /// Constructs a message to destroy a chart.
    pub fn destroy(uid: ChartUid) -> Msg {
        Self::Destroy(uid).into()
    }

    /// Sets the x-axis in the new chart element.
    pub fn new_chart_set_x(x: chart::axis::XAxis) -> Msg {
        Self::NewChartSetX(x).into()
    }
    /// Sets the y-axis in the new chart element.
    pub fn new_chart_set_y(y: chart::axis::YAxis) -> Msg {
        Self::NewChartSetY(y).into()
    }
}

/// Footer operation.
#[derive(Debug)]
pub enum FooterMsg {
    /// Toggles a tab.
    ToggleTab(footer::FooterTab),
}
impl FooterMsg {
    /// Toggles a tab.
    pub fn toggle_tab(tab: footer::FooterTab) -> Msg {
        Self::ToggleTab(tab).into()
    }
}

/// Operations over filters.
#[derive(Debug)]
pub enum FiltersMsg {}
