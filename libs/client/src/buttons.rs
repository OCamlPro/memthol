//! Helpers for memthol's UI buttons.

use crate::base::*;

/// Creates a close button with no onclick action.
pub fn dummy_close() -> Html {
    html! {
        <img class=style::class::button::CLOSE/>
    }
}

/// Creates a add button with no onclick action.
pub fn dummy_add() -> Html {
    html! {
        <img class=style::class::button::ADD/>
    }
}

/// Creates a close button.
pub fn close<Action>(action: Action) -> Html
where
    Action: OnClick,
{
    html! {
        <img
            class=style::class::button::CLOSE
            onclick=|e| action(e)
        />
    }
}

/// Creates an add button.
pub fn add<Action>(action: Action) -> Html
where
    Action: OnClick,
{
    html! {
        <img
            class=style::class::button::ADD
            onclick=|e| action(e)
        />
    }
}

/// Creates an expand button.
pub fn expand<Action>(action: Action) -> Html
where
    Action: OnClick,
{
    html! {
        <img
            class=style::class::button::EXPAND
            onclick=|e| action(e)
        />
    }
}

/// Creates a collapse button.
pub fn collapse<Action>(action: Action) -> Html
where
    Action: OnClick,
{
    html! {
        <img
            class=style::class::button::COLLAPSE
            onclick=|e| action(e)
        />
    }
}

/// Creates an inactive tickbox button.
pub fn inactive_tickbox<Action>(action: Action) -> Html
where
    Action: OnClick,
{
    html! {
        <img
            class=style::class::button::INACTIVE_TICK
            onclick=|e| action(e)
        />
    }
}

/// Creates an active tickbox button.
pub fn active_tickbox<Action>(action: Action) -> Html
where
    Action: OnClick,
{
    html! {
        <img
            class=style::class::button::ACTIVE_TICK
            onclick=|e| action(e)
        />
    }
}

/// Creates a move down button.
pub fn move_down<Action>(action: Action) -> Html
where
    Action: OnClick,
{
    html! {
        <img
            class=style::class::button::MOVE_DOWN
            onclick=|e| action(e)
        />
    }
}

/// Creates a move up button.
pub fn move_up<Action>(action: Action) -> Html
where
    Action: OnClick,
{
    html! {
        <img
            class=style::class::button::MOVE_UP
            onclick=|e| action(e)
        />
    }
}
