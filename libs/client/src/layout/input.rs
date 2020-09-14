//! Value input helpers.

prelude! {}

define_style! {
    input_style! = {
        width(85%),
        height(70%),
        border_radius(5 px),
        text_align(center),
        margin(none),
        padding(0%, 1%),
        border(none),
        bg({"#3a3a3a"}),
    };

    TEXT_INPUT_STYLE = {
        extends(input_style),
        fg(orange),
        font(code),
    };
    COLOR_INPUT_STYLE = {
        extends(input_style),
    };
}

pub fn text_input(value: &str, onchange: OnChangeAction) -> Html {
    html! {
        <input
            type = "text"
            class = "text_input"
            style = TEXT_INPUT_STYLE
            value = value
            onchange = onchange
        />
    }
}

fn parse_text_data(data: ChangeData) -> Res<String> {
    match data {
        yew::html::ChangeData::Value(txt) => Ok(txt),
        err @ yew::html::ChangeData::Select(_) | err @ yew::html::ChangeData::Files(_) => {
            bail!("unexpected text field update {:?}", err)
        }
    }
}

pub fn string_input(
    model: &Model,
    value: &str,
    msg: impl Fn(Res<String>) -> Msg + 'static,
) -> Html {
    text_input(
        value,
        model.link.callback(move |data| {
            msg(parse_text_data(data)
                .map_err(err::Err::from)
                .chain_err(|| "while parsing string value"))
        }),
    )
}

fn parse_usize_data(data: ChangeData) -> Res<usize> {
    use alloc::parser::Parseable;
    parse_text_data(data).and_then(|txt| usize::parse(txt).map_err(|e| e.into()))
}
pub fn usize_input(model: &Model, value: usize, msg: impl Fn(Res<usize>) -> Msg + 'static) -> Html {
    text_input(
        &value.to_string(),
        model.link.callback(move |data| {
            msg(parse_usize_data(data)
                .map_err(|e| err::Err::from(e))
                .chain_err(|| "while parsing integer value"))
        }),
    )
}

pub fn lifetime_input(
    model: &Model,
    value: time::Lifetime,
    msg: impl Fn(Res<time::Lifetime>) -> Msg + 'static,
) -> Html {
    text_input(
        &value.to_string(),
        model.link.callback(move |data| {
            let lifetime = parse_text_data(data).and_then(|txt| {
                time::Lifetime::from_str(&txt).chain_err(|| "while parsing lifetime value")
            });
            msg(lifetime)
        }),
    )
}

fn parse_u32_data(data: ChangeData) -> Res<u32> {
    use alloc::parser::Parseable;
    parse_text_data(data).and_then(|txt| u32::parse(txt).map_err(|e| e.into()))
}
pub fn u32_input(model: &Model, value: u32, msg: impl Fn(Res<u32>) -> Msg + 'static) -> Html {
    text_input(
        &value.to_string(),
        model.link.callback(move |data| {
            msg(parse_u32_data(data)
                .map_err(|e| err::Err::from(e))
                .chain_err(|| "while parsing integer value"))
        }),
    )
}

pub fn color_input(value: &impl fmt::Display, onchange: OnChangeAction) -> Html {
    html! {
        <input
            type = "color"
            id = "color_input"
            style = COLOR_INPUT_STYLE
            value = value
            onchange = onchange
        />
    }
}

pub fn checkbox(
    checked: bool,
    name: impl Into<String>,
    text: impl Into<String>,
    onchange: OnChangeAction,
) -> Html {
    let name = name.into();

    define_style! {
        POINTER = {
            pointer
        };
    }

    html! {
        <div
            style = POINTER
        >
            <input
                style = POINTER
                type = "checkbox"
                onchange = onchange
                name = name.clone()
                id = name.clone()
                checked = checked
            />
            <label
                style = POINTER
                for = name
            >
                {text.into()}
            </label>
        </div>
    }
}

pub fn radio(
    checked: bool,
    name: impl Into<String>,
    text: impl Into<String>,
    onchange: OnChangeAction,
    onclick: OnClickAction,
    pre_space: bool,
) -> Html {
    define_style! {
        POINTER = {
            pointer,
        };
        PRE_SPACE = {
            inline block,
            width(20 px),
        };
    }

    let name = name.into();
    let pre_space = if pre_space {
        html! {
            <span style = PRE_SPACE/>
        }
    } else {
        html!()
    };
    html! {
        <>
            {pre_space}
            <input
                style = POINTER
                type = "radio"
                onchange = onchange
                checked = checked
                name = name
            />
            <label
                style = POINTER
                onclick = onclick
                for = name
            >
                {text.into()}
            </label>
        </>
    }
}
