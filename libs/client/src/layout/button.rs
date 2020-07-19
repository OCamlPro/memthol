//! Button rendering.

prelude! {}

/// Button box properties.
///
/// Includes information about
///
/// - the border: color and strokewidth
///     - [`with_border_color`](#method.with_border_color)
///     - [`with_stroke_px`](#method.with_stroke_px)
/// - the border rounded corners: radius and which corner is rounded
///     - [`with_radius_px`](#method.with_radius_px)
/// - background color gradient: top and bottom
///     - [`with_gradient_top`](#method.with_gradient_top)
///     - [`with_gradient_bot`](#method.with_gradient_bot)
/// - z-index: whether the text in the box is on top or not (*top* here really means that the button
///   is part of the footer)
///     - [`for_footer`](#method.for_footer)
#[derive(Clone, Copy)]
pub struct BoxProps<'color> {
    border_color: &'color str,
    gradient_top: &'color str,
    gradient_bot: &'color str,
    radius_px: u8,
    stroke_px: u8,
    tl_rounded: bool,
    tr_rounded: bool,
    bl_rounded: bool,
    br_rounded: bool,
    top: bool,
    no_wrap: bool,
}

impl<'color> BoxProps<'color> {
    pub const fn new(border_color: &'color str) -> Self {
        Self {
            radius_px: 5,
            stroke_px: 1,
            border_color,
            tl_rounded: false,
            tr_rounded: false,
            bl_rounded: false,
            br_rounded: false,
            top: false,
            no_wrap: false,
            gradient_top: "#c1c1c1",
            gradient_bot: "black",
        }
    }

    pub const fn new_button(border_color: &'color str) -> Self {
        Self {
            radius_px: 5,
            stroke_px: 1,
            border_color,
            tl_rounded: true,
            tr_rounded: true,
            bl_rounded: true,
            br_rounded: true,
            top: false,
            no_wrap: false,
            gradient_top: "#c1c1c1",
            gradient_bot: "black",
        }
    }

    pub const fn new_tab(border_color: &'color str) -> Self {
        Self {
            radius_px: 5,
            stroke_px: 1,
            border_color,
            tl_rounded: true,
            tr_rounded: true,
            bl_rounded: false,
            br_rounded: false,
            top: false,
            no_wrap: true,
            gradient_top: "#c1c1c1",
            gradient_bot: "black",
        }
    }

    /// Inverts rounded corner flags and gradient top and bottom.
    pub fn revert_if(mut self, rev: bool) -> Self {
        if rev {
            self.tl_rounded = !self.tl_rounded;
            self.tr_rounded = !self.tr_rounded;
            self.bl_rounded = !self.bl_rounded;
            self.br_rounded = !self.br_rounded;
            std::mem::swap(&mut self.gradient_top, &mut self.gradient_bot);
        }
        self
    }

    /// Sets the color of the border.
    pub const fn with_border_color(mut self, border_color: &'color str) -> Self {
        self.border_color = border_color;
        self
    }

    /// Sets the strokewidth of the border, in pixels.
    pub const fn with_stroke_px(mut self, stroke_px: u8) -> Self {
        self.stroke_px = stroke_px;
        self
    }

    /// Sets the radius of the rounded corners of the border, in pixels.
    pub const fn with_radius_px(mut self, radius_px: u8) -> Self {
        self.radius_px = radius_px;
        self
    }

    /// Sets the top color of the gradient.
    pub const fn with_gradient_top(mut self, color: &'color str) -> Self {
        self.gradient_top = color;
        self
    }
    /// Sets the bottom color of the gradient.
    pub const fn with_gradient_bot(mut self, color: &'color str) -> Self {
        self.gradient_bot = color;
        self
    }

    /// Forbids wrapping the button's text (if any).
    pub const fn with_wrap_text(mut self, wrap: bool) -> Self {
        self.no_wrap = !wrap;
        self
    }

