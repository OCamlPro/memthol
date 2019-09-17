//! Basic types and helpers used by the whole crate.

pub use std::{
    collections::BTreeMap as Map,
    collections::BTreeSet as Set,
    net::{TcpListener, TcpStream},
};

pub use error_chain::bail;

pub use alloc_data::{Alloc, Diff, Init as AllocInit, SinceStart, Uid as AllocUid};
pub use charts::{msg, Charts, Json};

pub use crate::{
    err,
    err::{Res, ResultExt},
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
