//! Helpers for memthol's UI buttons.

use crate::common::*;

// /// Button constructor.
// ///
// /// This type is never constructed, it's only purpose is to make writing button-creation functions
// /// simpler by factoring the generic types for the `onclick` action and the title.
// pub struct Button<A, Title>
// where
//     A: Action,
//     Title: Into<String>,
// {
//     _unused_action: A,
//     _unused_title: Title,
// }

/// Used below to create the button-creation functions.
macro_rules! mk_buttons {
    ($(
        $(#[$meta:meta])*
        pub fn $fn_name:ident(...) {
            class = $button_class:ident
        }
    )*) => {
        $(
            $(#[$meta])*
            pub fn $fn_name(
                model: &crate::Model,
                title: impl Into<String>,
                action: impl OnClick,
            ) -> Html {
                let (class, title, onclick) = (
                    style::class::button::$button_class,
                    title.into(),
                    model.link.callback(action),
                );
                html! {
                    <div
                        class = class
                        title = title
                        onclick = onclick
                    />
                }
            }
        )*
    }
}

mk_buttons! {
    /// Constructs a close button.
    pub fn close(...) { class = CLOSE }
    /// Constructs an add button.
    pub fn add(...) { class = ADD }
    /// Creates an expand button.
    pub fn expand(...) { class = EXPAND }
    /// Creates a refresh button.
    pub fn refresh(...) { class = REFRESH }
    /// Creates a collapse button.
    pub fn collapse(...) { class= COLLAPSE }
    /// Creates an inactive tickbox button.
    pub fn inactive_tickbox(...) { class = INACTIVE_TICK }
    /// Creates an active tickbox button.
    pub fn active_tickbox(...) { class = ACTIVE_TICK }
    /// Creates a move down button.
    pub fn move_down(...) { class = MOVE_DOWN }
    /// Creates a move up button.
    pub fn move_up(...) { class = MOVE_UP }
}

/// Creates a text button.
pub fn text(
    model: &Model,
    text: impl Into<String>,
    title: impl Into<String>,
    action: impl OnClick,
    class: &'static str,
) -> Html {
    html! {
        <a
            class = class
            onclick = model.link.callback(action)
            title = title.into()
        >
            { text.into() }
        </a>
    }
}
