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

    /// True if the filter tab is active.
    pub fn is_filter_tab_active(&self) -> bool {
        self.active == Some(FooterTab::Filters)
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

    /// Renders itself.
    pub fn render(&self, filters: &filter::Filters) -> Html {
        html! {
            <footer><div id=style::id::FOOTER>
                <ul id=style::id::FOOTER_TABS class=style::class::tabs::UL>
                    { for FooterTab::all()
                        .into_iter()
                        .map(|tab| tab.render(Some(tab) == self.active))
                    }
                    {
                        if self.is_filter_tab_active() {
                            filters.render_tabs()
                        } else {
                            html!(<a/>)
                        }
                    }
                </ul>
                {
                    match self.active {
                        None => html!(<a/>),
                        Some(FooterTab::Filters) => html! {
                            <div class=style::class::footer::DISPLAY>
                                { filters.render_filter() }
                            </div>
                        },
                        Some(FooterTab::Info) => html! {
                            <div class=style::class::footer::DISPLAY>
                                { "Info footer tab is not implemented." }
                            </div>
                        },
                    }
                }
            </div></footer>
        }
    }
}

/// Footer tabs.
#[derive(Debug, Clone, Copy, PartialOrd, Ord, PartialEq, Eq, strum_macros::EnumIter)]
pub enum FooterTab {
    /// Statistics tab.
    Info,
    /// Filters tab.
    Filters,
}
impl FooterTab {
    /// Applies a function to all tabs.
    pub fn map_all<F>(mut f: F)
    where
        F: FnMut(FooterTab),
    {
        f(Self::Info);
        f(Self::Filters);
    }

    /// Renders a tab.
    pub fn render(&self, active: bool) -> Html {
        let tab = *self;
        html! {
            <li class={ style::class::tabs::li::get(true) }>
                <a
                    class={ style::class::tabs::get(active) }
                    onclick=|_| msg::FooterMsg::toggle_tab(tab)
                > {
                    self.to_string()
                } </a>
            </li>
        }
    }

    /// A list of all the tabs.
    pub fn all() -> Vec<Self> {
        use strum::IntoEnumIterator;
        Self::iter().collect()
    }
}
impl fmt::Display for FooterTab {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match self {
            FooterTab::Info => write!(fmt, "Info"),
            FooterTab::Filters => write!(fmt, "Filters"),
        }
    }
}
