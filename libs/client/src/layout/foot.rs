//! HTML builders.
#![allow(non_upper_case_globals)]

prelude! {}

pub const width_wrt_full: usize = 100;

pub const collapsed_height_px: usize = 50;
pub const expanded_height_px: usize = 500;

pub const tabs_height_px: usize = collapsed_height_px;

pub const expanded_tabs_height_px: usize = tabs_height_px;
pub const expanded_menu_height_px: usize = expanded_height_px - expanded_tabs_height_px;

pub const collapsed_tabs_height_px: usize = tabs_height_px;
pub const collapsed_menu_height_px: usize = tabs_height_px - collapsed_tabs_height_px;

pub const center_tile_width: usize = 70;
pub const left_tile_width: usize = (width_wrt_full - center_tile_width) / 2;
pub const right_tile_width: usize = left_tile_width;

define_style! {
    footer_style! = {
        font(default),
        fg({layout::LIGHT_BLUE_FG}),
        bg(transparent),
        fixed(bottom),
        z_index(600),
        width({width_wrt_full}%),
    };

    COLLAPSED_STYLE = {
        extends(footer_style),
        height({collapsed_height_px}px)
    };

    EXPANDED_STYLE = {
        extends(footer_style),
        height({expanded_height_px}px)
    };

    tabs_style! = {
        top,
        width({width_wrt_full}%),
    };
    COLLAPSED_TABS_STYLE = {
        extends(tabs_style),
        height({collapsed_tabs_height_px}px),
    };
    EXPANDED_TABS_STYLE = {
        extends(tabs_style),
        height({expanded_tabs_height_px}px),
    };

    menu_style! = {
        fg({layout::LIGHT_BLUE_FG}),
        bg({layout::DARK_GREY_BG}),
        bottom,
        width({width_wrt_full}%),
        // box_shadow(0 px, {-7} px, 50 px, 1 px, {layout::DARK_GREY_BG}),
        border_radius(20 px, 20 px, 0 px, 0 px),
        z_index(600),
        pos(relative),
    };
    COLLAPSED_MENU_STYLE = {
        extends(menu_style),
        height(100%),
    };
    EXPANDED_MENU_STYLE = {
        extends(menu_style),
        height({expanded_menu_height_px}px),
    };
}

/// Renders the footer.
pub fn render(footer: &footer::Footer, model: &Model) -> Html {
    match footer.active {
        None => html! {
            <footer
                id = "collapsed_footer"
                style = COLLAPSED_STYLE
            >
                <div
                    id = "collapsed_tabs_tile"
                    style = COLLAPSED_TABS_STYLE
                >
                    { tabs::render(model, None) }
                </div>
                <div
                    id = "collapsed_menu_tile"
                    style = COLLAPSED_MENU_STYLE
                />
            </footer>
        },
        Some(footer::FooterTab::Normal(_)) => unimplemented!(),
        Some(footer::FooterTab::Filter(filter_uid)) => {
            html! {
                <footer
                    id = "expanded_footer"
                    style = EXPANDED_STYLE
                >
                    <div
                        id = "expanded_tabs_tile"
                        style = EXPANDED_TABS_STYLE
                    >
                        { tabs::render(model, Some(filter_uid)) }
                    </div>
                    <div
                        id = "expanded_menu_tile"
                        style = EXPANDED_MENU_STYLE
                    >
                        {
                            if let Ok(filter) = model.footer_filters().get_spec(filter_uid) {
                                menu::render_filter(model, filter)
                            } else {
                                html!(<></>)
                            }
                        }
                    </div>
                </footer>
            }
        }
    }
}

define_style! {
    tile_style! = {
        height(100%),
        float(left),
    };
}

pub mod menu {
    use super::*;

    define_style! {
        menu_fg! = {
            fg({layout::LIGHT_BLUE_FG}),
        };
        MENU_LEFT_TILE = {
            extends(menu_fg),
            height(100%),
            width({left_tile_width}%),
            float(left),
        };
        MENU_RIGHT_TILE = {
            extends(menu_fg),
            height(100%),
            width({right_tile_width}%),
            float(left),
            text_align(center),
        };
        MENU_CENTER_TILE = {
            extends(menu_fg),
            height(100%),
            width({center_tile_width}%),
            float(left),
            overflow(auto),
            // border(left, 1 px, white),
            // border(right, 1 px, white),
        };

        // code_style! = {
        //     border_radius(5 px),
        //     margin(none),
        //     padding(0%, 1%),
        //     border(none),
        //     bg({"#3a3a3a"}),
        //     font(code),
        // };
    }

