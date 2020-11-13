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

//! Progress-bar rendering.
//!
//! This is used when the server is still parsing the dumps.

prelude! {}

/// Renders the progress bar.
///
/// This is used when the server is still parsing the dumps.
pub fn render(info: &LoadInfo) -> Html {
    define_style! {
        BIG = {
            font_size(180%),
        };
        PROGRESS = {
            width(70%),
        };
    }

    let percent = info.percent();

    html! {
        <center
            style = BIG
        >
            <br/>
            <div>
                {"Please wait, memthol is loading..."}
            </div>
            <br/>
            <div>
                {format!(
                    "{} / {}",
                    info.loaded, info.total,
                )}
            </div>
            <br/>
            <progress
                value = percent
                max = 100
                style = PROGRESS
            >
                { format!("{}%", percent) }
            </progress>
        </center>
    }
}
