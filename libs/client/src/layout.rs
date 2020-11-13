/*<LICENSE>
    This file is part of Memthol.

    Copyright (C) 2020 OCamlPro.

    Memthol is free software: you can redistribute it and/or modify
    it under the terms of the GNU General Public License as published by
    the Free Software Foundation, either version 3 of the License, or
    (at your option) any later version.

    Memthol is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
    GNU General Public License for more details.

    You should have received a copy of the GNU General Public License
    along with Memthol.  If not, see <https://www.gnu.org/licenses/>.
*/

//! HTML builders.

#![allow(non_upper_case_globals)]

prelude! {}

pub mod button;
pub mod chart;
pub mod footer;
pub mod header;
pub mod input;
pub mod progress;
pub mod table;
pub mod tabs;

/// Dark grey background.
pub const DARK_GREY_BG: &'static str = "#313131";
/// Light blue foreground.
pub const LIGHT_BLUE_FG: &'static str = "#8dedff";

define_style! {
    SECTION_STYLE = {
        font_size(150%),
        text_align(center),
        bold,
        underline,
    };
}

/// Displays a section title.
pub fn section_title(txt: &str) -> Html {
    html! {
        <div
            style = SECTION_STYLE
        >
            {txt}
        </div>
    }
}

/// Renders the full model.
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
    }

    let body_style = inline_css! {
        extends(body_style),
        padding(
            {model.header.height_px(model) + 30}px,
            0%,
            {model.footer.height_px()}px,
            0%,
        ),
    };

    html! {
        <>
            { model.header.render(model) }
            <div
                style = body_style
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
