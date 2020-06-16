//! HTML builders.
#![allow(non_upper_case_globals)]

use crate::common::*;

pub mod input {
    use super::*;

    define_style! {
        input_style! = {
            width(85%),
            height(70%),
            border_radius(5 px),
            text_align(center),
            margin(none),
            padding(0%, 1%),
            border(none),
            bg({"#3a3a3a"}),
        };

        TEXT_INPUT_STYLE = {
            extends(input_style),
            fg(orange),
            font(code),
        };
        COLOR_INPUT_STYLE = {
            extends(input_style),
        };
    }

    pub fn text_input(value: &str, onchange: OnChangeAction) -> Html {
        html! {
            <input
                type = "text"
                id = "text_input"
                style = TEXT_INPUT_STYLE
                value = value
                onchange = onchange
            />
        }
    }

    fn parse_text_data(data: ChangeData) -> Res<String> {
        match data {
            yew::html::ChangeData::Value(txt) => Ok(txt),
            err @ yew::html::ChangeData::Select(_) | err @ yew::html::ChangeData::Files(_) => {
                bail!("unexpected text field update {:?}", err)
            }
        }
    }

    pub fn string_input(
        model: &Model,
        value: &str,
        msg: impl Fn(Res<String>) -> Msg + 'static,
    ) -> Html {
        text_input(
            value,
            model.link.callback(move |data| {
                msg(parse_text_data(data)
                    .map_err(err::Err::from)
                    .chain_err(|| "while parsing string value"))
            }),
        )
    }

    fn parse_usize_data(data: ChangeData) -> Res<usize> {
        use alloc_data::Parseable;
        parse_text_data(data).and_then(|txt| usize::parse(txt).map_err(|e| e.into()))
    }
    pub fn usize_input(
        model: &Model,
        value: usize,
        msg: impl Fn(Res<usize>) -> Msg + 'static,
    ) -> Html {
        text_input(
            &value.to_string(),
            model.link.callback(move |data| {
                msg(parse_usize_data(data)
                    .map_err(|e| err::Err::from(e))
                    .chain_err(|| "while parsing integer value"))
            }),
        )
    }

    pub fn color_input(value: &impl fmt::Display, onchange: OnChangeAction) -> Html {
        html! {
            <input
                type = "color"
                id = "color_input"
                style = COLOR_INPUT_STYLE
                value = value
                onchange = onchange
            />
        }
    }
}

pub const width_wrt_full: usize = 100;

pub const collapsed_height_wrt_full: usize = 4;
pub const expanded_height_wrt_full: usize = 40;

pub const expanded_tabs_height: usize = expanded_height_wrt_full / collapsed_height_wrt_full;
pub const expanded_menu_height: usize = 100 - expanded_tabs_height;

pub const collapsed_tabs_height: usize = 100;
pub const collapsed_menu_height: usize = 0;

pub const center_tile_width: usize = 70;
pub const left_tile_width: usize = (width_wrt_full - center_tile_width) / 2;
pub const right_tile_width: usize = left_tile_width;