    pub fn render_filter(model: &Model, filter: &filter::FilterSpec) -> Html {
        let center = html! {
            <>
                {settings::render(model, filter)}
                {{
                    let empty = || html! { <></> };
                    match filter.uid() {
                        filter::LineUid::CatchAll |
                        filter::LineUid::Everything => empty(),
                        filter::LineUid::Filter(uid) => if let Ok(
                            (_index, filter)
                        ) = model.footer_filters().get_filter(uid) {
                            subfilters::render(model, filter)
                        } else {
                            empty()
                        }
                    }
                }}
            </>
        };
        html! {
            <>
                { render_left_tile() }
                { render_center_tile(center) }
                { render_right_tile(filter_right_tile::render(model, filter.uid())) }
            </>
        }
    }

    pub fn render_left_tile() -> Html {
        html! {
            <div
                id = "menu_left_tile"
                style = MENU_LEFT_TILE
            />
        }
    }

    pub fn render_right_tile(inner: Html) -> Html {
        html! {
            <div
                id = "menu_right_tile"
                style = MENU_RIGHT_TILE
            >
                {inner}
            </div>
        }
    }

    pub mod filter_right_tile {
        use super::*;

        pub fn render(model: &Model, uid: filter::LineUid) -> Html {
            html! {
                <>
                    <br/><br/>
                    {add_subfilter_button(model, uid)}
                    <br/>
                </>
            }
        }

        pub fn button(id: &str, txt: impl fmt::Display, onclick: Option<OnClickAction>) -> Html {
            define_style! {
                BUTTON_CONTAINER = {
                    width(70%),
                    height(10%),
                    margin(auto),
                };
            }

            html! {
                <div
                    id = format!("{}_container", id)
                    style = BUTTON_CONTAINER
                >
                    {layout::button::text::render_default_button(
                        id,
                        txt,
                        onclick,
                        false,
                    )}
                </div>
            }
        }

        pub fn add_subfilter_button(model: &Model, uid: filter::LineUid) -> Html {
            let action = match uid {
                filter::LineUid::Filter(uid) => {
                    Some(model.link.callback(move |_| msg::FilterMsg::add_new(uid)))
                }
                filter::LineUid::Everything | filter::LineUid::CatchAll => None,
            };
            button("add_subfilter_button", "add subfilter", action)
        }
    }

    pub fn render_center_tile(inner: Html) -> Html {
        html! {
            <div
                id = "menu_center_tile"
                style = MENU_CENTER_TILE
            >
                {inner}
            </div>
        }
    }

    pub mod settings {
        use super::*;

        pub fn render(model: &Model, filter: &filter::FilterSpec) -> Html {
            html! {
                <>
                    <br/>
                    {layout::section("Settings")}
                    <br/>

                    {render_name_row(model, filter)}
                    {render_color_row(model, filter)}
                </>
            }
        }

        pub fn render_name_row(model: &Model, filter: &filter::FilterSpec) -> Html {
            let mut table_row = layout::table::TableRow::new_menu(true, html! { "name" });
            table_row.push_single_value({
                let uid = filter.uid();
                layout::input::text_input(
                    filter.name(),
                    model
                        .link
                        .callback(move |data| msg::FilterSpecMsg::change_name(uid, data)),
                )
            });
            table_row.render()
        }
        pub fn render_color_row(model: &Model, filter: &filter::FilterSpec) -> Html {
            let mut table_row = layout::table::TableRow::new_menu(false, html! { "color" });
            table_row.push_single_value({
                let uid = filter.uid();
                layout::input::color_input(
                    filter.color(),
                    model
                        .link
                        .callback(move |data| msg::FilterSpecMsg::change_color(uid, data)),
                )
            });
            table_row.render()
        }
    }

