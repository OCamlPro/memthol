//! HTML builders.
#![allow(non_upper_case_globals)]

use crate::common::*;

pub mod foot;

define_style! {
    body_style! = {
        margin(0 px),
        height(min 100 vh),
        bg(white),
        fg(black),
        border_radius(20 px, 20 px, 0 px, 0 px),
    };

    COLLAPSED_FOOTER_BODY_STYLE = {
        extends(body_style),
        padding(2%, 0%, {foot::collapsed_height_px}px, 0%),
    };
    EXPANDED_FOOTER_BODY_STYLE = {
        extends(body_style),
        padding(2%, 0%, {foot::expanded_height_px}px, 0%),
    };
}

pub fn render(model: &Model) -> Html {
    html! {
        <>
            <div
                class = crate::style::class::FULL_BODY
                style = if model.footer.is_expanded() {
                    &*EXPANDED_FOOTER_BODY_STYLE
                } else {
                    &*COLLAPSED_FOOTER_BODY_STYLE
                }
            >
                <g
                    id = model.charts().dom_node_id()
                >
                    { model.charts.render(model) }
                </g>
            </div>
            { model.footer.render(model) }
        </>
    }
}

pub mod chart {
    use super::*;

    const border_size_px: usize = 2;

    const width: usize = 98;
    const chart_height_px: usize = 500;
    const collapsed_chart_height_px: usize = 40;

    const tiles_height_px: usize = 50;
    const filter_toggles_height_px: usize = 30;

    pub fn render(model: &Model, chart: &Chart) -> Html {
        info!(
            "rendering chart {}, visible: {}",
            chart.uid(),
            chart.is_visible()
        );
        html! {
            <g
                id = chart.container_id()
            >
                {tiles::render(model, chart)}
                {render_chart(model, chart)}
                {filter_toggles::render(model, chart)}
            </g>
        }
    }
    define_style! {
        CHART_CONTAINER_STYLE = {
            width(100%),
            justify_content(center),
            display(flex),
        };

        generic_chart_style! = {
            border_radius(20 px),
            border({border_size_px} px, black),
            width({width}%),
        };
        chart_style! = {
            extends(generic_chart_style),
            height({chart_height_px} px),
        };
        CHART_STYLE = {
            extends(chart_style),
        };
        HIDDEN_CHART_STYLE = {
            extends(chart_style),
            display(none),
        };

        collapsed_chart_style! = {
            extends(generic_chart_style),
            height({collapsed_chart_height_px} px),
            bg({"#777777"})
        };
        COLLAPSED_CHART_STYLE = {
            extends(collapsed_chart_style),
        };
        HIDDEN_COLLAPSED_CHART_STYLE = {
            extends(collapsed_chart_style),
            display(none),
        };
    }

    pub fn render_chart(_model: &Model, chart: &chart::Chart) -> Html {
        let visible = chart.is_visible();
        let canvas_id = chart.canvas_id();
        let collapsed_canvas_id = chart.collapsed_canvas_id();
        html! {
            <div
                id = "chart_canvas_container"
                style = CHART_CONTAINER_STYLE
            >
                <canvas
                    id = canvas_id
                    style = if visible {
                        &*CHART_STYLE
                    } else {
                        &*HIDDEN_CHART_STYLE
                    }
                />
                <canvas
                    id = collapsed_canvas_id
                    style = if visible {
                        &*HIDDEN_COLLAPSED_CHART_STYLE
                    } else {
                        &*COLLAPSED_CHART_STYLE
                    }
                />
            </div>
        }
    }

    pub mod tiles {
        use super::*;
        use tabs::{TabProps, Tabs};

        const tab_color: &str = "#accbff";
        const title_color: &str = "#00b1ff";

        pub fn render(model: &Model, chart: &Chart) -> Html {
            const left_tile_width: usize = 20;
            const right_tile_width: usize = left_tile_width;
            const center_tile_width: usize = 100 - (left_tile_width + right_tile_width);

            define_style! {
                TOP_TILES = {
                    height({tiles_height_px} px),
                    width(100%),
                };

                LEFT_TILE = {
                    height(100%),
                    width({left_tile_width}%),
                    float(left),
                    // bg(red)
                };
                RIGHT_TILE = {
                    height(100%),
                    width({right_tile_width}%),
                    float(left),
                    // bg(green)
                };
                CENTER_TILE = {
                    height(100%),
                    width({center_tile_width}%),
                    float(left),
                    // bg(blue)
                };
            }

            html! {
                <div
                    id = "chart_top_tiles"
                    style = TOP_TILES
                >
                    <div
                        id = "chart_top_left_tile"
                        style = LEFT_TILE
                    >
                        {render_left_tabs(model, chart)}
                    </div>
                    <div
                        id = "chart_top_center_tile"
                        style = CENTER_TILE
                    >
                        {render_center_tabs(model, chart)}
                    </div>
                    <div
                        id = "chart_top_right_tile"
                        style = RIGHT_TILE
                    >
                        {render_right_tabs(model, chart)}
                    </div>
                </div>
            }
        }

