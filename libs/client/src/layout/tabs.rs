//! Tab rendering.

prelude! {}

define_style! {
    OUTTER_CELL_STYLE = {
        height(100%),
        width(auto),
        table cell,
        pointer,
    };
}

/// Properties of a tab.
#[derive(Clone)]
pub struct TabProps {
    /// Color.
    color: String,
    /// True if active.
    active: IsActive,
    /// True if what the tab represents was edited.
    edited: bool,
    /// True if dimmed.
    dimmed: bool,
    /// True if tabs are displayed in reverse order.
    rev: bool,
    /// Used to generate the z-index.
    ///
    /// Footer tabs have a higher z-index than tabs that appear in the body. So they should
    /// respectively set this `top` field to `true` and `false`.
    top: bool,
}
impl TabProps {
    /// Creates a tab with some color.
    pub fn new(color: impl Into<String>) -> Self {
        Self {
            color: color.into(),
            active: IsActive::from_bool(false),
            edited: false,
            dimmed: false,
            rev: false,
            top: false,
        }
    }

    /// Creates a new footer tab with some color.
    pub fn new_footer(color: impl Into<String>) -> Self {
        Self {
            color: color.into(),
            active: IsActive::from_bool(false),
            edited: false,
            dimmed: false,
            rev: false,
            top: true,
        }
    }
    /// Creates a new gray footer tab.
    pub fn new_footer_gray() -> Self {
        Self::new_footer("#c1c1c1")
    }

    /// Turns itself into button box properties
    pub fn to_box_props(&self) -> layout::button::BoxProps {
        let active = self.active.to_bool();
        layout::button::BoxProps::new_tab("black")
            .with_gradient_top(if active {
                layout::DARK_GREY_BG
            } else {
                &self.color
            })
            .with_gradient_bot(if active { &self.color } else { "black" })
            .with_stroke_px(1)
            .with_radius_px(10)
            .revert_if(self.rev)
            .for_footer(self.top)
    }

    /// Activates the tab.
    pub fn set_active(mut self, is_active: bool) -> Self {
        self.active = IsActive::from_bool(is_active);
        self
    }

    /// Constructor from a layout-info function.
    ///
    /// Function `get` returns a triplet containing
    ///
    /// - a flag indicating whether the tab can move left,
    /// - a flag indicating whether the tab can move right,
    /// - the UID of the filter the tab is for.
    ///
    /// > **NB**: `get` only runs if the tab is active.
    pub fn with_first_last_uid(mut self, get: impl FnOnce() -> (bool, bool, uid::Filter)) -> Self {
        self.active = self.active.with_first_last_uid(get);
        self
    }

    /// Sets the value of the internal `edited` flag.
    pub fn set_edited(mut self, is_edited: bool) -> Self {
        self.edited = is_edited;
        self
    }

    /// Sets whether the tab is dimmed.
    pub fn set_dimmed(mut self, is_dimmed: bool) -> Self {
        self.dimmed = is_dimmed;
        self
    }

    /// Sets whether the tab is reverse-order.
    pub fn set_rev(mut self) -> Self {
        self.rev = true;
        self
    }

    /// Makes sure the z-index of the tab-text is higher than everything else.
    pub fn footer_tab(mut self) -> Self {
        self.top = true;
        self
    }
}

/// Indicates whether a tab is active, and whether it can be moved.
#[derive(Clone, Copy)]
pub enum IsActive {
    /// Tab is not active.
    No,
    /// Tab is active, cannot move.
    Yes,
    /// Tab is active and may move.
    YesWith {
        /// True iff the tab can move left.
        can_move_left: bool,
        /// True iff the tab can move right.
        can_move_right: bool,
        /// UID of the filter the tab is for.
        uid: uid::Filter,
    },
}
impl IsActive {
    /// Yields `true` iff active.
    pub fn to_bool(&self) -> bool {
        match self {
            Self::No => false,
            Self::Yes | Self::YesWith { .. } => true,
        }
    }
    /// Constructor from a boolean.
    pub fn from_bool(active: bool) -> Self {
        if active {
            Self::Yes
        } else {
            Self::No
        }
    }

    /// Constructor from a layout-info function.
    ///
    /// Function `get` returns a triplet containing
    ///
    /// - a flag indicating whether the tab can move left,
    /// - a flag indicating whether the tab can move right,
    /// - the UID of the filter the tab is for.
    ///
    /// > **NB**: `get` only runs if the tab is active.
    pub fn with_first_last_uid(self, get: impl FnOnce() -> (bool, bool, uid::Filter)) -> Self {
        match self {
            Self::No => Self::No,
            Self::YesWith { .. } | Self::Yes => {
                let (can_move_left, can_move_right, uid) = get();
                Self::YesWith {
                    can_move_left,
                    can_move_right,
                    uid,
                }
            }
        }
    }
}

