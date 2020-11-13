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