        pub fn render_center_tabs(model: &Model, chart: &Chart) -> Html {
            let chart_uid = chart.uid();
            let mut tabs = Tabs::new();

            tabs.push_tab(
                model,
                &chart.spec().desc(),
                TabProps::new(&*title_color),
                model
                    .link
                    .callback(move |_| msg::ChartsMsg::toggle_visible(chart_uid)),
            );

            define_style! {
                TABS_CONTAINER = {
                    height(100%),
                    font_size(150%),
                };
            }
            html! {
                <center
                    style = TABS_CONTAINER
                >
                    {tabs.render()}
                </center>
            }
        }

        pub fn render_left_tabs(model: &Model, chart: &Chart) -> Html {
            let chart_uid = chart.uid();
            let mut tabs = Tabs::new();

            tabs.push_tab(
                model,
                "move down",
                TabProps::new(&*tab_color),
                model
                    .link
                    .callback(move |_| msg::ChartsMsg::move_down(chart_uid)),
            );

            tabs.push_sep();

            tabs.push_tab(
                model,
                "move up",
                TabProps::new(&*tab_color),
                model
                    .link
                    .callback(move |_| msg::ChartsMsg::move_up(chart_uid)),
            );

            define_style! {
                TABS_CONTAINER = {
                    height(100%),
                    float(right),
                };
            }
            html! {
                <div
                    style = TABS_CONTAINER
                >
                    {tabs.render()}
                </div>
            }
        }

        pub fn render_right_tabs(model: &Model, chart: &Chart) -> Html {
            let chart_uid = chart.uid();

            let mut tabs = Tabs::new();
            tabs.push_tab(
                model,
                "remove",
                TabProps::new(&*tab_color),
                model
                    .link
                    .callback(move |_| msg::ChartsMsg::destroy(chart_uid)),
            );
            tabs.render()
        }
    }

    pub mod filter_toggles {
        use super::*;
        use tabs::{TabProps, Tabs};