/// CSS for some tab properties.
pub fn style(props: &TabProps) -> String {
    let active = props.active.to_bool();
    // let rev = props.rev;

    // let shadow_h_off = 4;
    // let shadow_v_off = if rev { 2 } else { -2 };
    // let (shadow_blur, shadow_spread) = if active { (34, 7) } else { (20, 1) };

    inline_css!(
        height(100%),
        width(auto),
        table,
        text_align(center),

        if(
            active,
            {
                pos(relative),
                z_index({
                    if props.top { 650 } else { 400 }
                }),
            },
        ),

        if(
            props.edited,
            italic,
        ),
    )
}

/// A list of tabs.
pub struct Tabs {
    /// The list of tabs.
    tabs: SVec64<Html>,
}

impl Tabs {
    /// Constructs an empty list of tabs.
    pub fn new() -> Self {
        Self {
            tabs: SVec64::new(),
        }
    }

    /// Pushes a tab.
    pub fn push_tab(&mut self, model: &Model, text: &str, props: TabProps, onclick: OnClickAction) {
        self.inner_push_tab(model, text, props, onclick)
    }

    /// Pushes a tab, handles the whole move-right/move-left business.
    fn inner_push_tab(
        &mut self,
        model: &Model,
        text: &str,
        props: TabProps,
        onclick: OnClickAction,
    ) {
        let edited = props.edited;
        let mut res = if edited {
            Self::raw_tab(&props, onclick, format!("*{}*", text))
        } else {
            Self::raw_tab(&props, onclick, text)
        };

        if let IsActive::YesWith {
            can_move_left,
            can_move_right,
            uid,
        } = props.active
        {
            if can_move_left {
                res = html! {
                    <>
                        {Self::raw_tab(
                            &props,
                            model.link.callback(
                                move |_| msg::filter::Msg::move_filter(uid, true)
                            ),
                            "<"
                        )}
                        {res}
                    </>
                };
            }
            if can_move_right {
                res = html! {
                    <>
                        {res}
                        {Self::raw_tab(
                            &props,
                            model.link.callback(
                                move |_| msg::filter::Msg::move_filter(uid, false)
                            ),
                            ">"
                        )}
                    </>
                }
            }
        }

        self.tabs.push(res)
    }

    /// Displays a raw tab.
    fn raw_tab(props: &TabProps, onclick: OnClickAction, content: impl fmt::Display) -> Html {
        html! {
            <div
                id = "filter_tab_cell"
                style = OUTTER_CELL_STYLE
            >
                <div
                    id = "filter_tab"
                    style = style(props)
                >
                    {layout::button::text::render(
                        Some(props.to_box_props()),
                        "filter_content",
                        content,
                        Some(onclick),
                        props.dimmed,
                    )}
                </div>
            </div>
        }
    }

    /// Pushes a tab which is really just an image.
    pub fn push_img_tab(
        &mut self,
        dimension_px: usize,
        props: TabProps,
        onclick: Option<OnClickAction>,
        img: layout::button::img::Img,
        desc: impl fmt::Display,
    ) {
        if let Some(onclick) = onclick {
            self.tabs
                .push(Self::raw_img_tab(dimension_px, props, onclick, img, desc))
        }
    }
    fn raw_img_tab(
        dimension_px: usize,
        props: TabProps,
        onclick: OnClickAction,
        img: layout::button::img::Img,
        desc: impl fmt::Display,
    ) -> Html {
        html! {
            <div
                id = "filter_tab_cell"
                style = OUTTER_CELL_STYLE
            >
                <div
                    id = "filter_tab"
                    style = style(&props)
                >
                    {img.button_render(
                        dimension_px,
                        Some(props.to_box_props()),
                        "filter_content",
                        desc,
                        onclick,
                        props.dimmed,
                    )}
                </div>
            </div>
        }
    }

    /// Pushes a separation, *i.e.* a tiny amount of space.
    pub fn push_sep(&mut self) {
        define_style! {
            SEP = {
                width(10 px),
                height(100%),
                table cell,
            };
        }

        self.tabs.push(html! {
            <div
                id = "tab_sep"
                style = SEP
            >
                {"\u{00a0}"}
            </div>
        })
    }

    /// Pushes a right separator, usually at the very end of a list of tabs.
    pub fn push_sep_right(&mut self) {
        define_style! {
            SEP = {
                width(10 px),
                height(100%),
                float(right),
                table cell,
            };
        }

        self.tabs.push(html! {
            <div
                id = "tab_sep"
                style = SEP
            >
                {"\u{00a0}"}
            </div>
        })
    }

    /// Renders the tabs.
    pub fn render(self) -> Html {
        define_style! {
            TABS_ROW = {
                width(auto),
                height(100%),
                table,
            };
        }

        html! {
            <div
                style = TABS_ROW
            >
                {for self.tabs.into_iter()}
            </div>
        }
    }

    /// Renders the tabs, float-right style.
    pub fn render_right(self) -> Html {
        define_style! {
            TABS_ROW = {
                width(auto),
                height(100%),
                table,
                float(right),
            };
        }

        html! {
            <div
                style = TABS_ROW
            >
                {for self.tabs.into_iter()}
            </div>
        }
    }
}
