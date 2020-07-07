//! HTML builders.

#![allow(non_upper_case_globals)]

prelude! {}

pub mod button;
pub mod chart;
pub mod foot;
pub mod tabs;

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
            padding(2%, 0%, {foot::collapsed_height_px}px, 0%),
        };
        EXPANDED_FOOTER_BODY_STYLE = {
            extends(body_style),
            padding(2%, 0%, {foot::expanded_height_px}px, 0%),
        };
    }

    html! {
        <>
            <div
                style = if model.footer.is_expanded() {
                    &*EXPANDED_FOOTER_BODY_STYLE
                } else {
                    &*COLLAPSED_FOOTER_BODY_STYLE
                }
            >
                { model.charts.render(model) }
            </div>
            { model.footer.render(model) }
        </>
    }
}
