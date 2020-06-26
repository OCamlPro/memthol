//! Button rendering.

prelude! {}

#[derive(Clone, Copy)]
pub struct BoxProps<'color> {
    radius_px: u8,
    stroke_px: u8,
    tl_rounded: bool,
    tr_rounded: bool,
    bl_rounded: bool,
    br_rounded: bool,
    border_color: &'color str,
    gradient_top: &'color str,
    gradient_bot: &'color str,
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
            gradient_top: "white",
            gradient_bot: "black",
        }
    }

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

    pub const fn with_radius_px(mut self, radius_px: u8) -> Self {
        self.radius_px = radius_px;
        self
    }
    pub const fn with_stroke_px(mut self, stroke_px: u8) -> Self {
        self.stroke_px = stroke_px;
        self
    }

    pub const fn with_gradient_top(mut self, color: &'color str) -> Self {
        self.gradient_top = color;
        self
    }
    pub const fn with_gradient_bot(mut self, color: &'color str) -> Self {
        self.gradient_bot = color;
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
            z_index(800),
        };

        CONTENT_STYLE = {
            extends(content_style),
            fg(white),
        };
        DIMMED_CONTENT_STYLE = {
            extends(content_style),
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

    pub fn render_default_button(
        id: &str,
        txt: impl fmt::Display,
        onclick: Option<OnClickAction>,
        dimmed: bool,
    ) -> Html {
        nu_render(Some(*DEFAULT_BUTTON_BOX_PROPS), id, txt, onclick, dimmed)
    }

    pub fn nu_render(
        props: Option<BoxProps<'_>>,
        id: &str,
        txt: impl fmt::Display,
        onclick: Option<OnClickAction>,
        dimmed: bool,
    ) -> Html {
        let mut inner = centered_text(txt, dimmed);
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

    pub fn render(
        id: &str,
        txt: impl fmt::Display,
        onclick: Option<OnClickAction>,
        dimmed: bool,
    ) -> Html {
        let inner = centered_text(txt, dimmed);
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

    // pub fn render_normal(id: &str, txt: impl fmt::Display, onclick: Option<OnClickAction>) -> Html {
    //     render(id, txt, onclick, false)
    // }
    // pub fn render_dimmed(id: &str, txt: impl fmt::Display, onclick: Option<OnClickAction>) -> Html {
    //     render(id, txt, onclick, false)
    // }

    // define_style! {
    //     link_box! = {
    //         width(70%),
    //         height(10%),
    //         bg(gradient {"#8e8e8e"} to black),
    //         border_radius(5 px),
    //         border(1 px, white),
    //         margin(auto),
    //         table,
    //     };
    //     LINK_BOX = {
    //         extends(link_box),
    //         pointer,
    //     };
    //     HIDDEN_LINK_BOX = {
    //         extends(link_box),
    //         visi(hidden),
    //     };
    // }

    // pub fn render_boxed(
    //     id: &str,
    //     txt: impl fmt::Display,
    //     onclick: Option<OnClickAction>,
    //     dimmed: bool,
    // ) -> Html {
    // }
}
