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

/// Generates HTML for a text input field.
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
/// Generates HTML for an input field with steps.
pub fn step_input(value: &str, step: impl fmt::Display, onchange: OnChangeAction) -> Html {
    html! {
        <input
            type = "text"
            step = step
            class = "text_input"
            style = TEXT_INPUT_STYLE
            value = value
            onchange = onchange
        />
    }
}

/// Parses a modification from a text-input field as a string.
fn parse_text_data(data: ChangeData) -> Res<String> {
    match data {
        yew::html::ChangeData::Value(txt) => Ok(txt),
        err @ yew::html::ChangeData::Select(_) | err @ yew::html::ChangeData::Files(_) => {
            bail!("unexpected text field update {:?}", err)
        }
    }
}

/// Generates a text-input field expecting a string result.
pub fn string_input(
    model: &Model,
    value: &str,
    msg: impl Fn(Res<String>) -> Msg + 'static,
) -> Html {
    text_input(
        value,
        model.link.callback(move |data| {
            msg(parse_text_data(data)
                .map_err(err::Error::from)
                .chain_err(|| "while parsing string value"))
        }),
    )
}

/// Parses a modification from a text-input field as a usize.
fn parse_usize_data(data: ChangeData) -> Res<usize> {
    use alloc::parser::Parseable;
    parse_text_data(data).and_then(|txt| usize::parse(txt).map_err(|e| e.into()))
}
/// Generates a text-input field expecting an integer (`usize`) result.
pub fn usize_input(model: &Model, value: usize, msg: impl Fn(Res<usize>) -> Msg + 'static) -> Html {
    text_input(
        &value.to_string(),
        model.link.callback(move |data| {
            msg(parse_usize_data(data)
                .map_err(|e| err::Error::from(e))
                .chain_err(|| "while parsing integer value"))
        }),
    )
}

/// Generates a text-input field expecting a lifetime-like value.
pub fn lifetime_input(
    model: &Model,
    value: time::Lifetime,
    msg: impl Fn(Res<time::Lifetime>) -> Msg + 'static,
) -> Html {
    text_input(
        &value.to_string(),
        model.link.callback(move |data| {
            let lifetime = parse_text_data(data).and_then(|txt| {
                time::Lifetime::parse_secs(&txt).chain_err(|| "while parsing lifetime value")
            });
            msg(lifetime)
        }),
    )
}

/// Generates a text-input field expecting an optional time-like (SinceStart) value.
pub fn since_start_opt_input(
    model: &Model,
    step: impl fmt::Display,
    value: &Option<time::SinceStart>,
    msg: impl Fn(Res<Option<time::SinceStart>>) -> Msg + 'static,
) -> Html {
    step_input(
        &value
            .map(|t| {
                let mut s = t.to_string();
                loop {
                    match s.pop() {
                        Some('0') => (),
                        Some(c) => {
                            if c != '.' {
                                s.push(c)
                            }
                            break;
                        }
                        None => {
                            s.push('0');
                            break;
                        }
                    }
                }
                s
            })
            .unwrap_or_else(|| "_".into()),
        step,
        model.link.callback(move |data| {
            let time_opt = parse_text_data(data).and_then(|txt| match &txt as &str {
                "" | "_" => Ok(None),
                txt => time::SinceStart::parse_secs(txt)
                    .chain_err(|| "while parsing optional time value")
                    .map(Some),
            });
            msg(time_opt)
        }),
    )
}

fn parse_u32_data(data: ChangeData) -> Res<u32> {
    use alloc::parser::Parseable;
    parse_text_data(data).and_then(|txt| u32::parse(txt).map_err(|e| e.into()))
}
/// Generates a text-input field expecting an integer (`u32`) value.
pub fn u32_input(model: &Model, value: u32, msg: impl Fn(Res<u32>) -> Msg + 'static) -> Html {
    text_input(
        &value.to_string(),
        model.link.callback(move |data| {
            msg(parse_u32_data(data)
                .map_err(|e| err::Error::from(e))
                .chain_err(|| "while parsing integer value"))
        }),
    )
}

/// Generates HTML for a color selector.
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

/// Generates HTML for a checkbox.
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

/// Generates HTML for a radio selector.
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
