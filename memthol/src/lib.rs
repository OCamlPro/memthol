//! Memthol's UI.
//!
//! Memthol's UI is decomposed in two parts: the *client* and the server. The client is the part
//! users interact with in their browser. It is compiled to webassembly.
//!
//! The server (this crate) is compiled normally and is responsible for
//!
//! - monitoring the files in the user-provided dump directory;
//! - organising the data from the diffs;
//! - answer browser's queries by sending them the client;
//! - maintain one session per client that performs whatever treatment the user requests.
//!
//! The documentation for the server is the present document. The client's crate is in the
//! `./libs/client` from the root of the repository.
//!
//! # Common Crates
//!
//! The client and the server have quite a lot of code in common: types for diffs, allocations,
//! charts, filters... Everything related to "raw data" is in the [`alloc_data` crate]: diff and
//! allocation types, but also parsing and file monitoring. This crate is in the `./libs/data`
//! directory from the root of the repo.
//!
//! The [`charts` crate] deals with chart representation, and how charts handle the raw data. It
//! also defines the messages that the server and the client can exchange.
//!
//! [`alloc_data` crate]: ../alloc_data/index.html (Memthol's alloc_data crate)
//! [`charts` crate]: ../charts/index.html (Memthol's charts crate)

#[macro_use]
pub mod prelude;

pub mod assets;
pub mod msg;
pub mod router;
pub mod socket;
