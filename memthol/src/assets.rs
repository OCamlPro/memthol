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
