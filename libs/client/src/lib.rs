//! Memthol's browser client.

#![recursion_limit = "1024"]

use wasm_bindgen::prelude::*;

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

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen(start)]
pub fn main() {
    wasm_logger::init(wasm_logger::Config::default());
    yew::start_app::<Model>();
}
