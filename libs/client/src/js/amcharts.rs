//! Bindings for [`amcharts`][amcharts].
//!
//! [amcharts]: https://www.amcharts.com/ (amcharts official)

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = am4core)]
    fn color(s: &str) -> JsValue;
}
