//! Messages of the client.

prelude! {}

pub use charts::msg::{to_client as from_server, to_server};

use chart::ChartUid;
use filter::{FilterUid, LineUid};

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
    /// A warning to print in the JS console.
    Warn(String),
    /// An error.
    Err(err::Err),

    /// A message that does nothing.
    Noop,
}

impl Msg {
    /// Text message constructor.
    pub fn msg<S: Into<String>>(txt: S) -> Msg {
        Self::Msg(txt.into())
    }
    /// Warning message constructor.
    pub fn warn<S: Into<String>>(txt: S) -> Msg {
        Self::Warn(txt.into())
    }
    /// Error message constructor.
    pub fn err(e: err::Err) -> Self {
        Self::Err(e)
    }
}

impl fmt::Display for Msg {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::FromServer(_) => write!(fmt, "from the server"),
            Self::ToServer(_) => write!(fmt, "for the server"),
            Self::ConnectionStatus(_) => write!(fmt, "connection status"),
            Self::Charts(charts_msg) => write!(fmt, "charts, {}", charts_msg),
            Self::Footer(footer_msg) => write!(fmt, "footer, {}", footer_msg),
            Self::Filter(filter_msg) => write!(fmt, "filter, {}", filter_msg),
            Self::Msg(_) => write!(fmt, "info"),
            Self::Warn(_) => write!(fmt, "warning"),
            Self::Err(_) => write!(fmt, "error"),
            Self::Noop => write!(fmt, "noop"),
        }
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
    /// Moves a chart up or down.
    Move {
        /// UID of the chart.
        uid: ChartUid,
        /// Move up if true, down otherwise.
        up: bool,
    },
    /// Toggles the visibility of a chart.
    ToggleVisible(ChartUid),
    /// Toggles the visibility of a filter for a chart.
    FilterToggleVisible(ChartUid, LineUid),

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
    /// Constructs a message to move a chart up.
    pub fn move_up(uid: ChartUid) -> Msg {
        Self::Move { uid, up: true }.into()
    }
    /// Constructs a message to move a chart down.
    pub fn move_down(uid: ChartUid) -> Msg {
        Self::Move { uid, up: false }.into()
    }
    /// Constructs a message to toggle the visibility of a chart.
    pub fn toggle_visible(uid: ChartUid) -> Msg {
        Self::ToggleVisible(uid).into()
    }
    /// Constructs a message to toggle the visibility of filter for a chart.
    pub fn filter_toggle_visible(chart_uid: ChartUid, filter_uid: LineUid) -> Msg {
        Self::FilterToggleVisible(chart_uid, filter_uid).into()
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

impl fmt::Display for ChartsMsg {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Move { uid, up } => write!(fmt, "move {}/{}", uid, up),
            Self::ToggleVisible(c_uid) => write!(fmt, "toggle visible {}", c_uid),
            Self::FilterToggleVisible(c_uid, l_uid) => {
                write!(fmt, "filter toggle visible {} for chart {}", l_uid, c_uid)
            }
            Self::Destroy(c_uid) => write!(fmt, "destroy {}", c_uid),
            Self::RefreshFilters => write!(fmt, "refresh filters"),
            Self::NewChartSetX(_) => write!(fmt, "new-chart-set-x"),
            Self::NewChartSetY(_) => write!(fmt, "new-chart-set-y"),
        }
    }
}

/// Footer operation.
#[derive(Debug)]
pub enum FooterMsg {
    /// Toggles a tab.
    ToggleTab(footer::FooterTab),
    // /// Lets the footer know a filter was removed.
    // Removed(FilterUid),
}
impl FooterMsg {
    /// Toggles a tab.
    pub fn toggle_tab(tab: impl Into<footer::FooterTab>) -> Msg {
        Self::ToggleTab(tab.into()).into()
    }
    // /// Lets the footer know a filter was removed.
    // pub fn removed(uid: FilterUid) -> Msg {
    //     Self::Removed(uid).into()
    // }
}

impl fmt::Display for FooterMsg {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::ToggleTab(_) => write!(fmt, "toggle tab"),
            // Self::Removed(f_uid) => write!(fmt, "remove {}", f_uid),
        }
    }
}

/// Operations over filters.
#[derive(Debug)]
pub enum FiltersMsg {
    /// Updates a filter on the server.
    Save,
    /// Removes a filter.
    Rm(FilterUid),
    /// A message for a specific filter specification.
    FilterSpec {
        /// Uid of the filter.
        uid: LineUid,
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
    /// Moves a filter left or right.
    Move { uid: FilterUid, left: bool },
}

impl fmt::Display for FiltersMsg {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Save => write!(fmt, "save"),
            Self::Rm(f_uid) => write!(fmt, "rm {}", f_uid),
            Self::FilterSpec { uid, msg } => write!(fmt, "filter spec {}, {}", uid, msg),
            Self::Filter { uid, msg } => write!(fmt, "filter {}, {}", uid, msg),
            Self::Move { uid, left } => write!(fmt, "move {} ({})", uid, left),
        }
    }
}
impl FiltersMsg {
    /// Updates a filter on the server.
    pub fn save() -> Msg {
        Self::Save.into()
    }
    /// Removes a filter.
    pub fn rm(uid: FilterUid) -> Msg {
        Self::Rm(uid).into()
    }
    /// A message for a specific filter specification.
    pub fn filter_spec(uid: LineUid, msg: FilterSpecMsg) -> Msg {
        Self::FilterSpec { uid, msg }.into()
    }
    /// A message for a specific filter.
    pub fn filter(uid: FilterUid, msg: FilterMsg) -> Msg {
        Self::Filter { uid, msg }.into()
    }
    /// Moves a filter left or right.
    pub fn move_filter(uid: FilterUid, left: bool) -> Msg {
        Self::Move { uid, left }.into()
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
    pub fn change_name(uid: LineUid, new_name: ChangeData) -> Msg {
        FiltersMsg::filter_spec(uid, Self::ChangeName(new_name)).into()
    }
    /// Changes the color of a filter.
    pub fn change_color(uid: LineUid, new_color: ChangeData) -> Msg {
        FiltersMsg::filter_spec(uid, Self::ChangeColor(new_color)).into()
    }
}

impl fmt::Display for FilterSpecMsg {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::ChangeName(_) => write!(fmt, "change name"),
            Self::ChangeColor(_) => write!(fmt, "change color"),
        }
    }
}

#[derive(Debug)]
pub enum FilterMsg {
    /// Adds a new subfilter.
    AddNew,
    /// Updates a subfilter.
    Sub(filter::SubFilter),
    /// Removes a subfilter.
    RmSub(filter::SubFilterUid),
}
impl FilterMsg {
    /// Adds a new subfilter.
    pub fn add_new(uid: FilterUid) -> Msg {
        FiltersMsg::filter(uid, Self::AddNew)
    }
    /// Updates a subfilter.
    pub fn update_sub(uid: FilterUid, sub: filter::SubFilter) -> Msg {
        FiltersMsg::filter(uid, Self::Sub(sub.into()))
    }
    /// Removes a subfilter.
    pub fn rm_sub(uid: FilterUid, sub_uid: filter::SubFilterUid) -> Msg {
        FiltersMsg::filter(uid, Self::RmSub(sub_uid))
    }
}

impl fmt::Display for FilterMsg {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::AddNew => write!(fmt, "add new"),
            Self::Sub(_) => write!(fmt, "subfilter update"),
            Self::RmSub(_) => write!(fmt, "remove subfilter"),
        }
    }
}
