//! Helpers for memthol's UI buttons.

use crate::base::*;

/// Button constructor.
///
/// This type is never constructed, it's only purpose is to make writing button-creation functions
/// simpler by factoring the generic types for the `onclick` action and the title.
pub struct Button<Action, Title>
where
    Action: OnClick,
    Title: Into<String>,
{
    _unused_action: Action,
    _unused_title: Title,
}

/// Used below to create the button-creation functions.
macro_rules! mk_buttons {
    ($(
        $(#[$meta:meta])*
        pub fn $fn_name:ident(...) {
            class = $button_class:ident
        }
    )*) => {
        impl<Action, Title> Button<Action, Title>
        where Action: OnClick, Title: Into<String> {
            $(
                $(#[$meta])*
                pub fn $fn_name(title: Title, action: Action) -> Html {
                    html! {
                        <img
                            class = style::class::button::$button_class
                            title = title.into()
                            onclick = |e| action(e)
                        />
                    }
                }
            )*
        }
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