    /// Indicates whether the button should be "on top", *i.e.* is a footer button.
    pub const fn for_footer(mut self, top: bool) -> Self {
        self.top = top;
        self
    }
}

lazy_static::lazy_static! {
    static ref DEFAULT_BUTTON_BOX_PROPS: BoxProps<'static> = {
        BoxProps::new_button("white")
            .with_radius_px(5)
            .with_stroke_px(1)
            .with_gradient_top("#8e8e8e")
            .with_gradient_bot("black")
    };
}

impl fmt::Display for BoxProps<'_> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        let radius_px = |rounded: bool| {
            if rounded {
                self.radius_px
            } else {
                0
            }
        };

        // Discarding potential errors here... Sorry about that.
        #[allow(unused_results)]
        css!(fmt: fmt,
            height(100%),
            width(auto),
            table,
            bg(gradient {self.gradient_top} to {self.gradient_bot}),
            border({self.stroke_px} px, {self.border_color}),
            border_radius(
                {radius_px(self.tl_rounded)} px,
                {radius_px(self.tr_rounded)} px,
                {radius_px(self.br_rounded)} px,
                {radius_px(self.bl_rounded)} px,
            ),
            if(
                self.no_wrap,
                white_space(nowrap),
            ),
        );

        Ok(())
    }
}

/// Text buttons.
pub mod text {
    use super::*;

    define_style! {
        link_box! = {
            width(auto),
            height(100%),
            table,
        };
        LINK_BOX = {
            extends(link_box),
            pointer,
        };
        HIDDEN_LINK_BOX = {
            extends(link_box),
            visi(hidden),
        };

        content_style! = {
            font_size(120%),
            font_outline(black),
            vertical_align(middle),
            table cell,
            padding(0%, 10 px),
            underline,
            pos(relative),
            z_index(400),
        };

        CONTENT_STYLE = {
            extends(content_style),
            fg(white),
        };
        DIMMED_CONTENT_STYLE = {
            extends(content_style),
            fg("#8a8a8a"),
        };

        top_content_style! = {
            font_size(120%),
            font_outline(black),
            vertical_align(middle),
            table cell,
            padding(0%, 10 px),
            underline,
            pos(relative),
            z_index(800),
        };

        TOP_CONTENT_STYLE = {
            extends(top_content_style),
            fg(white),
        };
        TOP_DIMMED_CONTENT_STYLE = {
            extends(top_content_style),
            fg("#8a8a8a"),
        };
    }

    fn centered(inner: Html, dimmed: bool) -> Html {
        html! {
            <a
                style = if dimmed { &*DIMMED_CONTENT_STYLE } else { &*CONTENT_STYLE }
            >
                {inner}
            </a>
        }
    }
    fn top_centered(inner: Html, dimmed: bool) -> Html {
        html! {
            <a
                style = if dimmed { &*TOP_DIMMED_CONTENT_STYLE } else { &*TOP_CONTENT_STYLE }
            >
                {inner}
            </a>
        }
    }

    pub fn render_default_button(
        id: impl fmt::Display,
        txt: impl fmt::Display,
        onclick: Option<OnClickAction>,
        dimmed: bool,
    ) -> Html {
        render(Some(*DEFAULT_BUTTON_BOX_PROPS), id, txt, onclick, dimmed)
    }

    pub fn render(
        props: Option<BoxProps<'_>>,
        id: impl fmt::Display,
        txt: impl fmt::Display,
        onclick: Option<OnClickAction>,
        dimmed: bool,
    ) -> Html {
        let mut inner = if props.as_ref().map(|props| props.top).unwrap_or(false) {
            top_centered(html! {{txt}}, dimmed)
        } else {
            centered(html! {{txt}}, dimmed)
        };
        if let Some(props) = props {
            inner = html! {
                <div
                    style = props
                >
                    {inner}
                </div>
            }
        }
        if let Some(onclick) = onclick {
            html! {
                <div
                    id = id
                    style = LINK_BOX
                    onclick = onclick
                >
                    {inner}
                </div>
            }
        } else {
            html! {
                <div
                    id = id
                    style = HIDDEN_LINK_BOX
                >
                    {inner}
                </div>
            }
        }
    }

