//! Footer DOM element.

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
            } // Removed(uid) => {
              //     if self.active == Some(FooterTab::Filter(uid::Line::Filter(uid))) {
              //         self.active = Some(FooterTab::Filter(uid::Line::Everything));
              //         Ok(true)
              //     } else {
              //         Ok(false)
              //     }
              // }
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

/// Normal footer tab, *e.g.* not a filter tab.
#[derive(Debug, Clone, Copy, PartialOrd, Ord, PartialEq, Eq, strum_macros::EnumIter)]
pub enum NormalFooterTab {
    /// Statistics tab.
    Info,
}

impl NormalFooterTab {
    /// Lists all tabs.
    pub fn all() -> Vec<NormalFooterTab> {
        use strum::IntoEnumIterator;
        Self::iter().collect()
    }
}

impl fmt::Display for NormalFooterTab {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Info => write!(fmt, "Info"),
        }
    }
}

/// Footer tabs.
#[derive(Debug, Clone, Copy, PartialOrd, Ord, PartialEq, Eq)]
pub enum FooterTab {
    /// Info tab.
    Normal(NormalFooterTab),
    /// Filters tab.
    Filter(uid::Line),
}

impl FooterTab {
    /// Filter tab constructor.
    pub fn filter(uid: uid::Line) -> Self {
        Self::Filter(uid)
    }

    /// The normal tab, if any.
    pub fn get_normal_tab(self) -> Option<NormalFooterTab> {
        match self {
            Self::Normal(tab) => Some(tab),
            Self::Filter(_) => None,
        }
    }
    /// The active filter, if any.
    pub fn get_filter(self) -> Option<uid::Line> {
        match self {
            Self::Normal(_) => None,
            Self::Filter(uid) => Some(uid),
        }
    }
}

impl fmt::Display for FooterTab {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match self {
            FooterTab::Normal(tab) => write!(fmt, "{}", tab),
            FooterTab::Filter(uid) => write!(fmt, "Filter({})", uid),
        }
    }
}

impl From<NormalFooterTab> for FooterTab {
    fn from(tab: NormalFooterTab) -> Self {
        FooterTab::Normal(tab)
    }
}
impl From<uid::Line> for FooterTab {
    fn from(uid: uid::Line) -> Self {
        FooterTab::Filter(uid)
    }
}
