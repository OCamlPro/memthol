//! HTML builders.

#![allow(non_upper_case_globals)]

prelude! {}

pub mod button;
pub mod chart;
pub mod foot;
pub mod header;
pub mod input;
pub mod progress;
pub mod table;
pub mod tabs;

pub const DARK_GREY_BG: &'static str = "#313131";
pub const LIGHT_BLUE_FG: &'static str = "#8dedff";

define_style! {
    SECTION_STYLE = {
        font_size(150%),
        text_align(center),
        bold,
        underline,
    };
}

pub fn section(txt: &str) -> Html {
    html! {
        <div
            style = SECTION_STYLE
        >
            {txt}
        </div>
    }
}

pub fn render(model: &Model) -> Html {
    define_style! {
        body_style! = {
            font(default),
            block,
            margin(0 px),
            height(min 100 vh),
            bg(white),
            fg(black),
            border_radius(20 px, 20 px, 0 px, 0 px),
        };

        COLLAPSED_FOOTER_BODY_STYLE = {
            extends(body_style),
            padding(
                {header::HEADER_HEIGHT_PX + 30}px,
                0%,
                {foot::collapsed_height_px}px,
                0%,
            ),
        };
        EXPANDED_FOOTER_BODY_STYLE = {
            extends(body_style),
            padding(
                {header::HEADER_HEIGHT_PX + 30}px,
                0%,
                {foot::expanded_height_px}px,
                0%,
            ),
        };
    }

    html! {
        <>
            { header::render(model) }
            <div
                style = if model.footer.is_expanded() {
                    &*EXPANDED_FOOTER_BODY_STYLE
                } else {
                    &*COLLAPSED_FOOTER_BODY_STYLE
                }
            >
                {
                    if let Some(load_info) = model.progress.as_ref() {
                        progress::render(load_info)
                    } else {
                        model.charts.render(model)
                    }
                }
            </div>
            { model.footer.render(model) }
        </>
    }
}
