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

//! Common imports for this crate
pub use yew::{
    html,
    html::ChangeData,
    services::websocket::{WebSocketService, WebSocketStatus, WebSocketTask},
    Callback, Component, Renderable, ShouldRender,
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
    cst, filter, js,
    layout::{self, footer, header},
    model::Model,
    msg::{self, Msg},
    settings,
};

/// Component link to the model, can send messages to the model.
pub type Link = yew::ComponentLink<Model>;

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