    pub mod subfilters {
        use super::*;
        use charts::filter::{sub::RawSubFilter, LifetimeFilter, SizeFilter, SubFilter};

        pub fn render(model: &Model, filter: &filter::Filter) -> Html {
            // if !filter.has_sub_filters() {
            //     return html! {<></>};
            // }

            html! {
                <>
                    <br/>
                    {layout::section("Sub-Filters")}
                    <br/>

                    {
                        for filter.iter().enumerate().map(
                            |(index, sub)| render_sub(model, filter.uid(), index == 0, sub)
                        )
                    }
                </>
            }
        }

        pub fn render_sub(
            model: &Model,
            uid: filter::FilterUid,
            is_first: bool,
            sub: &filter::SubFilter,
        ) -> Html {
            let key = render_key(model, uid, sub);
            let mut table_row = layout::table::TableRow::new_menu(is_first, key);
            let sub_uid = sub.uid();
            match sub.raw() {
                RawSubFilter::Size(sub) => {
                    size::render(&mut table_row, model, sub, move |size_sub_filter_res| {
                        err::msg_of_res(size_sub_filter_res.map(|size| {
                            msg::FilterMsg::update_sub(
                                uid,
                                filter::SubFilter::new(sub_uid, RawSubFilter::Size(size)),
                            )
                        }))
                    })
                }
                RawSubFilter::Lifetime(sub) => {
                    lifetime::render(&mut table_row, model, sub, move |lifetime_sub_filter_res| {
                        err::msg_of_res(lifetime_sub_filter_res.map(|lifetime| {
                            msg::FilterMsg::update_sub(
                                uid,
                                filter::SubFilter::new(sub_uid, RawSubFilter::Lifetime(lifetime)),
                            )
                        }))
                    })
                }
                RawSubFilter::Label(sub) => {
                    label::render(&mut table_row, model, sub, move |label_sub_filter_res| {
                        err::msg_of_res(label_sub_filter_res.map(|label| {
                            msg::FilterMsg::update_sub(
                                uid,
                                filter::SubFilter::new(sub_uid, RawSubFilter::Label(label)),
                            )
                        }))
                    })
                }
                RawSubFilter::Loc(sub) => {
                    location::render(&mut table_row, model, sub, move |loc_sub_filter_res| {
                        err::msg_of_res(loc_sub_filter_res.map(|loc| {
                            msg::FilterMsg::update_sub(
                                uid,
                                filter::SubFilter::new(sub_uid, RawSubFilter::Loc(loc)),
                            )
                        }))
                    })
                }
            };

            table_row.render()
        }

        #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
        pub enum SubKey {
            Remove,
            Change(charts::filter::FilterKind),
        }
        impl SubKey {
            pub fn all() -> Vec<SubKey> {
                charts::filter::FilterKind::all()
                    .into_iter()
                    .map(Self::Change)
                    .chain(Some(Self::Remove))
                    .collect()
            }
            pub fn from_kind(kind: charts::filter::FilterKind) -> Self {
                Self::Change(kind)
            }
        }
        impl fmt::Display for SubKey {
            fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
                match self {
                    Self::Remove => write!(fmt, "remove"),
                    Self::Change(kind) => kind.fmt(fmt),
                }
            }
        }

        pub fn render_key(model: &Model, uid: filter::FilterUid, sub: &SubFilter) -> Html {
            let sub_uid = sub.uid();
            let options = SubKey::all();
            let selected = Some(SubKey::from_kind(sub.kind()));
            let sub_clone = sub.clone();
            html! {
                <Select<SubKey>
                    options = options
                    selected = selected
                    on_change = model.link.callback(
                        move |sub_key| match sub_key {
                            SubKey::Change(kind) => {
                                let mut sub = sub_clone.clone();
                                sub.change_kind(kind);
                                msg::FilterMsg::update_sub(uid, sub)
                            }
                            SubKey::Remove => {
                                msg::FilterMsg::rm_sub(uid, sub_uid)
                            }
                        }
                    )
                />
            }
        }

        pub mod size {
            use super::*;
            use charts::filter::ord::Kind;

