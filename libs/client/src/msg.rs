/*<LICENSE>
    This file is part of Memthol.

    Copyright (C) 2020 OCamlPro.

    Memthol is free software: you can redistribute it and/or modify
    it under the terms of the GNU General Public License as published by
    the Free Software Foundation, either version 3 of the License, or
    (at your option) any later version.

    Memthol is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
    GNU General Public License for more details.

    You should have received a copy of the GNU General Public License
    along with Memthol.  If not, see <https://www.gnu.org/licenses/>.
*/

//! Messages of the client.

prelude! {}

pub use charts::msg::{to_client as from_server, to_server, ChartSettingsMsg};

/// Filter messages.
pub mod filter {
    pub use crate::filter::{FilterMsg, Msg, SpecMsg};
}
/// Settings messages.
pub mod settings {
    pub use crate::settings::Msg;
}

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
    Filter(filter::Msg),
    /// Settings operations.
    Settings(settings::Msg),

    /// A message to print in the JS console.
    Msg(String),
    /// A warning to print in the JS console.
    Warn(String),
    /// An error.
    Err(err::Error),

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
    pub fn err(e: impl Into<err::Error>) -> Self {
        Self::Err(e.into())
    }
}

/// Operations over charts.
#[derive(Debug)]
pub enum ChartsMsg {
    /// Moves a chart up or down.
    Move {
        /// UID of the chart.
        uid: uid::Chart,
        /// Move up if true, down otherwise.
        up: bool,
    },

    /// Message for a specific chart message.
    ChartMsg {
        /// UID of the chart the message is for.
        uid: uid::Chart,
        /// Actual message.
        msg: ChartMsg,
    },

    /// Destroys a chart.
    Destroy(uid::Chart),

    /// Forces to refresh the filters.
    RefreshFilters,

    /// Sets the x-axis in the new chart element.
    NewChartSetX(chart::axis::XAxis),
    /// Sets the y-axis in the new chart element.
    NewChartSetY(chart::axis::YAxis),
}
impl ChartsMsg {
    /// Constructs a message to move a chart up.
    pub fn move_up(uid: uid::Chart) -> Msg {
        Self::Move { uid, up: true }.into()
    }
    /// Constructs a message to move a chart down.
    pub fn move_down(uid: uid::Chart) -> Msg {
        Self::Move { uid, up: false }.into()
    }
    /// Constructs a message to destroy a chart.
    pub fn destroy(uid: uid::Chart) -> Msg {
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

/// A message for a specific chart.
#[derive(Debug)]
pub enum ChartMsg {
    /// Toggles a chart's visibility.
    SettingsToggleVisible,
    /// Toggles a filter's visibility.
    FilterToggleVisible(uid::Line),
    /// Updates the chart's settings.
    SettingsUpdate(ChartSettingsMsg),
}

impl ChartMsg {
    /// Toggles a chart's visibility.
    pub fn settings_toggle_visible(uid: uid::Chart) -> ChartsMsg {
        (uid, Self::SettingsToggleVisible).into()
    }
    /// Toggles a filter's visibility.
    pub fn filter_toggle_visible(uid: uid::Chart, line: uid::Line) -> ChartsMsg {
        (uid, Self::FilterToggleVisible(line)).into()
    }
}

/// Footer operation.
#[derive(Debug)]
pub enum FooterMsg {
    /// Toggles a tab.
    ToggleTab(footer::FooterTab),
    // /// Lets the footer know a filter was removed.
    // Removed(uid::Filter),
}
impl FooterMsg {
    /// Toggles a tab.
    pub fn toggle_tab(tab: impl Into<footer::FooterTab>) -> Msg {
        Self::ToggleTab(tab.into()).into()
    }
    // /// Lets the footer know a filter was removed.
    // pub fn removed(uid: uid::Filter) -> Msg {
    //     Self::Removed(uid).into()
    // }
}

base::implement! {
    impl Msg {
        Display {
            |&self, fmt| match self {
                Self::FromServer(_) => write!(fmt, "from the server"),
                Self::ToServer(_) => write!(fmt, "for the server"),
                Self::ConnectionStatus(_) => write!(fmt, "connection status"),
                Self::Charts(charts_msg) => write!(fmt, "charts, {}", charts_msg),
                Self::Footer(footer_msg) => write!(fmt, "footer, {}", footer_msg),
                Self::Filter(filter_msg) => write!(fmt, "filter, {}", filter_msg),
                Self::Settings(settings_msg) => write!(fmt, "settings, {}", settings_msg),
                Self::Msg(_) => write!(fmt, "info"),
                Self::Warn(_) => write!(fmt, "warning"),
                Self::Err(_) => write!(fmt, "error"),
                Self::Noop => write!(fmt, "noop"),
            }
        }
        From {
            from String => |s| Self::Msg(s),
            from err::Error => |e| Self::err(e),
            from from_server::RawMsg => |msg| Self::FromServer(msg),
            from to_server::Msg => |msg| Self::ToServer(msg),
            from ChartsMsg => |msg| Self::Charts(msg),
            from Res<ChartsMsg> => |res| match res {
                Ok(msg) => msg.into(),
                Err(e) => Self::err(e),
            },
            from FooterMsg => |msg| Self::Footer(msg),
            from settings::Msg => |msg| Self::Settings(msg),
        }
    }

    impl ChartsMsg {
        Display {
            |&self, fmt| match self {
                Self::Move { uid, up } => write!(fmt, "move {}/{}", uid, up),
                Self::Destroy(c_uid) => write!(fmt, "destroy {}", c_uid),
                Self::RefreshFilters => write!(fmt, "refresh filters"),
                Self::NewChartSetX(_) => write!(fmt, "new-chart-set-x"),
                Self::NewChartSetY(_) => write!(fmt, "new-chart-set-y"),
                Self::ChartMsg { uid, msg } => write!(fmt, "chart[{}]: {}", uid, msg),
            }
        }

        From {
            from (uid::Chart, ChartMsg) => |(uid, msg)| Self::ChartMsg { uid, msg },
            from (uid::Chart, ChartSettingsMsg) => |(uid, msg)| Self::ChartMsg {
                uid, msg: msg.into()
            },
        }
    }


    impl ChartMsg {
        Display {
            |&self, fmt| match self {
                Self::SettingsToggleVisible => write!(fmt, "settings toggle visible"),
                Self::FilterToggleVisible(l_uid) => write!(fmt, "filter toggle visible {}", l_uid),
                Self::SettingsUpdate(msg) => write!(fmt, "{}", msg),
            }
        }

        From {
            from ChartSettingsMsg => |msg| Self::SettingsUpdate(msg),
        }
    }

    impl FooterMsg {
        Display {
            |&self, fmt| match self {
                Self::ToggleTab(_) => write!(fmt, "toggle tab"),
                // Self::Removed(f_uid) => write!(fmt, "remove {}", f_uid),
            }
        }
    }
}
