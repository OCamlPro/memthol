//! Messages of the client.

use crate::base::*;

/// A message for the model.
#[derive(Debug)]
pub enum Msg {
    JsInit,
    /// Order to change tab.
    ChangeTab(crate::top_tabs::Tab),
    /// Allocation info message.
    Diff(DiffMsg),
    /// An action over the charts.
    ChartsAction(ChartsMsg),
    /// Do nothing.
    Nop,
    /// Start message.
    Start,
}
impl Msg {
    /// Start message constructor.
    pub fn start() -> Msg {
        Msg::Start
    }
}

/// A message for the collection of charts.
#[derive(Debug)]
pub enum ChartsMsg {
    /// Refresh all charts.
    RefreshAll,
    /// Close a chart.
    Close {
        /// The chart to close.
        uid: ChartUid,
    },
    /// Move a chart.
    Move {
        /// The chart to move.
        uid: ChartUid,
        /// Whether the chart should move up. (False means down.)
        up: bool,
    },
    /// Changes the visibility of a chart.
    Visibility {
        /// The chart to expand.
        uid: ChartUid,
        /// True if the chart must be made visible. (False means hide.)
        show: bool,
    },
}
impl ChartsMsg {
    /// Refresh message constructor.
    pub fn refresh() -> Msg {
        Msg::ChartsAction(Self::RefreshAll)
    }
    /// Close chart message constructor.
    pub fn close(uid: ChartUid) -> Msg {
        Msg::ChartsAction(Self::Close { uid })
    }
    /// Move chart up message constructor.
    pub fn move_up(uid: ChartUid) -> Msg {
        Msg::ChartsAction(Self::Move { uid, up: true })
    }
    /// Move chart down message constructor.
    pub fn move_down(uid: ChartUid) -> Msg {
        Msg::ChartsAction(Self::Move { uid, up: false })
    }
    /// Expand chart message constructor.
    pub fn expand(uid: ChartUid) -> Msg {
        Msg::ChartsAction(Self::Visibility { uid, show: true })
    }
    /// Collapse chart message constructor.
    pub fn collapse(uid: ChartUid) -> Msg {
        Msg::ChartsAction(Self::Visibility { uid, show: false })
    }
}
