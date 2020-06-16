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
        padding(2%, 0%, {foot::collapsed_height_wrt_full}%, 0%),
    };
    EXPANDED_FOOTER_BODY_STYLE = {
        extends(body_style),
        padding(2%, 0%, {foot::expanded_height_wrt_full}%, 0%),
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
                { model.charts.render(model) }
            </div>
            { model.footer.render(model) }
        </>
    }
}

pub mod chart {
    use super::*;

    const width: usize = 98;
    const height: usize = 100;
    const container_height_px: usize = 600;
    const hidden_container_height_px: usize = 42;

    const tiles_height_px: usize = 50;

    pub fn render(model: &Model, chart: &Chart) -> Html {
        html! {
            <>
                {tiles::render(model, chart)}
                {render_chart(model, chart.is_visible(), chart.canvas_id())}
            </>
        }
    }

    pub fn render_chart(_model: &Model, visible: bool, id: &str) -> Html {
        define_style! {
            chart_container_style! = {
                width(100%),
                text_align(center),
            };

            CHART_CONTAINER_STYLE = {
                extends(chart_container_style),
                height({container_height_px} px),
            };
            HIDDEN_CHART_CONTAINER_STYLE = {
                extends(chart_container_style),
                height({hidden_container_height_px} px),
            };

            chart_style! = {
                border_radius(20 px),
                border(2 px, black),
                width({width}%),
                height({height}%),
            };
            CHART_STYLE = {
                extends(chart_style),
            };
            HIDDEN_CHART_STYLE = {
                extends(chart_style),
            };
        }

        html! {
            <div
                id = "chart container"
                style = if visible {
                    &*CHART_CONTAINER_STYLE
                } else {
                    &*HIDDEN_CHART_CONTAINER_STYLE
                }
            >
                <canvas
                    id = if visible {
                        id.to_string()
                    } else {
                        format!("{}_inactive", id)
                    }
                    style = if visible {
                        &*CHART_STYLE
                    } else {
                        &*HIDDEN_CHART_STYLE
                    }
                />
            </div>
        }
    }

    pub mod tiles {
        use super::*;
        use tabs::{Tabs, NOT_ACTIVE};

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
                &&*title_color,
                NOT_ACTIVE,
                false,
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
                &&*tab_color,
                NOT_ACTIVE,
                false,
                model
                    .link
                    .callback(move |_| msg::ChartsMsg::move_up(chart_uid)),
            );

            tabs.push_sep();

            tabs.push_tab(
                model,
                "move up",
                &&*tab_color,
                NOT_ACTIVE,
                false,
                model
                    .link
                    .callback(move |_| msg::ChartsMsg::move_down(chart_uid)),
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
                &&*tab_color,
                NOT_ACTIVE,
                false,
                model
                    .link
                    .callback(move |_| msg::ChartsMsg::destroy(chart_uid)),
            );
            tabs.render()
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

        no_color_tab_style! = {
            height(100%),
            width(auto),
            table,
            text_align(center),
            border_radius(5 px, 5 px, 0 px, 0 px),
            border(left, 1 px, black),
            border(right, 1 px, black),
            border(top, 1 px, black),
        };
        no_color_active_tab_style! = {
            extends(no_color_tab_style),
            pos(relative),
            z_index(650),
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

        CONTENT_STYLE = {
            font_size(120%),
            fg(white),
            font_outline(black),
            vertical_align(middle),
            table cell,
            padding(0%, 10 px),
            underline,
            z_index(650),
            pos(relative),
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

    pub const NOT_ACTIVE: IsActive = IsActive::No;

    #[derive(Clone)]
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
        pub fn of_bool(active: bool) -> Self {
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

    pub fn style(color: &impl fmt::Display, active: bool, edited: bool) -> String {
        inline_css!(
            if(
                active,
                extends(no_color_active_tab_style),
                else extends(no_color_tab_style),
            ),
            if (
                active,
                box_shadow(4 px, {-2} px, 34 px, 7 px, {color}),
                else box_shadow(4 px, {-2} px, 20 px, 1 px, {color}),
            ),
            if(
                edited,
                extends(edited_tab_style),
            ),
            bg(gradient {color} to black),
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

        fn raw_tab(
            color: &impl fmt::Display,
            active: bool,
            edited: bool,
            onclick: OnClickAction,
            content: Html,
        ) -> Html {
            html! {
                <div
                    id = "filter_tab_cell"
                    style = OUTTER_CELL_STYLE
                    onclick = onclick
                >
                    <div
                        id = "filter_tab"
                        style = style(color, active, edited)
                    >
                        <div
                            id = "filter_content"
                            style = CONTENT_STYLE
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
            color: &impl fmt::Display,
            active: IsActive,
            edited: bool,
            onclick: OnClickAction,
        ) {
            let mut res = html! {
                {Self::raw_tab(
                    color, active.to_bool(), edited, onclick, html! {
                        {if edited { format!("*{}*", text) } else { text.into() }}
                    }
                )}
            };
            if let IsActive::YesWith {
                can_move_left,
                can_move_right,
                uid,
            } = active
            {
                if can_move_left {
                    res = html! {
                        <>
                            {Self::raw_tab(
                                color, false, false,
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
                                color, false, false,
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
