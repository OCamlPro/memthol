//! JS bindings.

use wasm_bindgen::prelude::*;

pub use wasm_bindgen::JsValue;

pub mod amcharts;

#[wasm_bindgen]
extern "C" {
    /// Issues an alert.
    pub fn alert(s: &str);
}

/// Server info.
pub mod server {
    use crate::common::*;

    fn location() -> Res<web_sys::Location> {
        web_sys::window()
            .map(|w| w.location())
            .ok_or_else(|| err::Err::from("could not retrieve (window) JS location"))
    }

    pub fn address() -> Res<String> {
        location()
            .and_then(|loc| {
                loc.host()
                    .map_err(|js_val| err::Err::from(format!("{:?}", js_val)))
            })
            .chain_err(|| "while retrieving server's address and port")
    }
}
