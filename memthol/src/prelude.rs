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

//! Basic types and helpers used by the whole crate.

pub use std::net::{TcpListener, TcpStream};

pub use base::{prelude::*, time};

pub use charts::{
    alloc_data::{Alloc, Diff as AllocDiff, Init as AllocInit},
    Charts,
};

pub use crate::msg;

/// A set of allocation UIDs.
pub type AllocUidSet = BTSet<uid::Alloc>;

/// Re-export for network-related stuff.
pub mod net {
    pub use std::net::{SocketAddr as IpAddr, TcpListener, TcpStream};

    pub use tungstenite::{protocol::CloseFrame, Message as Msg};

    /// Type alias for a tungstenite websocket for a TCP stream.
    pub type WebSocket = tungstenite::WebSocket<TcpStream>;
}

/// Type of the result of receiving messages from the client.
pub struct FromClient {
    /// Messages from the client, need to be handled before the next rendering phase.
    messages: Vec<msg::from_client::Msg>,
    /// True if the client requested to close the connection.
    closed: bool,
    ///
    close_data: Option<net::CloseFrame<'static>>,
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
        if self.closed {
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
    pub fn set_close_data(&mut self, data: Option<net::CloseFrame<'static>>) -> Res<()> {
        if !self.closed {
            bail!("trying to set close data of an open connection")
        }
        // Set close_data.
        let prev = std::mem::replace(&mut self.close_data, data);
        if prev.is_some() {
            bail!("trying to set the close data of a connection twice")
        }
        Ok(())
    }

    /// Close data accessor.
    pub fn close_data(&self) -> Option<&net::CloseFrame<'static>> {
        self.close_data.as_ref()
    }
}
