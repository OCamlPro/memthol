//! Basic types and helpers used by the whole crate.

pub use std::{
    collections::BTreeMap as Map,
    collections::BTreeSet as Set,
    net::{TcpListener, TcpStream},
    time::{Duration, Instant},
};

pub use error_chain::bail;

pub use websocket::message::CloseData;

pub use alloc_data::{Alloc, Diff, Init as AllocInit, SinceStart, Uid as AllocUid};
pub use charts::{Charts, Json};

pub use crate::{
    err,
    err::{Res, ResExt},
    msg,
};

/// A set of allocation UIDs.
pub type AllocUidSet = Set<AllocUid>;

/// A websocket server.
pub type Server = websocket::sync::Server<websocket::server::NoTlsAcceptor>;

/// A request.
pub type Request = websocket::server::upgrade::WsUpgrade<
    std::net::TcpStream,
    Option<websocket::server::upgrade::sync::Buffer>,
>;

/// An IP address.
pub type IpAddr = std::net::SocketAddr;
/// A receiver for a request.
pub type Receiver = websocket::receiver::Reader<std::net::TcpStream>;
/// A sender for a request.
pub type Sender = websocket::sender::Writer<std::net::TcpStream>;

/// Type of the result of receiving messages from the client.
pub struct FromClient {
    /// Messages from the client, need to be handled before the next rendering phase.
    messages: Vec<msg::from_client::Msg>,
    /// True if the client requested to close the connection.
    closed: bool,
    /// Optional close data. It can happen that `self.closed` is true while
    /// `self.close_data.is_none()`. The only invariant here is that `self.close_data.is_some()`
    /// implies `self.closed`.
    close_data: Option<CloseData>,
}
impl FromClient {
    /// Constructor: no messages, not closed and no close data.
    pub fn new() -> Self {
        Self {
            messages: vec![],
            closed: false,
            close_data: None,
        }
    }

    /// Pushes a message.
    ///
    /// Fails if either
    ///
    /// - `self.close()` was called before, or
    /// - `self.close_data(_)` was called before.
    pub fn push(&mut self, msg: msg::from_client::Msg) -> Res<()> {
        // The second part of this disjunction should be redundant. It's there to be safe, in case
        // this struct's workflow changes.
        if self.closed || self.close_data.is_some() {
            bail!("receiving messages from a closed connection")
        }
        self.messages.push(msg);
        Ok(())
    }

    /// Drains all the messages.
    pub fn drain(&mut self) -> std::vec::Drain<msg::from_client::Msg> {
        self.messages.drain(0..)
    }

    /// Sets the closed flag.
    ///
    /// Fails if
    ///
    /// - `self.close()` was called before.
    pub fn close(&mut self) -> Res<()> {
        if self.closed {
            bail!("trying to close a connection twice")
        }
        self.closed = true;
        Ok(())
    }

    /// True if the connection is closed.
    pub fn is_closed(&self) -> bool {
        self.closed
    }

    /// Sets the close data.
    ///
    /// Fails if either
    ///
    /// - `self.close()` was **not** called before, or
    /// - `self.set_close_data(data)` was called before and `data.is_some()`.
    pub fn set_close_data(&mut self, data: Option<CloseData>) -> Res<()> {
        if !self.closed {
            bail!("trying to set close data of an open connection")
        }
        // Set close_data.
        let prev = std::mem::replace(&mut self.close_data, data);
        if let Some(_) = prev {
            bail!("trying to set the close data of a connection twice")
        }
        Ok(())
    }

    /// Close data accessor.
    pub fn close_data(&self) -> Option<&CloseData> {
        self.close_data.as_ref()
    }
}