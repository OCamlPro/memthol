//! Common imports for this crate

pub use std::{
    collections::BTreeMap as Map,
    collections::BTreeSet as Set,
    convert::{TryFrom, TryInto},
    fmt,
    ops::Deref,
    str::FromStr,
    time::Duration,
};

pub use log::{debug, error, info, warn};
pub use yew::{
    html,
    html::ChangeData,
    services::websocket::{WebSocketService, WebSocketStatus, WebSocketTask},
    Callback, Component, ComponentLink, Renderable, ShouldRender,
};
pub use yew_components::Select;

pub use base::{
    error_chain::{self, bail},
    impl_display, lazy_static, svec, SVec,
};

pub use charts::prelude::{alloc, time, Alloc, AllocDiff, AllocUid, Json, LoadInfo, Regex};

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

/// Re-exports from `charts::point`.
pub mod point {
    pub use charts::point::{Point, Points, TimePoints};
}
pub use point::Point;

pub use crate::{
    chart::{self, Chart, Charts},
    cst,
    err::{self, Res, ResExt},
    filter, footer, js, layout,
    model::Model,
    msg::{self, Msg},
    style,
};

/// Trait for conversion to JS.
pub trait JsExt {
    /// Conversion to JS.
    fn as_js(self) -> js::Value;
}

pub type OnClickAction = Callback<yew::events::MouseEvent>;
pub type OnChangeAction = Callback<yew::events::ChangeData>;

/// Type of `onclick` actions.
pub trait OnClick: Fn(yew::events::MouseEvent) -> Msg + 'static {}
impl<Action> OnClick for Action where Action: Fn(yew::events::MouseEvent) -> Msg + 'static {}

pub type Action = Callback<Model>;

/// Type of HTML elements in the client.
pub type Html = yew::Html;