        pub fn render(model: &Model, chart: &Chart) -> Html {
            define_style! {
                TOGGLE_BAR = {
                    width(100%),
                    height({filter_toggles_height_px} px),
                    text_align(center),
                    // bg(red),
                };
                TOGGLE_CONTAINER = {
                    height(100%),
                };
            }

            let chart_uid = chart.uid();

            info!("filter visiblity:");
            for (uid, visible) in chart.filter_visibility() {
                info!("- {}: {}", uid, visible)
            }

            let is_active = |spec: &filter::FilterSpec| {
                chart
                    .filter_visibility()
                    .get(&spec.uid())
                    .cloned()
                    .unwrap_or(false)
            };
            let callback = |spec: &filter::FilterSpec| {
                let uid = spec.uid();
                model
                    .link
                    .callback(move |_| msg::ChartsMsg::filter_toggle_visible(chart_uid, uid))
            };

            let mut tabs = Tabs::new();

            let (everything, other_opt) = model.filters().filters_to_render();

            if let Some((catch_all, others)) = other_opt {
                tabs.push_tab(
                    model,
                    everything.name(),
                    TabProps::new(everything.color().to_string())
                        .set_dimmed(!is_active(everything))
                        .set_rev(),
                    callback(everything),
                );

                tabs.push_sep();

                for filter in others {
                    let spec = filter.spec();
                    tabs.push_tab(
                        model,
                        spec.name(),
                        TabProps::new(spec.color().to_string())
                            .set_dimmed(!is_active(spec))
                            .set_rev(),
                        callback(spec),
                    );
                }

                tabs.push_sep();
                tabs.push_tab(
                    model,
                    catch_all.name(),
                    TabProps::new(catch_all.color().to_string())
                        .set_dimmed(!is_active(catch_all))
                        .set_rev(),
                    callback(catch_all),
                );
            }

            html! {
                <div
                    style = TOGGLE_BAR
                >
                    <center
                        style = TOGGLE_CONTAINER
                    >
                        {tabs.render()}
                    </center>
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

pub mod tabs {
    use super::*;

    pub const height: usize = 100;

    pub const sep_width: usize = 2;
    pub const max_width: usize = 30;
    pub const min_width: usize = 5;

    define_style! {
        TABS_ROW = {
            height(100%),
            table,
            table_layout(fixed),
        };

        raw_tab_style! = {
            height(100%),
            width(auto),
            table,
            text_align(center),
        };

        no_color_tab_style! = {
            extends(raw_tab_style),
            border_radius(5 px, 5 px, 0 px, 0 px),
            border(left, 1 px, black),
            border(right, 1 px, black),
            border(top, 1 px, black),
        };
        no_color_rev_tab_style! = {
            extends(raw_tab_style),
            border_radius(0 px, 0 px, 5 px, 5 px),
            border(left, 1 px, black),
            border(right, 1 px, black),
            border(bottom, 1 px, black),
        };

        raw_active_tab_style! = {
            pos(relative),
            z_index(650),
        };

        no_color_active_tab_style! = {
            extends(no_color_tab_style),
            extends(raw_active_tab_style),
        };
        no_color_active_rev_tab_style! = {
            extends(no_color_rev_tab_style),
            extends(raw_active_tab_style),
        };

        edited_tab_style! = {
            italic,
        };

        OUTTER_CELL_STYLE = {
            height(100%),
            width(min {min_width}%),
            table cell,
            pointer,
        };

        content_style! = {
            font_size(120%),
            font_outline(black),
            vertical_align(middle),
            table cell,
            padding(0%, 10 px),
            underline,
            pos(relative),
        };

        CONTENT_STYLE = {
            extends(content_style),
            fg(white),
        };
        DIMMED_CONTENT_STYLE = {
            extends(content_style),
            fg("#8a8a8a"),
        };

        SEP = {
            width(10%),
            height(100%),
            table cell,
        };
        END_SEP = {
            width(auto),
            height(100%),
            table cell,
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

    pub const NOT_ACTIVE: IsActive = IsActive::No;

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
            if(
                active,
                if(
                    rev,
                    extends(no_color_active_rev_tab_style),
                    else
                    extends(no_color_active_tab_style),
                ),
                else if(
                    rev,
                    extends(no_color_rev_tab_style),
                    else
                    extends(no_color_tab_style)
                ),
            ),

            if(
                props.edited,
                extends(edited_tab_style),
            ),

            if(
                props.rev,
                bg(gradient black to {&props.color}),
                else
                bg(gradient {&props.color} to black),
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
        style: &'static str,
        tabs: SVec<Html>,
    }

    impl Tabs {
        pub fn new() -> Self {
            Self {
                style: &*TABS_ROW,
                tabs: SVec::new(),
            }
        }

        fn raw_tab(props: &TabProps, onclick: OnClickAction, content: Html) -> Html {
            html! {
                <div
                    id = "filter_tab_cell"
                    style = OUTTER_CELL_STYLE
                    onclick = onclick
                >
                    <div
                        id = "filter_tab"
                        style = style(props)
                    >
                        <div
                            id = "filter_content"
                            style = if props.dimmed {
                                &*DIMMED_CONTENT_STYLE
                            } else {
                                &*CONTENT_STYLE
                            }
                        >
                            {content}
                        </div>
                    </div>
                </div>
            }
        }

        pub fn push_tab(
            &mut self,
            model: &Model,
            text: &str,
            props: TabProps,
            onclick: OnClickAction,
        ) {
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
            let mut res = html! {
                {Self::raw_tab(
                    &props, onclick, html! {
                        {if edited { format!("*{}*", text) } else { text.into() }}
                    }
                )}
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
                                html! { "<" }
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
                                html! { ">" }
                            )}
                        </>
                    }
                }
            }

            self.tabs.push(res)
        }

        pub fn push_sep(&mut self) {
            self.tabs.push(html! {
                <div id = "filter_tab_sep" style = SEP/>
            })
        }

        pub fn render(self) -> Html {
            html! {
                <div
                    style = self.style
                >
                    {for self.tabs.into_iter()}
                    <div id = "filter_tab_end_sep" style = END_SEP/>
                </div>
            }
        }
    }
}
