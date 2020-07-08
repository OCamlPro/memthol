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
            gradient_top: "white",
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
            gradient_top: "white",
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
            gradient_top: "white",
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
            width(100%),
            table,
            bg(gradient {self.gradient_top} to {self.gradient_bot}),
            border({self.stroke_px} px, {self.border_color}),
            border_radius(
                {radius_px(self.tl_rounded)} px,
                {radius_px(self.tr_rounded)} px,
                {radius_px(self.br_rounded)} px,
                {radius_px(self.bl_rounded)} px,
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
            width(100%),
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

    fn centered_text(txt: impl fmt::Display, dimmed: bool) -> Html {
        html! {
            <a
                style = if dimmed { &*DIMMED_CONTENT_STYLE } else { &*CONTENT_STYLE }
            >
                {txt}
            </a>
        }
    }
    fn top_centered_text(txt: impl fmt::Display, dimmed: bool) -> Html {
        html! {
            <a
                style = if dimmed { &*TOP_DIMMED_CONTENT_STYLE } else { &*TOP_CONTENT_STYLE }
            >
                {txt}
            </a>
        }
    }

    pub fn render_default_button(
        id: &str,
        txt: impl fmt::Display,
        onclick: Option<OnClickAction>,
        dimmed: bool,
    ) -> Html {
        render(Some(*DEFAULT_BUTTON_BOX_PROPS), id, txt, onclick, dimmed)
    }

    pub fn render(
        props: Option<BoxProps<'_>>,
        id: &str,
        txt: impl fmt::Display,
        onclick: Option<OnClickAction>,
        dimmed: bool,
    ) -> Html {
        let mut inner = if props.as_ref().map(|props| props.top).unwrap_or(false) {
            top_centered_text(txt, dimmed)
        } else {
            centered_text(txt, dimmed)
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
            bg(transparent),
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
    }
    impl Img {
        pub fn render(
            self,
            id: &str,
            onclick: Option<OnClickAction>,
            desc: impl fmt::Display,
        ) -> Html {
            match self {
                Self::Close => close(id, onclick, desc),
                Self::Collapse => collapse(id, onclick, desc),
                Self::Expand => expand(id, onclick, desc),
                Self::ArrowUp => arrow_up(id, onclick, desc),
                Self::ArrowDown => arrow_down(id, onclick, desc),
                Self::Plus => plus(id, onclick, desc),
            }
        }
    }

    /// Internal rendering function.
    ///
    /// If `onclick.is_none()`, the button will be hidden and deactivated.
    fn raw_render(
        id: &str,
        inner: Html,
        onclick: Option<OnClickAction>,
        desc: impl fmt::Display,
    ) -> Html {
        if let Some(onclick) = onclick {
            html! {
                <button
                    style = LINK_BOX
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
    pub fn close(id: &str, onclick: Option<OnClickAction>, desc: impl fmt::Display) -> Html {
        raw_render(id, close_img(), onclick, desc)
    }
    fn close_img() -> Html {
        html! {
            <svg
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
    pub fn collapse(id: &str, onclick: Option<OnClickAction>, desc: impl fmt::Display) -> Html {
        raw_render(id, collapse_img(), onclick, desc)
    }
    fn collapse_img() -> Html {
        html! {
            <svg
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
    pub fn expand(id: &str, onclick: Option<OnClickAction>, desc: impl fmt::Display) -> Html {
        raw_render(id, expand_img(), onclick, desc)
    }
    fn expand_img() -> Html {
        html! {
            <svg
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
    pub fn arrow_up(id: &str, onclick: Option<OnClickAction>, desc: impl fmt::Display) -> Html {
        raw_render(id, arrow_up_img(), onclick, desc)
    }
    fn arrow_up_img() -> Html {
        html! {
            <svg
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
    pub fn arrow_down(id: &str, onclick: Option<OnClickAction>, desc: impl fmt::Display) -> Html {
        raw_render(id, arrow_down_img(), onclick, desc)
    }
    fn arrow_down_img() -> Html {
        html! {
            <svg
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
    pub fn plus(id: &str, onclick: Option<OnClickAction>, desc: impl fmt::Display) -> Html {
        raw_render(id, plus_img(), onclick, desc)
    }
    fn plus_img() -> Html {
        html! {
            <svg
                height = "100%"
                viewBox = "0 0 16 16"
                xmlns = "http://www.w3.org/2000/svg"
            >
                <path
                    fill-rule = "evenodd"
                    d = "\
                        M16 8A8 8 0 1 1 0 8a8 8 0 0 1 16 0zM8.5 4a.5.5 0 0 0-1 0v3.5H4a.5.5 0 0 0 \
                        0 1h3.5V12a.5.5 0 0 0 1 0V8.5H12a.5.5 0 0 0 0-1H8.5V4z\
                    "
                />
            </svg>
        }
    }
}