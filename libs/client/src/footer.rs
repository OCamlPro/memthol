//! Footer DOM element.

use crate::base::*;

pub struct Footer {
    /// Active footer tab, if any.
    active: Option<FooterTab>,
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
                info!("footer: toggle {:?}", tab);
                info!("        active {:?}", self.active);
                if self.active == Some(tab) {
                    self.active = None
                } else {
                    self.active = Some(tab)
                }
                Ok(true)
            }
        }
    }

    /// Renders itself.
    pub fn render(&self, filters: &filter::Filters) -> Html {
        html! {
            <footer id=style::id::FOOTER>
                <ul id=style::id::FOOTER_TABS class=style::class::tabs::UL>
                    { for NormalFooterTab::all()
                        .into_iter()
                        .map(|tab|
                            tab.render(Some(tab) == self.active.and_then(FooterTab::get_normal_tab))
                        )
                    }
                    { filters.render_tabs(self.active.and_then(FooterTab::get_filter)) }
                </ul>
                {
                    match self.active {
                        None => html!(<a/>),
                        Some(FooterTab::Filter(active)) => html! {
                            <div class=style::class::footer::DISPLAY>
                                { filters.render_filter(active) }
                            </div>
                        },
                        Some(FooterTab::Normal(NormalFooterTab::Info)) => html! {
                            <div class=style::class::footer::DISPLAY>
                                { "Info footer tab is not implemented." }
                            </div>
                        },
                    }
                }
            </footer>
        }
    }
}

/// Normal footer tab, *e.g.* not a filter tab.
#[derive(Debug, Clone, Copy, PartialOrd, Ord, PartialEq, Eq, strum_macros::EnumIter)]
pub enum NormalFooterTab {
    /// Statistics tab.
    Info,
}

impl NormalFooterTab {
    /// Renders a tab.
    pub fn render(&self, active: bool) -> Html {
        let tab = *self;
        html! {
            <li class={ style::class::tabs::li::get(true) }>
                <a
                    class={ style::class::tabs::get(active) }
                    onclick=|_| msg::FooterMsg::toggle_tab(tab.into())
                > {
                    self.to_string()
                } </a>
            </li>
        }
    }

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
    Filter(Option<filter::FilterUid>),
}

impl FooterTab {
    /// Filter tab constructor.
    pub fn filter(uid: Option<filter::FilterUid>) -> Self {
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
    pub fn get_filter(self) -> Option<Option<filter::FilterUid>> {
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
            FooterTab::Filter(uid) => {
                write!(fmt, "Filter(")?;
                match uid {
                    None => write!(fmt, "catch-all")?,
                    Some(uid) => write!(fmt, "#{}", uid)?,
                }
                write!(fmt, ")")
            }
        }
    }
}

impl From<NormalFooterTab> for FooterTab {
    fn from(tab: NormalFooterTab) -> Self {
        FooterTab::Normal(tab)
    }
}
