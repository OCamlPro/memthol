//! Handles the footer of the UI.
//!
//! The footer contains tabs for filters and statistics.

use crate::base::*;

pub mod filter;

/// Map from tabs to something.
pub type TabMap<Val> = Map<FooterTab, Val>;

/// A message for the control menu.
#[derive(Debug)]
pub enum FooterMsg {
    /// Toggle a footer tab.
    Toggle(footer::FooterTab),
    /// A message for the filter section.
    Filter(filter::FooterFilterMsg),
}
impl FooterMsg {
    /// Expands or collapses the control menu.
    pub fn toggle(tab: footer::FooterTab) -> Msg {
        Msg::FooterAction(Self::Toggle(tab))
    }
    /// Filter message.
    pub fn filter(msg: filter::FooterFilterMsg) -> Msg {
        Msg::FooterAction(Self::Filter(msg))
    }
}

/// Footer structure.
pub struct Footer {
    /// Mapping from tabs to a boolean indicating whether they're active.
    tabs: TabMap<bool>,
    /// Filter section.
    filters: filter::FilterFooter,
}

impl Footer {
    /// Constructor.
    pub fn new() -> Self {
        let mut tabs = TabMap::new();
        FooterTab::map_all(|tab| {
            let prev = tabs.insert(tab, false);
            debug_assert!(prev.is_none())
        });
        let filters = filter::FilterFooter::new();
        Footer { tabs, filters }
    }

    /// Returns the active tab, if any.
    fn get_active(&self) -> Option<FooterTab> {
        for (tab, active) in &self.tabs {
            if *active {
                return Some(*tab);
            }
        }
        None
    }

    /// Renders itself.
    pub fn render(&self) -> Html {
        html! {
            <div id=style::id::FOOTER>
                <ul id=style::id::FOOTER_TABS class=style::class::tabs::UL>
                    { for self.tabs.iter().map(|(tab, active)| tab.render(*active)) }
                </ul>
                {
                    match self.get_active() {
                        None => html!(<a/>),
                        Some(FooterTab::Filters) => html! {
                            <div class=style::class::footer::DISPLAY>
                                { self.filters.render() }
                            </div>
                        },
                        Some(FooterTab::Stats) => html! {
                            <div class=style::class::footer::DISPLAY>
                                { self.filters.render() }
                            </div>
                        },
                    }
                }
            </div>
        }
    }

    /// Handles a message.
    pub fn update(&mut self, msg: FooterMsg) -> ShouldRender {
        match msg {
            FooterMsg::Toggle(tab) => {
                let is_active_now = !*self
                    .tabs
                    .get(&tab)
                    .expect("[bug] footer was asked to toggle a tab that does not exist");
                if is_active_now {
                    // Tab is now active, activate tab and deactivate everything else.
                    for (tabb, active) in &mut self.tabs {
                        *active = *tabb == tab
                    }
                } else {
                    // Tab is now inactive, deactivate tab and do not touch other tabs.
                    let active = self
                        .tabs
                        .get_mut(&tab)
                        .expect("[bug] footer was asked to toggle a tab that does not exist");
                    *active = false
                }

                true
            }

            FooterMsg::Filter(msg) => self.filters.update(msg),
        }
    }
}

/// Footer tabs.
#[derive(Debug, Clone, Copy, PartialOrd, Ord, PartialEq, Eq)]
pub enum FooterTab {
    /// Statistics tab.
    Stats,
    /// Filters tab.
    Filters,
}
impl FooterTab {
    /// Applies a function to all tabs.
    pub fn map_all<F>(mut f: F)
    where
        F: FnMut(FooterTab),
    {
        f(Self::Stats);
        f(Self::Filters);
    }

    /// Renders a tab.
    pub fn render(&self, active: bool) -> Html {
        let tab = *self;
        html! {
            <li class={ style::class::tabs::li::get(true) }>
                <a
                    class={ style::class::tabs::get(active) }
                    onclick=|_| FooterMsg::toggle(tab)
                > {
                    self.to_string()
                } </a>
            </li>
        }
    }
}
impl fmt::Display for FooterTab {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match self {
            FooterTab::Stats => write!(fmt, "Stats"),
            FooterTab::Filters => write!(fmt, "Filters"),
        }
    }
}
