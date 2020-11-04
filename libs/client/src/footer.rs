//! Footer state.

prelude! {}

/// Footer state.
pub struct Footer {
    /// Active footer tab, if any.
    pub active: Option<FooterTab>,
}

impl Footer {
    /// Constructor.
    pub fn new() -> Self {
        Self { active: None }
    }

    /// Applies a footer action.
    pub fn update(&mut self, msg: msg::FooterMsg) -> Res<ShouldRender> {
        use msg::FooterMsg::*;
        match msg {
            ToggleTab(tab) => {
                if self.active == Some(tab) {
                    self.active = None
                } else {
                    self.active = Some(tab)
                }
                Ok(true)
            }
        }
    }

    /// True if the footer is expanded.
    pub fn is_expanded(&self) -> bool {
        self.active.is_some()
    }

    /// Renders itself.
    pub fn render(&self, model: &Model) -> Html {
        layout::foot::render(self, model)
    }
}

/// Footer tabs.
#[derive(Debug, Clone, Copy, PartialOrd, Ord, PartialEq, Eq)]
pub enum FooterTab {
    /// Filters tab.
    Filter(uid::Line),
}

impl FooterTab {
    /// Filter tab constructor.
    pub fn filter(uid: uid::Line) -> Self {
        Self::Filter(uid)
    }

    /// The active filter, if any.
    pub fn get_filter(self) -> Option<uid::Line> {
        match self {
            Self::Filter(uid) => Some(uid),
        }
    }
}

impl fmt::Display for FooterTab {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match self {
            FooterTab::Filter(uid) => write!(fmt, "Filter({})", uid),
        }
    }
}

impl From<uid::Line> for FooterTab {
    fn from(uid: uid::Line) -> Self {
        FooterTab::Filter(uid)
    }
}