define_style! {
    footer_style! = {
        font(default),
        fg(black),
        bg(white),
        fixed(bottom),
        z_index(600),
        width({width_wrt_full}%),
        // bg(yellow),
    };

    COLLAPSED_STYLE = {
        extends(footer_style),
        height({collapsed_height_wrt_full}%)
    };

    EXPANDED_STYLE = {
        extends(footer_style),
        height({expanded_height_wrt_full}%)
    };

    tabs_style! = {
        top,
        width({width_wrt_full}%),
    };
    COLLAPSED_TABS_STYLE = {
        extends(tabs_style),
        height({collapsed_tabs_height}%),
    };
    EXPANDED_TABS_STYLE = {
        extends(tabs_style),
        height({expanded_tabs_height}%),
    };

    menu_style! = {
        fg(white),
        bg(black),
        bottom,
        width({width_wrt_full}%),
        box_shadow(0 px, {-7} px, 50 px, 1 px, black),
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
        height({expanded_menu_height}%),
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
                            if let Ok(filter) = model.filters().get_spec(filter_uid) {
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
            fg({"#8dedff"}),
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
            font_size(120%),
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
                        ) = model.filters().get_filter(uid) {
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

        define_style! {
            link_box! = {
                width(70%),
                height(10%),
                bg(gradient {"#8e8e8e"} to black),
                border_radius(5 px),
                border(1 px, white),
                margin(auto),
                table,
            };
            LINK_BOX = {
                extends(link_box),
                pointer,
            };
            HIDDEN_LINK_BOX = {
                extends(link_box),
                visi(hidden),
            };
        }

        pub fn render(model: &Model, uid: filter::LineUid) -> Html {
            let edited = model.filters().edited();
            html! {
                <>
                    <br/><br/>
                    {save_button(model, edited)}
                    <br/>
                    {undo_button(model, edited)}
                    <br/><br/><br/><br/>
                    {add_subfilter_button(model, uid)}
                    <br/>
                </>
            }
        }

        pub fn button(id: &str, txt: &str, onclick: Option<OnClickAction>) -> Html {
            if let Some(onclick) = onclick {
                html! {
                    <div
                        id = id
                        style = LINK_BOX
                        onclick = onclick
                    >
                        <a
                            style = layout::tabs::CONTENT_STYLE
                        >
                            {txt}
                        </a>
                    </div>
                }
            } else {
                html! {
                    <div
                        id = id
                        style = HIDDEN_LINK_BOX
                    >
                    </div>
                }
            }
        }
        pub fn save_button(model: &Model, edited: bool) -> Html {
            let action = if edited {
                Some(model.link.callback(move |_| msg::FiltersMsg::save()))
            } else {
                None
            };
            button("save_button", "save all", action)
        }
        pub fn undo_button(model: &Model, edited: bool) -> Html {
            let action = if edited {
                Some(
                    model
                        .link
                        .callback(move |_| msg::to_server::FiltersMsg::revert()),
                )
            } else {
                None
            };
            button("undo_button", "undo all", action)
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

    define_style! {
        TITLE_STYLE = {
            font_size(150%),
            text_align(center),
            bold,
            underline,
        };
    }

    pub mod settings {
        use super::*;

        pub fn render(model: &Model, filter: &filter::FilterSpec) -> Html {
            html! {
                <>
                    <br/>
                    <div
                        id = "settings_title"
                        style = TITLE_STYLE
                    >
                        { "Settings" }
                    </div>

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
                input::text_input(
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
                input::color_input(
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
        use charts::filter::{sub::RawSubFilter, SizeFilter, SubFilter};

        pub fn render(model: &Model, filter: &filter::Filter) -> Html {
            // if !filter.has_sub_filters() {
            //     return html! {<></>};
            // }

            html! {
                <>
                    <br/>

                    <div
                        id = "subfilter_title"
                        style = TITLE_STYLE
                    >
                        { "Sub-Filters" }
                    </div>

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
                    onchange = model.link.callback(
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
                            onchange = model.link.callback(
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
                        table_row.push_value(input::usize_input(model, val, move |usize_res| {
                            msg(usize_res.map(|val| SizeFilter::Cmp { cmp, val }))
                        }))
                    }
                    SizeFilter::In { lb, ub } => {
                        let msg_fn = msg.clone();
                        let lb_html = input::usize_input(model, lb, move |usize_res| {
                            msg_fn(usize_res.map(|lb| SizeFilter::In { lb, ub }))
                        });
                        let ub_html = input::usize_input(model, ub, move |usize_res| {
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

        pub mod label {
            use super::*;
            use charts::filter::{
                label::{LabelPred, LabelSpec},
                LabelFilter,
            };

            pub fn render<Update>(
                table_row: &mut layout::table::TableRow,
                model: &Model,
                sub: &LabelFilter,
                msg: Update,
            ) where
                Update: Fn(Res<LabelFilter>) -> Msg + 'static + Clone,
            {
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
                            onchange = model.link.callback(
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
                    let inner = input::string_input(model, &value, {
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

            pub fn render<Update>(
                table_row: &mut layout::table::TableRow,
                model: &Model,
                sub: &LocFilter,
                msg: Update,
            ) where
                Update: Fn(Res<LocFilter>) -> Msg + 'static + Clone,
            {
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

                let selector = {
                    let selected = Some(sub.pred().clone());
                    let specs = sub.specs().clone();
                    let msg = msg.clone();
                    html! {
                        <Select<LocPred>
                            selected = selected
                            options = LocPred::all()
                            onchange = model.link.callback(
                                move |pred| msg(Ok(LocFilter::new(pred, specs.clone())))
                            )
                        />
                    }
                };
                table_row.push_selector(selector);

                for (idx, spec) in sub.specs().iter().enumerate() {
                    push_add_button!(idx);

                    let value = spec.to_string();
                    let inner = input::string_input(model, &value, {
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

    pub const width: usize = 100;
    pub const height: usize = 100;

    pub fn render(model: &Model, active: Option<filter::LineUid>) -> Html {
        html! {
            <>
                { tabs_left::render() }
                { tabs_center::render(model, active) }
                { tabs_right::render(model) }
            </>
        }
    }

    pub mod tabs_left {
        use super::*;

        define_style! {
            LEFT_STYLE = {
                extends(tile_style),
                width({left_tile_width}%),
                // bg(red),
            };
        }
        pub fn render() -> Html {
            html! {
                <div id = "left_tabs_tile" style = LEFT_STYLE />
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
                // bg(green),
            };
        }

        pub fn render(model: &Model) -> Html {
            let mut tabs = layout::tabs::Tabs::new();
            tabs.push_sep();
            tabs.push_tab(
                model,
                "add filter",
                &"white",
                layout::tabs::NOT_ACTIVE,
                false,
                model
                    .link
                    .callback(move |_| msg::to_server::FiltersMsg::request_new()),
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

        use layout::tabs::IsActive;

        define_style! {
            CENTER_STYLE = {
                extends(tile_style),
                width({center_tile_width}%),
                table,
                // bg(blue),
            };
            TABS_ROW = {
                height(100%),
                table,
                table_layout(fixed),
            };
        }

        pub fn render(model: &Model, active: Option<filter::LineUid>) -> Html {
            let (everything, others) = model.filters().filters_to_render();

            let is_active = |filter: &filter::FilterSpec| {
                IsActive::of_bool(active.map(|uid| uid == filter.uid()).unwrap_or(false))
            };
            let callback = |uid: filter::LineUid| {
                model
                    .link
                    .callback(move |_| msg::FooterMsg::toggle_tab(footer::FooterTab::filter(uid)))
            };

            let push_spec =
                |tabs: &mut layout::tabs::Tabs, filter: &filter::FilterSpec, edited: bool| {
                    tabs.push_tab(
                        model,
                        filter.name(),
                        filter.color(),
                        is_active(filter),
                        edited || filter.edited(),
                        callback(filter.uid()),
                    )
                };

            let push_filter =
                |tabs: &mut layout::tabs::Tabs, index: usize, filter: &filter::Filter| {
                    let active = is_active(filter.spec()).with_first_last_uid(|| {
                        (
                            0 < index,
                            index + 1 < model.filters().filters.len(),
                            filter.uid(),
                        )
                    });
                    tabs.push_tab(
                        model,
                        filter.spec().name(),
                        filter.spec().color(),
                        active,
                        filter.edited() || filter.spec().edited(),
                        callback(filter.spec().uid()),
                    )
                };

            let mut tabs = layout::tabs::Tabs::new();

            tabs.push_sep();
            push_spec(&mut tabs, everything, false);

            if let Some((catch_all, filters)) = others {
                tabs.push_sep();

                for (index, filter) in filters.iter().enumerate() {
                    push_filter(&mut tabs, index, filter)
                }

                tabs.push_sep();
                push_spec(&mut tabs, catch_all, false);
            }

            html! {
                <div
                    id = "center_tabs_tile"
                    style = CENTER_STYLE
                >
                    {tabs.render()}
                </div>
            }
        }
    }
}
