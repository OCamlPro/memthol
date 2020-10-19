//! Chart settings.

prelude! {}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
pub enum DisplayMode {
    Normal,
    StackedArea,
    StackedAreaPercent,
}
impl DisplayMode {
    pub fn desc(self) -> &'static str {
        match self {
            Self::Normal => "normal",
            Self::StackedArea => "stacked area",
            Self::StackedAreaPercent => "stacked area (%)",
        }
    }

    pub fn is_normal(self) -> bool {
        match self {
            Self::Normal => true,
            Self::StackedArea | Self::StackedAreaPercent => false,
        }
    }
    pub fn is_stacked_area(self) -> bool {
        match self {
            Self::Normal => false,
            Self::StackedArea | Self::StackedAreaPercent => true,
        }
    }

    pub fn all() -> Vec<Self> {
        vec![Self::Normal, Self::StackedArea, Self::StackedAreaPercent]
    }

    pub fn to_uname(self) -> &'static str {
        match self {
            Self::Normal => "normal",
            Self::StackedArea => "stacked_area",
            Self::StackedAreaPercent => "stacked_area_percent",
        }
    }
    pub fn from_uname(uname: &'static str) -> Option<Self> {
        Some(match uname {
            "normal" => Self::Normal,
            "stacked_area" => Self::StackedArea,
            "stacked_area_percent" => Self::StackedAreaPercent,
            _ => return None,
        })
    }
}
impl fmt::Display for DisplayMode {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        self.desc().fmt(fmt)
    }
}

/// Resolution.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Resolution {
    /// Width.
    pub width: u32,
    /// Height.
    pub height: u32,
}
impl From<(u32, u32)> for Resolution {
    fn from((width, height): (u32, u32)) -> Self {
        Self { width, height }
    }
}
impl fmt::Display for Resolution {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{}x{}", self.width, self.height)
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ChartSettings {
    title: String,
    display_mode: DisplayMode,
    can_stacked_area: bool,
    visible: bool,
    x_log: bool,
    y_log: bool,
    resolution: Option<Resolution>,
}
impl ChartSettings {
    /// Constructor.
    pub fn new(title: impl Into<String>, can_stacked_area: bool) -> Self {
        Self {
            title: title.into(),
            display_mode: DisplayMode::Normal,
            can_stacked_area,
            visible: true,
            x_log: false,
            y_log: false,
            resolution: None,
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

    /// Applies an update.
    fn inner_update(&mut self, msg: msg::ChartSettingsMsg) -> bool {
        use msg::ChartSettingsMsg::*;
        match msg {
            ToggleVisible => {
                self.toggle_visible();
                false
            }
            ChangeTitle(title) => {
                self.set_title(title);
                false
            }
            SetDisplayMode(mode) => {
                self.set_display_mode(mode);
                false
            }
            SetResolution(resolution) => {
                self.set_resolution(resolution);
                true
            }
        }
    }

    /// Applies an update.
    #[cfg(any(test, feature = "server"))]
    pub fn update(&mut self, msg: msg::ChartSettingsMsg) -> bool {
        self.inner_update(msg)
    }

    /// Applies an update.
    #[cfg(not(any(test, feature = "server")))]
    pub fn update(&mut self, msg: msg::ChartSettingsMsg) {
        self.inner_update(msg);
    }

    /// Title accessor.
    pub fn title(&self) -> &str {
        &self.title
    }
    /// True if the chart is visible.
    pub fn is_visible(&self) -> bool {
        self.visible
    }
    /// Stacked-area setting accessor.
    pub fn display_mode(&self) -> DisplayMode {
        self.display_mode
    }
    /// True if the chart can render itself in stacked-area mode.
    pub fn can_stacked_area(&self) -> bool {
        self.can_stacked_area
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

    /// Makes the chart visible or not.
    pub fn set_visible(&mut self, is_visible: bool) {
        self.visible = is_visible
    }
    /// Toggles the charts visibility.
    pub fn toggle_visible(&mut self) {
        self.visible = !self.visible
    }

    /// Sets the display setting.
    pub fn set_display_mode(&mut self, setting: DisplayMode) {
        if self.can_stacked_area || !setting.is_stacked_area() {
            self.display_mode = setting
        }
    }
    /// List of legal display modes for this chart.
    ///
    /// None if the chart supports only one display mode.
    pub fn legal_display_modes(&self) -> Option<Vec<DisplayMode>> {
        if !self.can_stacked_area {
            None
        } else {
            Some(DisplayMode::all())
        }
    }

    /// Sets the resolution of the chart.
    pub fn set_resolution(&mut self, resolution: Resolution) {
        self.resolution = Some(resolution);
    }
    /// Retrieves the resolution of the chart, if one was set.
    pub fn resolution(&self) -> Option<Resolution> {
        self.resolution
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
