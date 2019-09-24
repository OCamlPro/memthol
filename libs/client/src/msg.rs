//! Messages of the client.

use crate::base::*;

pub use charts::msg::{to_client as from_server, to_server};

use chart::ChartUid;
use filter::FilterUid;

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
    /// Filter operations.
    Filter(FiltersMsg),

    /// A message to print in the JS console.
    Msg(String),
    /// An error.
    Err(err::Err),

    /// A message that does nothing.
    Noop,
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
impl From<FiltersMsg> for Msg {
    fn from(msg: FiltersMsg) -> Self {
        Self::Filter(msg)
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

    /// Forces to refresh the filters.
    RefreshFilters,

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

    /// Forces to refresh all the filters.
    pub fn refresh_filters() -> Msg {
        Self::RefreshFilters.into()
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
pub enum FiltersMsg {
    /// A message for a specific filter specification.
    FilterSpec {
        /// Uid of the filter.
        uid: Option<FilterUid>,
        /// Message.
        msg: FilterSpecMsg,
    },
    /// A message for a specific filter.
    Filter {
        /// UID of the iflter.
        uid: FilterUid,
        /// Message.
        msg: FilterMsg,
    },
}
impl FiltersMsg {
    /// A message for a specific filter specification.
    pub fn filter_spec(uid: Option<FilterUid>, msg: FilterSpecMsg) -> Msg {
        Self::FilterSpec { uid, msg }.into()
    }
    /// A message for a specific filter.
    pub fn filter(uid: FilterUid, msg: FilterMsg) -> Msg {
        Self::Filter { uid, msg }.into()
    }
}

#[derive(Debug)]
pub enum FilterSpecMsg {
    /// Changes the name of a filter.
    ChangeName(ChangeData),
    /// Changes the color of a filter.
    ChangeColor(ChangeData),
}
impl FilterSpecMsg {
    /// Changes the name of a filter.
    pub fn change_name(uid: Option<FilterUid>, new_name: ChangeData) -> Msg {
        FiltersMsg::filter_spec(uid, Self::ChangeName(new_name)).into()
    }
    /// Changes the color of a filter.
    pub fn change_color(uid: Option<FilterUid>, new_color: ChangeData) -> Msg {
        FiltersMsg::filter_spec(uid, Self::ChangeColor(new_color)).into()
    }
}

#[derive(Debug)]
pub enum FilterMsg {
    /// Adds a new subfilter.
    AddNew,
    /// Updates a subfilter.
    Sub(filter::SubFilter),
}
impl FilterMsg {
    /// Adds a new subfilter.
    pub fn add_new(uid: FilterUid) -> Msg {
        FiltersMsg::filter(uid, Self::AddNew)
    }
    /// Updates a subfilter.
    pub fn update_sub(uid: FilterUid, sub: filter::SubFilter) -> Msg {
        FiltersMsg::filter(uid, Self::Sub(sub))
    }
}
