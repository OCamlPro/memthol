//! JS bindings.

use wasm_bindgen::prelude::*;

pub use wasm_bindgen::JsValue;

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

    pub fn address() -> Res<(String, usize)> {
        location()
            .and_then(|loc| {
                Ok((
                    loc.hostname()
                        .map_err(|js_val| err::Err::from(format!("{:?}", js_val)))?,
                    usize::from_str_radix(
                        &loc.port()
                            .map_err(|js_val| err::Err::from(format!("{:?}", js_val)))?,
                        10,
                    )
                    .map_err(|e| e.to_string())?,
                ))
            })
            .chain_err(|| "while retrieving server's address and port")
    }
}