    pub(super) fn img_render(
        dimension_px: usize,
        props: Option<BoxProps<'_>>,
        id: impl fmt::Display,
        img: img::Img,
        desc: impl fmt::Display,
        onclick: OnClickAction,
        dimmed: bool,
    ) -> Html {
        let mut inner = if props.as_ref().map(|props| props.top).unwrap_or(false) {
            top_centered(
                img.render(
                    Some(dimension_px),
                    format!("{}_content", id),
                    Some(onclick),
                    desc,
                ),
                dimmed,
            )
        } else {
            centered(
                img.render(
                    Some(dimension_px),
                    format!("{}_content", id),
                    Some(onclick),
                    desc,
                ),
                dimmed,
            )
        };
        if let Some(props) = props {
            inner = html! {
                <div
                    style = props
                >
                    {inner}
                </div>
            }
        }
        html! {
            <div
                id = id
                style = LINK_BOX
            >
                {inner}
            </div>
        }
    }
}

/// Image buttons.
///
/// Images come from the [getbootstrap] free image library.
///
/// [getbootstrap]: https://icons.getbootstrap.com (getbootstrap official website)
pub mod img {
    use super::*;

    define_style! {
        link_box! = {
            width(100%),
            height(100%),
            border(none),
            margin(none),
            padding(none),
            bg(transparent),
            fg(inherit),
            display(table),
        };
        LINK_BOX = {
            extends(link_box),
            pointer,
        };
        HIDDEN_LINK_BOX = {
            extends(link_box),
            visi(hidden),
        };
    }

    #[derive(Clone, Copy, Debug)]
    pub enum Img {
        Close,
        Collapse,
        Expand,
        ArrowUp,
        ArrowDown,
        Plus,
        Minus,
        Undo,
        Check,
    }
    impl Img {
        pub fn render(
            self,
            dimension_px: Option<usize>,
            id: impl fmt::Display,
            onclick: Option<OnClickAction>,
            desc: impl fmt::Display,
        ) -> Html {
            match self {
                Self::Close => close(dimension_px, id, onclick, desc),
                Self::Collapse => collapse(dimension_px, id, onclick, desc),
                Self::Expand => expand(dimension_px, id, onclick, desc),
                Self::ArrowUp => arrow_up(dimension_px, id, onclick, desc),
                Self::ArrowDown => arrow_down(dimension_px, id, onclick, desc),
                Self::Plus => plus(dimension_px, id, onclick, desc),
                Self::Minus => minus(dimension_px, id, onclick, desc),
                Self::Undo => undo(dimension_px, id, onclick, desc),
                Self::Check => check(dimension_px, id, onclick, desc),
            }
        }

        pub fn button_render(
            self,
            dimension_px: usize,
            props: Option<BoxProps<'_>>,
            id: impl fmt::Display,
            desc: impl fmt::Display,
            onclick: OnClickAction,
            dimmed: bool,
        ) -> Html {
            text::img_render(dimension_px, props, id, self, desc, onclick, dimmed)
        }
    }

    /// Internal rendering function.
    ///
    /// If `onclick.is_none()`, the button will be hidden and deactivated.
    fn raw_render(
        dimension_px: Option<usize>,
        id: impl fmt::Display,
        inner: Html,
        onclick: Option<OnClickAction>,
        desc: impl fmt::Display,
    ) -> Html {
        if let Some(onclick) = onclick {
            let style = dimension_px.map(|dim| {
                inline_css! {
                    extends_style(&*LINK_BOX),
                    height({dim}px),
                    width({dim}px),
                }
            });
            html! {
                <button
                    style = style.as_ref().unwrap_or(&*LINK_BOX)
                    onclick = onclick
                    title = desc
                >
                    {inner}
                </button>
            }
        } else {
            html! {
                <button
                    id = id
                    style = HIDDEN_LINK_BOX
                    disabled = true
                >
                    {inner}
                </button>
            }
        }
    }

