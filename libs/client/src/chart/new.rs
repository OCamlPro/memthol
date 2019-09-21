//! Part of the BUI that handles chart creation.

use crate::base::{
    chart::axis::{XAxis, YAxis},
    *,
};

pub struct NewChart {
    /// X-axis selection.
    x_axis: XAxis,
    /// Y-axis selection.
    y_axis: YAxis,
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
    pub fn render(&self) -> Html {
        let (x_axis, y_axis) = (self.x_axis, self.y_axis);
        html! {
            <g>
                <center class=style::class::chart::HEADER>
                    { buttons::add(move |_| msg::to_server::ChartsMsg::new(x_axis, y_axis).into()) }

                    <h2>
                        <Select<XAxis>
                            selected = Some(x_axis)
                            options = XAxis::all()
                            onchange = msg::ChartsMsg::new_chart_set_x
                        />
                        { "    /    " }
                        <Select<YAxis>
                            selected = Some(y_axis)
                            options = x_axis.y_axes()
                            onchange = msg::ChartsMsg::new_chart_set_y
                        />
                    </h2>
                </center>
            </g>
        }
    }
}