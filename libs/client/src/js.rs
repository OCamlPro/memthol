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
    use super::*;

    #[wasm_bindgen]
    extern "C" {
        /// Retrieves the address of the server.
        #[wasm_bindgen(js_namespace = serverAddr, js_name = get_addr)]
        pub fn get_addr_only() -> String;
        /// Retrieves the port of the server.
        #[wasm_bindgen(js_namespace = serverAddr, js_name = get_port)]
        pub fn get_port_only() -> usize;
    }

    /// Retrieves the address and port of the server.
    pub fn addr_and_port() -> (String, usize) {
        (get_addr_only(), get_port_only())
    }
}
