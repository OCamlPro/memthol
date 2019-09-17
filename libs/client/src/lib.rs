//! Memthol's browser client.

#![recursion_limit = "1024"]

#[macro_use]
mod base;
#[macro_use]
mod uid;

pub mod buttons;
pub mod chart;
pub mod cst;
pub mod data;
pub mod err;
pub mod filter;
pub mod footer;
mod model;
pub mod msg;
pub mod nu_chart;
pub mod nu_data;
pub mod point;
pub mod style;
mod tmp;
pub mod top_tabs;

pub use model::Model;
pub use msg::Msg;