    /// Close button.
    ///
    /// Inline SVG for https://icons.getbootstrap.com/icons/x.
    pub fn close(
        dimension_px: Option<usize>,
        id: impl fmt::Display,
        onclick: Option<OnClickAction>,
        desc: impl fmt::Display,
    ) -> Html {
        raw_render(dimension_px, id, close_img(), onclick, desc)
    }
    fn close_img() -> Html {
        html! {
            <svg
                fill = "currentColor"
                height = "100%"
                viewBox = "0 0 16 16"
                xmlns = "http://www.w3.org/2000/svg"
            >
                <path
                    fill-rule = "evenodd"
                    d = "\
                        M11.854 4.146a.5.5 0 0 1 0 .708l-7 7a.5.5 \
                        0 0 1-.708-.708l7-7a.5.5 0 0 1 .708 0z\
                    "
                />
                <path
                    fill-rule = "evenodd"
                    d = "\
                        M4.146 4.146a.5.5 0 0 0 0 .708l7 \
                        7a.5.5 0 0 0 .708-.708l-7-7a.5.5 0 0 0-.708 0z\
                    "
                />
            </svg>
        }
    }

    /// Collapse button.
    ///
    /// Inline SVG for https://icons.getbootstrap.com/icons/chevron-bar-up.
    pub fn collapse(
        dimension_px: Option<usize>,
        id: impl fmt::Display,
        onclick: Option<OnClickAction>,
        desc: impl fmt::Display,
    ) -> Html {
        raw_render(dimension_px, id, collapse_img(), onclick, desc)
    }
    fn collapse_img() -> Html {
        html! {
            <svg
                fill = "currentColor"
                height = "100%"
                viewBox = "0 0 16 16"
                xmlns = "http://www.w3.org/2000/svg"
            >
                <path
                    fill-rule = "evenodd"
                    d = "\
                        M3.646 11.854a.5.5 0 0 0 .708 0L8 8.207l3.646 3.647a.5.5 0 0 0 \
                        .708-.708l-4-4a.5.5 0 0 0-.708 0l-4 4a.5.5 0 0 0 0 .708zM2.4 5.2c0 \
                        .22.18.4.4.4h10.4a.4.4 0 0 0 0-.8H2.8a.4.4 0 0 0-.4.4z\
                    "
                />
            </svg>
        }
    }

    /// Expand button.
    ///
    /// Inline SVG for https://icons.getbootstrap.com/icons/chevron-bar-down.
    pub fn expand(
        dimension_px: Option<usize>,
        id: impl fmt::Display,
        onclick: Option<OnClickAction>,
        desc: impl fmt::Display,
    ) -> Html {
        raw_render(dimension_px, id, expand_img(), onclick, desc)
    }
    fn expand_img() -> Html {
        html! {
            <svg
                fill = "currentColor"
                height = "100%"
                viewBox = "0 0 16 16"
                xmlns = "http://www.w3.org/2000/svg"
            >
                <path
                    fill-rule = "evenodd"
                    d = "\
                        M3.646 4.146a.5.5 0 0 1 .708 0L8 7.793l3.646-3.647a.5.5 0 0 1 .708.708l-4 \
                        4a.5.5 0 0 1-.708 0l-4-4a.5.5 0 0 1 0-.708zM1 11.5a.5.5 0 0 1 \
                        .5-.5h13a.5.5 0 0 1 0 1h-13a.5.5 0 0 1-.5-.5z\
                    "
                />
            </svg>
        }
    }

