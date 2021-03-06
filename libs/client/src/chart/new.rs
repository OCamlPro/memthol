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

//! Part of the BUI that handles chart creation.

prelude! {}

use chart::axis::{XAxis, YAxis};

/// Representation of a chart before it is constructed.
///
/// Used when the user specifies a new chart before clicking "create".
pub struct NewChart {
    /// X-axis selection.
    pub x_axis: XAxis,
    /// Y-axis selection.
    pub y_axis: YAxis,
}

impl NewChart {
    /// Returns the default y-axis corresponding to an x-axis.
    ///
    /// If `hint = Some(y)`, then this function will return `y` iff `x_axis` is compatible with `y`.
    pub fn default_y_axis_for(x_axis: XAxis, hint: Option<YAxis>) -> Res<YAxis> {
        let y_axes = x_axis.y_axes();
        if let Some(y) = hint {
            if y_axes.iter().any(|y_axis| y_axis == &y) {
                return Ok(y);
            }
        }
        let y_axis = y_axes
            .into_iter()
            .next()
            .ok_or_else(|| format!("{} axis is not compatible with any y-axis", x_axis.desc()))?;
        Ok(y_axis)
    }

    /// Constructor with default selection.
    pub fn new() -> Self {
        let x_axis = XAxis::default();
        let y_axis =
            Self::default_y_axis_for(x_axis, None).expect("cannot construct new chart DOM element");
        Self { x_axis, y_axis }
    }

    /// Sets the x-axis.
    pub fn set_x_axis(&mut self, x_axis: XAxis) -> Res<ShouldRender> {
        let y_axis = Self::default_y_axis_for(x_axis, Some(self.y_axis))
            .chain_err(|| format!("cannot set x-axis to {} axis", x_axis.desc()))?;
        self.x_axis = x_axis;
        self.y_axis = y_axis;
        Ok(true)
    }

    /// Sets the y-axis.
    pub fn set_y_axis(&mut self, y_axis: YAxis) -> Res<ShouldRender> {
        self.y_axis = y_axis;
        Ok(true)
    }

    /// Renders itself.
    pub fn render(&self, model: &Model) -> Html {
        define_style! {
            HEADER_STYLE = {
                font_size(120%),
            };
            CREATE_STYLE = {
                pointer,
                underline,
            };
        }

        let (x_axis, y_axis) = (self.x_axis, self.y_axis);

        html! {
            <center class="chart_header">
                <h2>
                    <div
                        style = CREATE_STYLE
                        onclick = model.link.callback(
                            move |_| msg::to_server::ChartsMsg::new(x_axis, y_axis)
                        )
                    >
                        {"create chart"}
                    </div>
                    <Select<XAxis>
                        selected = Some(x_axis)
                        options = XAxis::all()
                        on_change = model.link.callback(msg::ChartsMsg::new_chart_set_x)
                    />
                    { "    /    " }
                    <Select<YAxis>
                        selected = Some(y_axis)
                        options = x_axis.y_axes()
                        on_change = model.link.callback(msg::ChartsMsg::new_chart_set_y)
                    />
                </h2>
            </center>
        }
    }
}
