//! Memthol's browser client.

#![recursion_limit = "1024"]

#[macro_use]
pub mod prelude;

#[macro_use]
pub mod macros;

pub mod err;
pub mod js;
pub mod layout;
pub mod style;

pub mod model;

pub mod chart;
pub mod cst;
pub mod filter;
pub mod footer;
pub mod msg;

prelude! {}
use wasm::*;

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen(start)]
pub fn main() {
    wasm_logger::init(wasm_logger::Config::default());
    yew::start_app::<Model>();
}
