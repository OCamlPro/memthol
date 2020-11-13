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

//! Handles the assets of the UI's client.

/// Landing page (`index.html`, bytes).
pub static INDEX: &[u8] = include_bytes!("../../rsc/static/index.html");
/// Landing page favicon (bytes).
pub static FAVICON: &[u8] = include_bytes!("../../rsc/static/favicon.png");

/// Wasm client (bytes).
pub static CLIENT_WASM: &[u8] = include_bytes!(concat!(
    "../../",
    base::client_wasm_build_dir!(),
    "/client_bg.wasm"
));
/// JS client (bytes).
pub static CLIENT_JS: &[u8] = include_bytes!(concat!(
    "../../",
    base::client_wasm_build_dir!(),
    "/client.js"
));
