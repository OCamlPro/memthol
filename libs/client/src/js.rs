//! JS bindings.

prelude! {}

use wasm::*;

pub const HORIZONTAL_SCROLL_SCRIPT: &str = r#"
var items = document.getElementsByClassName('horizontal_scroll');

var len = items.length;
console.log("adding wheel stuff to " + len + " elements");
for (var i = 0; i < len; i++) {
    var item = items.item(i);
    item.addEventListener(
        'wheel',
        function(e) {
            console.log("wheel event, deltaY: " + e.deltaY);
            if (e.deltaY > 0) item.scrollLeft += 100;
            else item.scrollLeft -= 100;
        }
    )
}
"#;

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
            .ok_or_else(|| err::Error::from("could not retrieve (window) JS location"))
    }

    pub fn address() -> Res<(String, usize)> {
        location()
            .and_then(|loc| {
                Ok((
                    loc.hostname()
                        .map_err(|js_val| err::Error::from(format!("{:?}", js_val)))?,
                    usize::from_str_radix(
                        &loc.port()
                            .map_err(|js_val| err::Error::from(format!("{:?}", js_val)))?,
                        10,
                    )
                    .map_err(|e| e.to_string())?,
                ))
            })
            .chain_err(|| "while retrieving server's address and port")
    }
}
