//! Memthol's browser client.

#![recursion_limit = "1024"]

#[macro_use]
pub mod common;

pub mod err;
pub mod js;

pub mod model;

pub mod buttons;
pub mod chart;
pub mod cst;
pub mod filter;
pub mod footer;
pub mod msg;
pub mod point;
pub mod style;

pub use model::Model;
