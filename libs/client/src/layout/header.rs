//! Header rendering stuff.

prelude! {}

pub const HEADER_HEIGHT_PX: usize = 60;

pub fn render(model: &Model) -> Html {
    define_style! {
        HEADER_STYLE = {
            font(default),
            width(100%),
            height({HEADER_HEIGHT_PX}px),

            fg({layout::LIGHT_BLUE_FG}),
            bg({layout::DARK_GREY_BG}),

            border_radius(0 px, 0 px, 20 px, 20 px),

            fixed(top),

            z_index(600),
        };

        CONTENT_STYLE = {
            margin(auto),
        };
    }

    html! {
        <header
            style = HEADER_STYLE
        >
            <div
                style = CONTENT_STYLE
            >
                {format!("memthol by OCamlPro, stats is some: {}", model.alloc_stats.is_some())}
            </div>
        </header>
    }
}
