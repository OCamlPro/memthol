//! JS bindings.

prelude! {}

use wasm::*;

#[wasm_bindgen]
extern "C" {
    /// Issues an alert.
    pub fn alert(s: &str);
}

/// Alias type for `wasm_bindgen`'s `JsValue`.
pub type Value = JsValue;

/// Retrieves a DOM element from its id.
pub fn try_get_element_by_id(id: &str) -> Res<Option<web_sys::Element>> {
    let document = web_sys::window()
        .ok_or("could not retrieve window")?
        .document()
        .ok_or("could not retrieve document from window")?;

    Ok(document.get_element_by_id(id))
}

/// Retrieves a DOM element from its id.
pub fn get_element_by_id(id: &str) -> Res<web_sys::Element> {
    let res =
        try_get_element_by_id(id)?.ok_or_else(|| format!("unknown DOM element id {:?}", id))?;
    Ok(res)
}

/// Server info.
pub mod server {
    prelude! {}

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
