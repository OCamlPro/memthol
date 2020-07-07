//! HTML builders.
#![allow(non_upper_case_globals)]

prelude! {}

pub const width_wrt_full: usize = 100;

pub const collapsed_height_px: usize = 50;
pub const expanded_height_px: usize = 500;

pub const expanded_tabs_height_px: usize = collapsed_height_px;
pub const expanded_menu_height_px: usize = expanded_height_px - expanded_tabs_height_px;

pub const collapsed_tabs_height_px: usize = collapsed_height_px;
pub const collapsed_menu_height_px: usize = collapsed_height_px - collapsed_tabs_height_px;

pub const center_tile_width: usize = 70;
pub const left_tile_width: usize = (width_wrt_full - center_tile_width) / 2;
pub const right_tile_width: usize = left_tile_width;

define_style! {
    footer_style! = {
        font(default),
        fg(black),
        bg(transparent),
        fixed(bottom),
        z_index(600),
        width({width_wrt_full}%),
        // bg(yellow),
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
            let edited = model.footer_filters().edited();
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
            let mut table_row = table::TableRow::new_menu(true, html! { "name" });
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
            let mut table_row = table::TableRow::new_menu(false, html! { "color" });
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
            let mut table_row = table::TableRow::new_menu(is_first, key);
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
                table_row: &mut table::TableRow,
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
                        table_row.push_value(input::u32_input(model, val, move |usize_res| {
                            msg(usize_res.map(|val| SizeFilter::Cmp { cmp, val }))
                        }))
                    }
                    SizeFilter::In { lb, ub } => {
                        let msg_fn = msg.clone();
                        let lb_html = input::u32_input(model, lb, move |usize_res| {
                            msg_fn(usize_res.map(|lb| SizeFilter::In { lb, ub }))
                        });
                        let ub_html = input::u32_input(model, ub, move |usize_res| {
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
                table_row: &mut table::TableRow,
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
                table_row: &mut table::TableRow,
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
                            on_change = model.link.callback(
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
    use layout::tabs::TabProps;

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
                TabProps::new_footer("white"),
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
            let (everything, others) = model.footer_filters().filters_to_render();

            let is_active = |filter: &filter::FilterSpec| {
                active.map(|uid| uid == filter.uid()).unwrap_or(false)
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
                        TabProps::new_footer(filter.color().to_string())
                            .set_active(is_active(filter))
                            .set_edited(edited || filter.edited()),
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
                            .set_edited(filter.edited() || filter.spec().edited()),
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

pub mod table {
    use super::*;

    pub const LEFT_COL_WIDTH: usize = 15;
    pub const RIGHT_COL_WIDTH: usize = 100 - LEFT_COL_WIDTH - 1;

    define_style! {
        row_style! = {
            width(100%),
            height(10%),
            block,
        };
        FIRST_ROW_STYLE = {
            extends(row_style),
        };
        NON_FIRST_ROW_STYLE = {
            extends(row_style),
            border(top, 2 px, white),
        };

        LEFT_COL = {
            block,
            height(100%),
            width({LEFT_COL_WIDTH}%),
            text_align(center),
            float(left),
            padding(none),
            margin(none),
            border(right, 2 px, white),
        };
        RIGHT_COL = {
            block,
            height(100%),
            width({RIGHT_COL_WIDTH}%),
            float(left),
            padding(none),
            margin(none),
            overflow(auto),
        };
        table_cell_style! = {
            table cell,
            vertical_align(middle),
        };
    }

    define_style! {
        cell_style! = {
            height(100%),
            display(table),
            float(left),
            text_align(center),
        };
        VALUE_CONTAINER_STYLE = {
            extends(cell_style),
            width(min 10%),
        };
        TINY_VALUE_CONTAINER_STYLE = {
            extends(cell_style),
            width(3%),
        };
        SINGLE_VALUE_CONTAINER_STYLE = {
            extends(cell_style),
            width(100%),
        };
        SEP_CONTAINER_STYLE = {
            extends(cell_style),
            width(2%),
        };
        SELECTOR_CONTAINER_STYLE = {
            extends(cell_style),
            width(10%),
        };
        SINGLE_VALUE_WITH_SELECTOR_CONTAINER_STYLE = {
            extends(cell_style),
            width(90%)
        };

        value_cell_style! = {
            extends(table_cell_style),
            height(100%),
            width(auto),
            // width(min 15%),
        };
        CELL_STYLE = {
            extends(value_cell_style),
        };
        VALUE_STYLE = {
            extends(value_cell_style),
            // margin(0%, 1%),
        };
        SEP_STYLE = {
            extends(value_cell_style),
            // padding(0%, 1%),
            // width(5%),
            font(code),
        };
        ADD_STYLE = {
            // extends(value_cell_style),
            // width(5%),
            font(code),
            pointer,
        };

        SELECTOR_STYLE = {
            extends(value_cell_style),
            // margin(0%, 1%),
        };

        BUTTON_STYLE = {
            font(code),
            pointer,
        };
    }

    pub struct TableRow {
        style: &'static str,
        lft_style: &'static str,
        lft: Html,
        rgt_style: &'static str,
        rgt: SVec<Html>,
    }

    impl TableRow {
        fn new(
            is_first: bool,
            lft_style: &'static str,
            lft: Html,
            rgt_style: &'static str,
        ) -> Self {
            let style = if is_first {
                &*FIRST_ROW_STYLE
            } else {
                &*NON_FIRST_ROW_STYLE
            };
            let lft = html! {
                <div
                    style = SINGLE_VALUE_CONTAINER_STYLE
                >
                    {Self::new_cell(lft)}
                </div>
            };
            Self {
                style,
                lft_style,
                lft,
                rgt_style,
                rgt: SVec::new(),
            }
        }

        pub fn new_menu(is_first: bool, lft: Html) -> Self {
            Self::new(is_first, &*LEFT_COL, lft, &*RIGHT_COL)
        }

        pub fn render(self) -> Html {
            html! {
                <div
                    style = self.style
                >
                    <div
                        style = self.lft_style
                    >
                        {self.lft}
                    </div>
                    <div
                        style = self.rgt_style
                    >
                        { for self.rgt.into_iter() }
                    </div>
                </div>
            }
        }

        fn new_cell(inner: Html) -> Html {
            html! {
                <div
                    style = CELL_STYLE
                >
                    {inner}
                </div>
            }
        }

        pub fn push_selector(&mut self, selector: Html) {
            self.rgt.push(html! {
                <div
                    style = SELECTOR_CONTAINER_STYLE
                >
                    {Self::new_cell(selector)}
                </div>
            })
        }
        pub fn push_sep(&mut self, sep: Html) {
            self.rgt.push(html! {
                <div
                    style = SEP_CONTAINER_STYLE
                >
                    {Self::new_cell(sep)}
                </div>
            })
        }
        pub fn push_value(&mut self, value: Html) {
            self.rgt.push(html! {
                <div
                    style = VALUE_CONTAINER_STYLE
                >
                    {Self::new_cell(value)}
                </div>
            })
        }
        pub fn push_tiny_value(&mut self, value: Html) {
            self.rgt.push(html! {
                <div
                    style = TINY_VALUE_CONTAINER_STYLE
                >
                    {Self::new_cell(value)}
                </div>
            })
        }
        pub fn push_single_value(&mut self, value: Html) {
            self.rgt.push(html! {
                <div
                    style = SINGLE_VALUE_CONTAINER_STYLE
                >
                    {Self::new_cell(value)}
                </div>
            })
        }

        pub fn push_single_selector_and_value(&mut self, selector: Html, value: Html) {
            self.rgt.push(html! {
                <div
                    style = SELECTOR_CONTAINER_STYLE
                >
                    {Self::new_cell(selector)}
                </div>
            });
            self.rgt.push(html! {
                <div
                    style = VALUE_CONTAINER_STYLE
                >
                    {Self::new_cell(value)}
                </div>
            })
        }

        pub fn push_button(&mut self, txt: &str, action: OnClickAction) {
            self.rgt.push(html! {
                <div
                    style = SEP_CONTAINER_STYLE
                >
                    {Self::new_cell(html! {
                        <div
                            style = BUTTON_STYLE
                            onclick = action
                        >
                            {txt}
                        </div>
                    })}
                </div>
            })
        }
    }
}

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
        use alloc::parser::Parseable;
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

    fn parse_u32_data(data: ChangeData) -> Res<u32> {
        use alloc::parser::Parseable;
        parse_text_data(data).and_then(|txt| u32::parse(txt).map_err(|e| e.into()))
    }
    pub fn u32_input(model: &Model, value: u32, msg: impl Fn(Res<u32>) -> Msg + 'static) -> Html {
        text_input(
            &value.to_string(),
            model.link.callback(move |data| {
                msg(parse_u32_data(data)
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
