//! Memthol's browser client.

#![recursion_limit = "1024"]

mod base;
#[macro_use]
mod uid;

pub mod chart;
pub mod cst;
pub mod data;
mod model;
mod tmp;
pub mod top_tabs;

pub use base::Msg;
pub use model::Model;
