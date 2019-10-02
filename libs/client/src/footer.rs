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
                if self.active == Some(tab) {
                    self.active = None
                } else {
                    self.active = Some(tab)
                }
                Ok(true)
            }
            Removed(uid) => {
                if self.active == Some(FooterTab::Filter(filter::LineUid::Filter(uid))) {
                    self.active = Some(FooterTab::Filter(filter::LineUid::Everything));
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
        }
    }

    /// Renders itself.
    pub fn render(&self, filters: &filter::Filters) -> Html {
        html! {
            <footer id = style::id::FOOTER>
                <ul class = style::class::footer::TABS>
                    <li class = style::class::footer::tabs::LEFT>
                        { Button::refresh(
                            "Reload all points in all charts",
                            |_| msg::to_server::ChartsMsg::reload().into()
                        ) }
                    </li>
                    <li class = style::class::footer::tabs::CENTER>
                        { for NormalFooterTab::all()
                            .into_iter()
                            .map(|tab|
                                tab.render(Some(tab) == self.active.and_then(FooterTab::get_normal_tab))
                            )
                        }
                        { filters.render_tabs(self.active.and_then(FooterTab::get_filter)) }
                    </li>
                    <li class = style::class::footer::tabs::RIGHT>
                        { Button::add(
                            "Create a new filter",
                            |_| msg::to_server::FiltersMsg::request_new().into()
                        ) }
                        {
                            if filters.edited() {
                                html! {
                                    <>
                                        { Button::save(
                                            "Save all changes",
                                            move |_| msg::FiltersMsg::save()
                                        ) }
                                        { Button::undo(
                                            "Undo all changes",
                                            move |_| msg::to_server::FiltersMsg::revert().into()
                                        ) }
                                    </>
                                }
                            } else {
                                html!(<a/>)
                            }
                        }
                    </li>
                </ul>
                {
                    match self.active {
                        None => html!(<a/>),
                        Some(FooterTab::Filter(active)) => html! {
                            <div class = style::class::footer::DISPLAY>
                                { filters.render_filter(active) }
                            </div>
                        },
                        Some(FooterTab::Normal(NormalFooterTab::Info)) => html! {
                            <div class = style::class::footer::DISPLAY>
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
            <li class = { style::class::tabs::li::get(true) }>
                <a
                    class = { style::class::tabs::get(active) }
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
    Filter(filter::LineUid),
}

impl FooterTab {
    /// Filter tab constructor.
    pub fn filter(uid: filter::LineUid) -> Self {
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
    pub fn get_filter(self) -> Option<filter::LineUid> {
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
