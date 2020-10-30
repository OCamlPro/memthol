//! Common imports for this crate
pub use yew::{
    html,
    html::ChangeData,
    services::websocket::{WebSocketService, WebSocketStatus, WebSocketTask},
    Callback, Component, ComponentLink, Renderable, ShouldRender,
};
pub use yew_components::Select;

pub use base::prelude::*;

pub use charts::{
    palette,
    prelude::{
        alloc, filter::stats::AllFilterStats, num_fmt, time, Alloc, AllocStats, LoadInfo, Regex,
    },
};

/// Re-exports from `plotters`, `plotters_canvas`, and `palette`.
pub mod plotters {
    pub use charts::plotters::*;
    pub use plotters_canvas::CanvasBackend;
}

/// Wasm-bindgen re-exports.
pub mod wasm {
    pub use wasm_bindgen::prelude::*;
}

/// Imports this crate's prelude.
macro_rules! prelude {
    () => {
        use $crate::prelude::*;
    };
}

/// Turns a `JsValue` into an error.
pub fn error_from_js_val(js_val: wasm_bindgen::JsValue) -> err::Error {
    let msg = js_val
        .as_string()
        .unwrap_or_else(|| format!("{:?}", js_val));
    err::Error::from(msg)
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

/// Re-exports from `charts::point`.
pub mod point {
    pub use charts::point::{Point, Points, TimePoints};
}
pub use point::Point;

pub use crate::{
    chart::{self, Chart, Charts},
    cst, filter, footer, js, layout,
    model::Model,
    msg::{self, Msg},
    settings,
};

/// Trait for conversion to JS.
pub trait JsExt {
    /// Conversion to JS.
    fn as_js(self) -> js::Value;
}

/// Type alias for mouse-event callbacks.
pub type OnClickAction = Callback<yew::events::MouseEvent>;
/// Type alias for text-input-like callbacks.
pub type OnChangeAction = Callback<yew::events::ChangeData>;

/// Type of `onclick` actions.
pub trait OnClick: Fn(yew::events::MouseEvent) -> Msg + 'static {}
impl<Action> OnClick for Action where Action: Fn(yew::events::MouseEvent) -> Msg + 'static {}

/// Type alias for a generic model callback.
pub type Action = Callback<Model>;

/// Type of HTML elements in the client.
pub type Html = yew::Html;
