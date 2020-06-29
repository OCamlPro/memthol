//! Tab rendering.

prelude! {}

pub const height: usize = 100;

pub const sep_width: usize = 2;
pub const max_width: usize = 30;
pub const min_width: usize = 5;

define_style! {
    OUTTER_CELL_STYLE = {
        height(100%),
        width(min {min_width}%),
        table cell,
        pointer,
    };
}

#[derive(Clone)]
pub struct TabProps {
    color: String,
    active: IsActive,
    edited: bool,
    dimmed: bool,
    rev: bool,
}
impl TabProps {
    pub fn new(color: impl Into<String>) -> Self {
        Self {
            color: color.into(),
            active: IsActive::from_bool(false),
            edited: false,
            dimmed: false,
            rev: false,
        }
    }
    pub fn new_active(color: impl Into<String>) -> Self {
        Self {
            color: color.into(),
            active: IsActive::from_bool(true),
            edited: false,
            dimmed: false,
            rev: false,
        }
    }
    pub fn new_inactive(color: impl Into<String>) -> Self {
        Self {
            color: color.into(),
            active: IsActive::from_bool(false),
            edited: false,
            dimmed: false,
            rev: false,
        }
    }

    pub fn to_box_props(&self) -> layout::button::BoxProps {
        layout::button::BoxProps::new_tab("black")
            .with_gradient_top(&self.color)
            .with_gradient_bot("black")
            .with_stroke_px(1)
            .with_radius_px(5)
            .revert_if(self.rev)
    }

    pub fn set_active(mut self, is_active: bool) -> Self {
        self.active = IsActive::from_bool(is_active);
        self
    }
    pub fn with_first_last_uid(
        mut self,
        get: impl FnOnce() -> (bool, bool, filter::FilterUid),
    ) -> Self {
        self.active = self.active.with_first_last_uid(get);
        self
    }
    pub fn set_edited(mut self, is_edited: bool) -> Self {
        self.edited = is_edited;
        self
    }
    pub fn set_dimmed(mut self, is_dimmed: bool) -> Self {
        self.dimmed = is_dimmed;
        self
    }
    pub fn set_rev(mut self) -> Self {
        self.rev = true;
        self
    }
}

#[derive(Clone, Copy)]
pub enum IsActive {
    No,
    Yes,
    YesWith {
        can_move_left: bool,
        can_move_right: bool,
        uid: filter::FilterUid,
    },
}
impl IsActive {
    pub fn to_bool(&self) -> bool {
        match self {
            Self::No => false,
            Self::Yes | Self::YesWith { .. } => true,
        }
    }
    pub fn from_bool(active: bool) -> Self {
        if active {
            Self::Yes
        } else {
            Self::No
        }
    }
    pub fn with_first_last_uid(
        self,
        get: impl FnOnce() -> (bool, bool, filter::FilterUid),
    ) -> Self {
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

pub fn style(props: &TabProps) -> String {
    let active = props.active.to_bool();
    let rev = props.rev;

    let shadow_h_off = 4;
    let shadow_v_off = if rev { 2 } else { -2 };
    let (shadow_blur, shadow_spread) = if active { (34, 7) } else { (20, 1) };

    inline_css!(
        height(100%),
        width(auto),
        table,
        text_align(center),

        if(
            active,
            {
                pos(relative),
                z_index(650),
            },
        ),

        if(
            props.edited,
            italic,
        ),

        box_shadow(
            {shadow_h_off} px,
            {shadow_v_off} px,
            {shadow_blur} px,
            {shadow_spread} px,
            {&props.color},
        ),
    )
}

pub struct Tabs {
    tabs: SVec<Html>,
}

impl Tabs {
    pub fn new() -> Self {
        Self { tabs: SVec::new() }
    }

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

    pub fn push_tab(&mut self, model: &Model, text: &str, props: TabProps, onclick: OnClickAction) {
        self.inner_push_tab(model, text, props, onclick)
    }

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
                                move |_| msg::FiltersMsg::move_filter(uid, true)
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
                                move |_| msg::FiltersMsg::move_filter(uid, false)
                            ),
                            ">"
                        )}
                    </>
                }
            }
        }

        self.tabs.push(res)
    }

    pub fn push_sep(&mut self) {
        define_style! {
            SEP = {
                width(10%),
                height(100%),
                table cell,
            };
        }

        self.tabs.push(html! {
            <div
                id = "tab_sep"
                style = SEP
            />
        })
    }

    pub fn render(self) -> Html {
        define_style! {
            TABS_ROW = {
                height(100%),
                table,
                table_layout(fixed),
            };

            END_SEP = {
                width(auto),
                height(100%),
                table cell,
            };
        }

        html! {
            <div
                style = TABS_ROW
            >
                {for self.tabs.into_iter()}
                <div id = "tab_end_sep" style = END_SEP/>
            </div>
        }
    }
}
