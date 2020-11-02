//! Memthol's browser client.

#![deny(missing_docs)]
#![recursion_limit = "1024"]

#[macro_use]
pub mod prelude;

#[macro_use]
pub mod macros;

pub mod js;
pub mod layout;

pub mod model;
pub mod settings;

pub mod chart;
pub mod cst;
pub mod filter;
pub mod footer;
pub mod msg;

prelude! {}
use wasm::*;

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

/// WASM client entry point.
#[wasm_bindgen(start)]
pub fn main() {
    wasm_logger::init(wasm_logger::Config::default());
    yew::start_app::<Model>();
}
