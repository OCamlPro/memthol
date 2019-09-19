//! Messages of the client.

use crate::base::*;

pub use charts::msg::{to_client as from_server, to_server};

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

#[derive(Debug)]
pub enum ChartsMsg {
    /// Builds a chart and attaches it to its container.
    Build(charts::uid::ChartUid),
    // /// Moves a chart up or down.
    // Move { index: index::Chart, up: bool },
}
impl ChartsMsg {
    pub fn build(uid: charts::uid::ChartUid) -> Msg {
        Self::Build(uid).into()
    }
}