            pub fn render<Update>(
                table_row: &mut layout::table::TableRow,
                model: &Model,
                sub: &SizeFilter,
                msg: Update,
            ) where
                Update: Fn(Res<SizeFilter>) -> Msg + 'static + Clone,
            {
                let selector = {
                    let selected = Some(sub.cmp_kind());
                    let sub_clone = sub.clone();
                    let msg = msg.clone();
                    html! {
                        <Select<Kind>
                            selected = selected
                            options = Kind::all()
                            on_change = model.link.callback(
                                move |kind| {
                                    let sub = sub_clone.clone().change_cmp_kind(kind);
                                    msg(Ok(sub))
                                }
                            )
                        />
                    }
                };
                table_row.push_selector(selector);

                match *sub {
                    SizeFilter::Cmp { cmp, val } => {
                        table_row.push_value(layout::input::u32_input(
                            model,
                            val,
                            move |usize_res| msg(usize_res.map(|val| SizeFilter::Cmp { cmp, val })),
                        ));
                        table_row.push_value(html! {
                            "machine word(s)"
                        })
                    }
                    SizeFilter::In { lb, ub } => {
                        let msg_fn = msg.clone();
                        let lb_html = layout::input::u32_input(model, lb, move |usize_res| {
                            msg_fn(usize_res.map(|lb| SizeFilter::In { lb, ub }))
                        });
                        let ub_html = layout::input::u32_input(model, ub, move |usize_res| {
                            msg(usize_res.map(|ub| SizeFilter::In { lb, ub }))
                        });
                        table_row.push_sep(html! {"["});
                        table_row.push_value(lb_html);
                        table_row.push_sep(html! {","});
                        table_row.push_value(ub_html);
                        table_row.push_sep(html! {"]"});
                    }
                }
            }
        }

        pub mod lifetime {
            use super::*;
            use charts::filter::ord::Kind;

            pub fn render(
                table_row: &mut layout::table::TableRow,
                model: &Model,
                sub: &LifetimeFilter,
                msg: impl Fn(Res<LifetimeFilter>) -> Msg + 'static + Clone,
            ) {
                let selector = {
                    let selected = Some(sub.cmp_kind());
                    let sub_clone = sub.clone();
                    let msg = msg.clone();
                    html! {
                        <Select<Kind>
                            selected = selected
                            options = Kind::all()
                            on_change = model.link.callback(
                                move |kind| {
                                    let sub = sub_clone.clone().change_cmp_kind(kind);
                                    msg(Ok(sub))
                                }
                            )
                        />
                    }
                };
                table_row.push_selector(selector);

                match *sub {
                    LifetimeFilter::Cmp { cmp, val } => {
                        table_row.push_value(layout::input::lifetime_input(
                            model,
                            val,
                            move |usize_res| {
                                msg(usize_res.map(|val| LifetimeFilter::Cmp { cmp, val }))
                            },
                        ));
                        table_row.push_value(html! {
                            "second(s)"
                        })
                    }
                    LifetimeFilter::In { lb, ub } => {
                        let msg_fn = msg.clone();
                        let lb_html = layout::input::lifetime_input(model, lb, move |usize_res| {
                            msg_fn(usize_res.map(|lb| LifetimeFilter::In { lb, ub }))
                        });
                        let ub_html = layout::input::lifetime_input(model, ub, move |usize_res| {
                            msg(usize_res.map(|ub| LifetimeFilter::In { lb, ub }))
                        });
                        table_row.push_sep(html! {"["});
                        table_row.push_value(lb_html);
                        table_row.push_sep(html! {","});
                        table_row.push_value(ub_html);
                        table_row.push_sep(html! {"]"});
                    }
                }
            }
        }

        pub mod label {
            use super::*;
            use charts::filter::{
                label::{LabelPred, LabelSpec},
                LabelFilter,
            };