    /// Expand button.
    ///
    /// Inline SVG for https://icons.getbootstrap.com/icons/arrow-bar-up.
    pub fn arrow_up(
        dimension_px: Option<usize>,
        id: impl fmt::Display,
        onclick: Option<OnClickAction>,
        desc: impl fmt::Display,
    ) -> Html {
        raw_render(dimension_px, id, arrow_up_img(), onclick, desc)
    }
    fn arrow_up_img() -> Html {
        html! {
            <svg
                fill = "currentColor"
                height = "100%"
                viewBox = "0 0 16 16"
                xmlns = "http://www.w3.org/2000/svg"
            >
                <path
                    fill-rule = "evenodd"
                    d = "\
                        M11.354 5.854a.5.5 0 0 0 0-.708l-3-3a.5.5 0 0 0-.708 0l-3 3a.5.5 0 1 0 \
                        .708.708L8 3.207l2.646 2.647a.5.5 0 0 0 .708 0z\
                    "
                />
                <path
                    fill-rule = "evenodd"
                    d = "\
                        M8 10a.5.5 0 0 0 .5-.5V3a.5.5 0 0 0-1 0v6.5a.5.5 0 0 0 .5.5zm-4.8 \
                        1.6c0-.22.18-.4.4-.4h8.8a.4.4 0 0 1 0 .8H3.6a.4.4 0 0 1-.4-.4z\
                    "
                />
            </svg>
        }
    }

    /// Expand button.
    ///
    /// Inline SVG for https://icons.getbootstrap.com/icons/arrow-bar-down.
    pub fn arrow_down(
        dimension_px: Option<usize>,
        id: impl fmt::Display,
        onclick: Option<OnClickAction>,
        desc: impl fmt::Display,
    ) -> Html {
        raw_render(dimension_px, id, arrow_down_img(), onclick, desc)
    }
    fn arrow_down_img() -> Html {
        html! {
            <svg
                fill = "currentColor"
                height = "100%"
                viewBox = "0 0 16 16"
                xmlns = "http://www.w3.org/2000/svg"
            >
                <path
                    fill-rule = "evenodd"
                        d = "M11.354 10.146a.5.5 0 0 1 0 .708l-3 3a.5.5 0 0 1-.708 \
                        0l-3-3a.5.5 0 0 1 .708-.708L8 12.793l2.646-2.647a.5.5 0 0 1 .708 0z\
                    "
                />
                <path
                    fill-rule = "evenodd"
                    d = "\
                        M8 6a.5.5 0 0 1 .5.5V13a.5.5 0 0 1-1 0V6.5A.5.5 0 0 1 8 6zM2 \
                        3.5a.5.5 0 0 1 .5-.5h11a.5.5 0 0 1 0 1h-11a.5.5 0 0 1-.5-.5z\
                    "
                />
            </svg>
        }
    }

    /// Plus button.
    ///
    /// Inline SVG for https://icons.getbootstrap.com/icons/plus-circle-fill.
    pub fn plus(
        dimension_px: Option<usize>,
        id: impl fmt::Display,
        onclick: Option<OnClickAction>,
        desc: impl fmt::Display,
    ) -> Html {
        raw_render(dimension_px, id, plus_img(), onclick, desc)
    }
    fn plus_img() -> Html {
        html! {
            <svg
                fill = "currentColor"
                height = "100%"
                viewBox = "0 0 16 16"
                xmlns = "http://www.w3.org/2000/svg"
            >
                <path
                    fill-rule = "evenodd"
                    d = "\
                        M8 3.5a.5.5 0 0 1 .5.5v4a.5.5 0 0 1-.5.5H4a.5.5 0 0 1 0-1h3.5V4a.5.5 0 0 1 \
                        .5-.5z\
                    "
                />
                <path
                    fill-rule = "evenodd"
                    d = "M7.5 8a.5.5 0 0 1 .5-.5h4a.5.5 0 0 1 0 1H8.5V12a.5.5 0 0 1-1 0V8z"
                />
                <path
                    fill-rule = "evenodd"
                    d = "M8 15A7 7 0 1 0 8 1a7 7 0 0 0 0 14zm0 1A8 8 0 1 0 8 0a8 8 0 0 0 0 16z"
                />
            </svg>
        }
    }

