//! Chart settings.

prelude! {}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ChartSettings {
    title: String,
    stacked_area: Option<bool>,
    x_log: bool,
    y_log: bool,
}
impl ChartSettings {
    /// Constructor.
    pub fn new(title: impl Into<String>, can_stacked_area: bool) -> Self {
        Self {
            title: title.into(),
            stacked_area: if can_stacked_area { Some(false) } else { None },
            x_log: false,
            y_log: false,
        }
    }

    /// Constructor from a pair of axes.
    pub fn from_axes(
        title: impl Into<String>,
        _x: chart::axis::XAxis,
        y: chart::axis::YAxis,
    ) -> Self {
        Self::new(title, y.can_stack_area())
    }

    /// Title accessor.
    pub fn title(&self) -> &str {
        &self.title
    }
    /// Stacked-area setting accessor.
    pub fn stacked_area(&self) -> Option<bool> {
        self.stacked_area
    }
    /// X-axis log setting accessor.
    pub fn x_log(&self) -> bool {
        self.x_log
    }
    /// Y-axis log setting accessor.
    pub fn y_log(&self) -> bool {
        self.y_log
    }

    /// Sets the title.
    pub fn set_title(&mut self, title: impl Into<String>) {
        self.title = title.into()
    }
    /// Sets the stacked-area setting.
    ///
    /// Does nothing if stacked-area is not allowed.
    pub fn set_stacked_area(&mut self, stacked_area: bool) {
        if let Some(val) = self.stacked_area.as_mut() {
            *val = stacked_area
        }
    }
    /// Toggles the value of the stacked-area setting.
    ///
    /// Does nothing if stacked-area is not allowed.
    pub fn toggle_stacked_area(&mut self) {
        if let Some(val) = self.stacked_area.as_mut() {
            *val = !*val
        }
    }
    /// Sets the x-axis-log setting.
    pub fn set_x_log(&mut self, x_log: bool) {
        self.x_log = x_log
    }
    /// Sets the y-axis-log setting.
    pub fn set_y_log(&mut self, y_log: bool) {
        self.y_log = y_log
    }
}
