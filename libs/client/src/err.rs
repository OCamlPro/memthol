//! Error-handling types.

prelude! {}

use msg::Msg;

pub use charts::err::*;

/// Turns a `JsValue` into an error.
pub fn from_js_val(js_val: wasm_bindgen::JsValue) -> Err {
    let msg = js_val
        .as_string()
        .unwrap_or_else(|| format!("{:?}", js_val));
    Err::from(msg)
}

/// Turns a result into a message.
///
/// If it's an error, it becomes an error message.
pub fn msg_of_res(res: Res<Msg>) -> Msg {
    match res {
        Ok(msg) => msg,
        Result::Err(e) => e.into(),
    }
}