            pub fn render(
                table_row: &mut layout::table::TableRow,
                model: &Model,
                sub: &LabelFilter,
                msg: impl Fn(Res<LabelFilter>) -> Msg + 'static + Clone,
            ) {
                macro_rules! push_add_button {
                    ($idx:expr) => {
                        table_row.push_button("+", {
                            let msg = msg.clone();
                            let sub = sub.clone();
                            let idx = $idx;
                            model.link.callback(move |_| {
                                let mut sub = sub.clone();
                                sub.insert(idx, LabelSpec::default());
                                msg(Ok(sub))
                            })
                        });
                    };
                }

                let selector = {
                    let selected = Some(sub.pred().clone());
                    let specs = sub.specs().clone();
                    let msg = msg.clone();
                    html! {
                        <Select<LabelPred>
                            selected = selected
                            options = LabelPred::all()
                            on_change = model.link.callback(
                                move |new_pred| msg(Ok(
                                    LabelFilter::new(new_pred, specs.clone())
                                ))
                            )
                        />
                    }
                };
                table_row.push_selector(selector);

                for (idx, spec) in sub.specs().iter().enumerate() {
                    push_add_button!(idx);

                    let value = spec.to_string();
                    let inner = layout::input::string_input(model, &value, {
                        let msg = msg.clone();
                        let sub = sub.clone();

                        move |str_res| {
                            msg(str_res.and_then(LabelSpec::new).map(|spec| {
                                let mut sub = sub.clone();
                                sub.replace(idx, spec);
                                sub
                            }))
                        }
                    });
                    if spec.matches_anything() {
                        table_row.push_tiny_value(inner)
                    } else {
                        table_row.push_value(inner)
                    }
                }

                push_add_button!(sub.specs().len());
            }
        }

        pub mod location {
            use super::*;
            use charts::filter::{
                loc::{LocPred, LocSpec},
                LocFilter,
            };

            pub fn render(
                table_row: &mut layout::table::TableRow,
                model: &Model,
                sub: &LocFilter,
                msg: impl Fn(Res<LocFilter>) -> Msg + 'static + Clone,
            ) {
                let selector = {
                    let selected = Some(sub.pred().clone());
                    let specs = sub.specs().clone();
                    let msg = msg.clone();
                    html! {
                        <Select<LocPred>
                            selected = selected
                            options = LocPred::all()
                            on_change = model.link.callback(
                                move |pred| msg(Ok(LocFilter::new(pred, specs.clone())))
                            )
                        />
                    }
                };
                table_row.push_selector(selector);

                macro_rules! push_add_button {
                    ($idx:expr) => {
                        table_row.push_button("+", {
                            let msg = msg.clone();
                            let sub = sub.clone();
                            let idx = $idx;
                            model.link.callback(move |_| {
                                let mut sub = sub.clone();
                                sub.insert(idx, LocSpec::default());
                                msg(Ok(sub))
                            })
                        });
                    };
                }

                for (idx, spec) in sub.specs().iter().enumerate() {
                    push_add_button!(idx);

                    let value = spec.to_string();
                    let inner = layout::input::string_input(model, &value, {
                        let msg = msg.clone();
                        let sub = sub.clone();

                        move |str_res| {
                            msg(str_res.and_then(LocSpec::new).map(|spec| {
                                let mut sub = sub.clone();
                                sub.replace(idx, spec);
                                sub
                            }))
                        }
                    });
                    if spec.matches_anything() {
                        table_row.push_tiny_value(inner)
                    } else {
                        table_row.push_value(inner)
                    }
                }

                push_add_button!(sub.specs().len());
            }
        }
    }
}

pub mod tabs {
    use super::*;
    use layout::tabs::TabProps;

    pub const width: usize = 100;
    pub const height: usize = 100;

    const img_dim_px: usize = 4 * (tabs_height_px / 5);

    pub fn render(model: &Model, active: Option<filter::LineUid>) -> Html {
        html! {
            <>
                { tabs_left::render(model) }
                { tabs_center::render(model, active) }
                { tabs_right::render(model, active.and_then(|uid| uid.filter_uid())) }
            </>
        }
    }

    pub mod tabs_left {
        use super::*;

