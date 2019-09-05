//! Handles the footer of the UI.
//!
//! The footer contains tabs for filters and statistics.

use crate::base::*;

/// Map from tabs to something.
pub type TabMap<Val> = Map<FooterTab, Val>;

/// A message for the control menu.
#[derive(Debug)]
pub enum FooterMsg {
    /// Toggle a footer tab.
    Toggle(footer::FooterTab),
}
impl FooterMsg {
    /// Expands or collapses the control menu.
    pub fn toggle(tab: footer::FooterTab) -> Msg {
        Msg::FooterAction(FooterMsg::Toggle(tab))
    }
}

/// Footer structure.
pub struct Footer {
    /// Mapping from tabs to a boolean indicating whether they're active.
    tabs: TabMap<bool>,
}

impl Footer {
    /// Constructor.
    pub fn new() -> Self {
        let mut tabs = TabMap::new();
        FooterTab::map_all(|tab| {
            let prev = tabs.insert(tab, false);
            debug_assert!(prev.is_none())
        });
        Footer { tabs }
    }

    /// Renders itself.
    pub fn render(&self) -> Html {
        html! {
            <ul id="footer_title" class="tab_list">
                { for self.tabs.iter().map(|(tab, active)| tab.render(*active)) }
            </ul>
        }
    }

    /// Handles a message.
    pub fn update(&mut self, msg: FooterMsg) -> ShouldRender {
        info!("receiving message {:?}", msg);
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
            }
        }
        true
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
            <li class="li_center">
                <a
                    class={
                        if active {
                            cst::class::top_tab::ACTIVE
                        } else {
                            cst::class::top_tab::INACTIVE
                        }
                    }
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
