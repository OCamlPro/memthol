//! Memthol's browser client.

#![recursion_limit = "1024"]

mod base;
#[macro_use]
mod uid;

pub mod chart;
pub mod data;
pub mod filter;
pub mod footer;
mod model;
pub mod msg;
pub mod style;
mod tmp;
pub mod top_tabs;

pub use model::Model;
pub use msg::Msg;