        define_style! {
            LEFT_STYLE = {
                extends(tile_style),
                width({left_tile_width}%),
                table,
            };
        }
        pub fn render(model: &Model) -> Html {
            let mut tabs = layout::tabs::Tabs::new();

            let edited = model.footer_filters().edited();

            tabs.push_img_tab(
                img_dim_px,
                TabProps::new_footer_gray(),
                if edited {
                    Some(
                        model
                            .link
                            .callback(move |_| msg::to_server::FiltersMsg::revert()),
                    )
                } else {
                    None
                },
                layout::button::img::Img::Undo,
                "undo all modifications",
            );
            tabs.push_img_tab(
                img_dim_px,
                TabProps::new_footer_gray(),
                if edited {
                    Some(model.link.callback(move |_| msg::FiltersMsg::save()))
                } else {
                    None
                },
                layout::button::img::Img::Check,
                "save all modifications",
            );

            tabs.push_sep_right();

            html! {
                <div
                    id = "left_tabs_tile"
                    style = LEFT_STYLE
                >
                    {tabs.render_right()}
                </div>
            }
        }
    }

    pub mod tabs_right {
        use super::*;

        define_style! {
            RIGHT_STYLE = {
                extends(tile_style),
                width({right_tile_width}%),
                table,
            };
        }

        pub fn render(model: &Model, current_filter: Option<filter::FilterUid>) -> Html {
            let mut tabs = layout::tabs::Tabs::new();

            tabs.push_sep();

            tabs.push_img_tab(
                img_dim_px,
                TabProps::new_footer_gray(),
                Some(
                    model
                        .link
                        .callback(move |_| msg::to_server::FiltersMsg::request_new()),
                ),
                layout::button::img::Img::Plus,
                "add a new filter",
            );
            tabs.push_img_tab(
                img_dim_px,
                TabProps::new_footer_gray(),
                current_filter.map(|uid| model.link.callback(move |_| msg::FiltersMsg::rm(uid))),
                layout::button::img::Img::Minus,
                "remove current filter",
            );

            html! {
                <div
                    id = "right_tabs_tile"
                    style = RIGHT_STYLE
                >
                    {tabs.render()}
                </div>
            }
        }
    }

    pub mod tabs_center {
        use super::*;

        define_style! {
            CENTER_STYLE = {
                extends(tile_style),
                width({center_tile_width}%),
                block,
                overflow(x: auto),
            };
            TABS_ROW = {
                height(100%),
                table,
                table_layout(fixed),
            };
        }

        pub fn render(model: &Model, active: Option<filter::LineUid>) -> Html {
            let (everything, others) = model.footer_filters().filters_to_render();

            let is_active = |filter: &filter::FilterSpec| {
                active.map(|uid| uid == filter.uid()).unwrap_or(false)
            };
            let callback = |uid: filter::LineUid| {
                model
                    .link
                    .callback(move |_| msg::FooterMsg::toggle_tab(footer::FooterTab::filter(uid)))
            };

            let is_edited =
                |filter: &filter::FilterSpec| model.filters.is_filter_edited(filter.uid());

            let push_spec = |tabs: &mut layout::tabs::Tabs, filter: &filter::FilterSpec| {
                tabs.push_tab(
                    model,
                    filter.name(),
                    TabProps::new_footer(filter.color().to_string())
                        .set_active(is_active(filter))
                        .set_edited(is_edited(filter)),
                    callback(filter.uid()),
                )
            };

            let push_filter =
                |tabs: &mut layout::tabs::Tabs, index: usize, filter: &filter::Filter| {
                    tabs.push_tab(
                        model,
                        filter.spec().name(),
                        TabProps::new_footer(filter.spec().color().to_string())
                            .set_active(is_active(filter.spec()))
                            .with_first_last_uid(|| {
                                (
                                    0 < index,
                                    index + 1 < model.footer_filters().filters.len(),
                                    filter.uid(),
                                )
                            })
                            .set_edited(is_edited(filter.spec())),
                        callback(filter.spec().uid()),
                    )
                };

            let mut tabs = layout::tabs::Tabs::new();

            tabs.push_sep();
            push_spec(&mut tabs, everything);

            if let Some((catch_all, filters)) = others {
                tabs.push_sep();

                for (index, filter) in filters.iter().enumerate() {
                    push_filter(&mut tabs, index, filter)
                }

                tabs.push_sep();
                push_spec(&mut tabs, catch_all);
            }

            html! {
                <>
                    <div
                        id = "center_tabs_tile"
                        style = CENTER_STYLE
                        class = "h_scroll"
                    >
                        {tabs.render()}
                    </div>
                </>
            }
        }
    }
}
