//! Top tabs of the client.

use crate::base::*;

/// Enumeration of the top tabs.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tab {
    View,
    Help,
    About,
}
impl Tab {
    /// String representation of the tab (its label).
    pub fn to_str(self) -> &'static str {
        match self {
            Tab::View => "View",
            Tab::Help => "Help",
            Tab::About => "About",
        }
    }

    /// Index of a tab from left to right.
    pub fn index(self) -> usize {
        match self {
            Tab::View => 0,
            Tab::Help => 1,
            Tab::About => 2,
        }
    }

    /// True if the tab is on the left side of the header.
    pub fn is_left(self) -> bool {
        match self {
            Tab::View => true,
            Tab::Help | Tab::About => false,
        }
    }

    /// Array of all the tabs.
    pub fn all() -> Vec<Self> {
        vec![
            // Left-hand part, normal order.
            Tab::View,
            // Right-hand part, reverse order.
            Tab::About,
            Tab::Help,
        ]
    }
}

/// A tab description.
#[derive(Clone, Debug)]
pub struct TabDesc {
    /// Kind of tab.
    kind: Tab,
    /// True if the tab is active.
    active: bool,
    /// True if the tab is on the left side.
    left: bool,
}
impl TabDesc {
    /// Creates an inactive tab.
    pub fn new(kind: Tab) -> Self {
        TabDesc {
            kind,
            active: false,
            left: kind.is_left(),
        }
    }

    /// Array of all tabs.
    pub fn all() -> Vec<Self> {
        Tab::all().into_iter().map(Self::new).collect()
    }

    /// Renders a tab.
    pub fn render(&self) -> Html {
        use cst::class::top_tab::*;
        let kind = self.kind;
        let left = if self.left { LEFT } else { RIGHT };
        html! {
            <li class={left}>
                <a
                    class={
                        if self.active {
                            ACTIVE
                        } else {
                            INACTIVE
                        }
                    }
                    onclick=|_| Msg::ChangeTab(kind)
                >
                    { self.kind.to_str() }
                </a>
            </li>
        }
    }
}

/// An array of tabs descriptions.
#[derive(Debug, Clone)]
pub struct TabDescs {
    content: Vec<TabDesc>,
}
impl std::ops::Index<Tab> for TabDescs {
    type Output = TabDesc;
    fn index(&self, index: Tab) -> &Self::Output {
        &self.content[index.index()]
    }
}
impl std::ops::IndexMut<Tab> for TabDescs {
    fn index_mut(&mut self, index: Tab) -> &mut Self::Output {
        &mut self.content[index.index()]
    }
}
impl std::ops::Deref for TabDescs {
    type Target = Vec<TabDesc>;
    fn deref(&self) -> &Self::Target {
        &self.content
    }
}
// impl std::ops::DerefMut for TabDescs {
//     fn deref_mut(&mut self) -> &mut Self::Target {
//         &mut self.content
//     }
// }

impl TabDescs {
    /// Constructor, all tabs are inactive by default.
    pub fn new() -> Self {
        Self {
            content: TabDesc::all(),
        }
    }

    /// Activates a tab, deactivates all other tabs.
    pub fn activate(&mut self, tab: Tab) -> ShouldRender {
        let mut changed = false;
        for current_tab in &mut self.content {
            let is_active = current_tab.kind == tab;
            if is_active != current_tab.active {
                changed = true;
                current_tab.active = is_active
            }
        }
        changed
    }
}

/// The top tabs.
#[derive(Clone, Debug)]
pub struct TopTabs {
    /// Vector of all tabs.
    tabs: TabDescs,
}
impl TopTabs {
    /// Constructor.
    ///
    /// On construction:
    ///
    /// - all tabs are inactive
    pub fn new() -> Self {
        TopTabs {
            tabs: TabDescs::new(),
        }
    }

    /// Activates the default tab.
    pub fn activate_default(&mut self) -> ShouldRender {
        self.activate(Tab::View)
    }

    /// Activates a tab and deactivates all active tab(s).
    pub fn activate(&mut self, tab: Tab) -> ShouldRender {
        self.tabs.activate(tab)
    }

    /// True if the active tab is `View`.
    pub fn is_view_active(&self) -> bool {
        for tab in self.tabs.iter() {
            if tab.active {
                return tab.kind == Tab::View;
            }
            if tab.kind == Tab::View {
                return tab.active;
            }
        }
        false
    }

    /// Renders all the tabs.
    pub fn render(&self) -> Html {
        html! {
            <ul class="tab_list">
                { for self.tabs.iter().map(TabDesc::render) }
            </ul>
        }
    }
}
