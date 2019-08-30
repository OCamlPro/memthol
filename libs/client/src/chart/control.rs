//! Control part of the charts. Lives in the top-bar.

use crate::base::*;

use msg::ControlMsg;

/// Control part of the charts.
pub struct Control {
    /// True if the control menu is expanded.
    visible: bool,
}
impl Control {
    /// Constructor.
    pub fn new() -> Self {
        Self { visible: false }
    }
}

/// ## Rendering.
impl Control {
    /// Renders itself.
    pub fn render(&self) -> Html {
        use cst::class::top_tab::*;
        html! {
            <center>
                <div class="control_title">
                    <a
                        class={ if self.visible { ACTIVE } else { INACTIVE } }
                        onclick=|_| ControlMsg::toggle_visible()
                    >
                        { "Control Menu" }
                    </a>
                </div>
            </center>
        }
    }

    /// Renders a default title when the control menu should not be displayed.
    pub fn render_default(&self) -> Html {
        html! {
            <center>
                <div class="control_title">
                    <h2> { "Memthol" } </h2>
                </div>
            </center>
        }
    }
}

/// ## Actions.
impl Control {
    /// Handles a message.
    pub fn update(&mut self, msg: msg::ControlMsg) -> ShouldRender {
        use ControlMsg::*;
        match msg {
            ToggleVisible => {
                self.visible = !self.visible;
                true
            }
        }
    }
}