    /// Minus button.
    ///
    /// Inline SVG for https://icons.getbootstrap.com/icons/dash-circle.
    pub fn minus(
        dimension_px: Option<usize>,
        id: impl fmt::Display,
        onclick: Option<OnClickAction>,
        desc: impl fmt::Display,
    ) -> Html {
        raw_render(dimension_px, id, minus_img(), onclick, desc)
    }
    fn minus_img() -> Html {
        html! {
            <svg
                fill = "currentColor"
                height = "100%"
                viewBox = "0 0 16 16"
                xmlns = "http://www.w3.org/2000/svg"
            >
                <path
                    fill-rule = "evenodd"
                    d = "M8 15A7 7 0 1 0 8 1a7 7 0 0 0 0 14zm0 1A8 8 0 1 0 8 0a8 8 0 0 0 0 16z"
                />
                <path
                    fill-rule = "evenodd"
                    d = "M3.5 8a.5.5 0 0 1 .5-.5h8a.5.5 0 0 1 0 1H4a.5.5 0 0 1-.5-.5z"
                />
            </svg>
        }
    }

    /// Undo button.
    ///
    /// Inline SVG for https://icons.getbootstrap.com/icons/arrow-counterclockwise.
    pub fn undo(
        dimension_px: Option<usize>,
        id: impl fmt::Display,
        onclick: Option<OnClickAction>,
        desc: impl fmt::Display,
    ) -> Html {
        raw_render(dimension_px, id, undo_img(), onclick, desc)
    }
    fn undo_img() -> Html {
        html! {
            <svg
                fill = "currentColor"
                height = "100%"
                viewBox = "0 0 16 16"
                xmlns = "http://www.w3.org/2000/svg"
            >
                <path
                    fill-rule = "evenodd"
                    d = "\
                        M12.83 6.706a5 5 0 0 0-7.103-3.16.5.5 0 1 1-.454-.892A6 6 0 1 1 2.545 \
                        5.5a.5.5 0 1 1 .91.417 5 5 0 1 0 9.375.789z\
                    "
                />
                <path
                    fill-rule = "evenodd"
                    d = "\
                        M7.854.146a.5.5 0 0 0-.708 0l-2.5 2.5a.5.5 0 0 0 0 .708l2.5 2.5a.5.5 0 1 0 \
                        .708-.708L5.707 3 7.854.854a.5.5 0 0 0 0-.708z\
                    "
                />
            </svg>
        }
    }

    /// Check button.
    ///
    /// Inline SVG for https://icons.getbootstrap.com/icons/check-circle.
    pub fn check(
        dimension_px: Option<usize>,
        id: impl fmt::Display,
        onclick: Option<OnClickAction>,
        desc: impl fmt::Display,
    ) -> Html {
        raw_render(dimension_px, id, check_img(), onclick, desc)
    }
    fn check_img() -> Html {
        html! {
            <svg
                fill = "currentColor"
                height = "100%"
                viewBox = "0 0 16 16"
                xmlns = "http://www.w3.org/2000/svg"
            >
                <path
                    fill-rule = "evenodd"
                    d = "M8 15A7 7 0 1 0 8 1a7 7 0 0 0 0 14zm0 1A8 8 0 1 0 8 0a8 8 0 0 0 0 16z"
                />
                <path
                    fill-rule = "evenodd"
                    d = "\
                        M10.97 4.97a.75.75 0 0 1 1.071 1.05l-3.992 4.99a.75.75 0 0 1-1.08.02L4.324 \
                        8.384a.75.75 0 1 1 1.06-1.06l2.094 2.093 3.473-4.425a.236.236 0 0 1 \
                        .02-.022z\
                    "
                />
            </svg>
        }
    }
}
