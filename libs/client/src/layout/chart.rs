//! Chart rendering.

prelude! {}

const border_size_px: usize = 2;

const chart_height_px: usize = 500;
const collapsed_chart_height_px: usize = 40;

const tiles_height_px: usize = 50;
const filter_toggles_height_px: usize = 30;

#[derive(Clone, Copy)]
pub struct ChartPos {
    is_first: bool,
    is_last: bool,
}
impl ChartPos {
    pub fn from_pos_and_len(pos: usize, len: usize) -> Self {
        Self {
            is_first: pos == 0,
            is_last: pos + 1 == len,
        }
    }
}

pub fn render(model: &Model, chart: &Chart, pos: ChartPos) -> Html {
    define_style! {
        CONTAINER_STYLE = {
            block,
            margin(0%, 1%, 2%, 1%),
        };
        MAIN_CONTAINER_STYLE = {
            block,
            width(100%),
            border_radius(20 px),
            border({border_size_px} px, black),
            box_shadow(
                4 px,
                {-2} px,
                20 px,
                1 px,
                black,
            ),
        };
    }
    html! {
        <div
            id = chart.top_container_id()
            style = CONTAINER_STYLE
        >
            <div
                style = MAIN_CONTAINER_STYLE
            >
                {tiles::render(model, chart, pos)}
                {render_chart(model, chart)}
            </div>
            {filter_toggles::render(model, chart)}
        </div>
    }
}

define_style! {
    CHART_CONTAINER_STYLE = {
        width(100%),
        justify_content(center),
        display(flex),
    };
    COLLAPSED_CHART_CONTAINER_STYLE = {
        width(100%),
        justify_content(center),
        display(none),
    };

    generic_chart_style! = {
        width(99%),
        margin(0%, 0%, 1%, 0%),
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

pub fn render_chart(_model: &Model, chart: &Chart) -> Html {
    let visible = chart.is_visible();
    let canvas_id = chart.canvas_id();
    // let collapsed_canvas_id = chart.collapsed_canvas_id();
    let inner = if chart.has_canvas() {
        html! {
            <></>
        }
    } else {
        html! {
            <canvas
                id = canvas_id
                style = if visible {
                    &*CHART_STYLE
                } else {
                    &*HIDDEN_CHART_STYLE
                }
            />
        }
    };
    html! {
        <div
            id = chart.container_id()
            style = if visible {
                &*CHART_CONTAINER_STYLE
            } else {
                &*COLLAPSED_CHART_CONTAINER_STYLE
            }
        >
            {inner}
        </div>
    }
}

pub mod tiles {
    use super::*;

    pub fn render(model: &Model, chart: &Chart, pos: ChartPos) -> Html {
        const left_tile_width: usize = 20;
        const right_tile_width: usize = left_tile_width;
        const center_tile_width: usize = 100 - (left_tile_width + right_tile_width);

        define_style! {
            top_tiles! = {
                height({tiles_height_px} px),
                width(100%),
            };

            TOP_TILES = {
                extends(top_tiles),
                border(bottom, 2 px, black),
            };
            COLLAPSED_TOP_TILES = {
                extends(top_tiles),
            };

            LEFT_TILE = {
                height(100%),
                width({left_tile_width}%),
                float(left),
            };
            RIGHT_TILE = {
                height(100%),
                width({right_tile_width}%),
                float(left),
            };
            CENTER_TILE = {
                height(100%),
                width({center_tile_width}%),
                float(left),
            };
        }

        html! {
            <div
                id = "chart_top_tiles"
                style = if chart.is_visible() {
                    &*TOP_TILES
                } else {
                    &*COLLAPSED_TOP_TILES
                }
            >
                <div
                    id = "chart_top_left_tile"
                    style = LEFT_TILE
                >
                    {render_left(model, chart, pos)}
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
                    {render_right(model, chart)}
                </div>
            </div>
        }
    }

    pub fn render_center_tabs(_model: &Model, chart: &Chart) -> Html {
        // let chart_uid = chart.uid();

        define_style! {
            TITLE_CONTAINER = {
                height(100%),
                width(100%),
                font_size(170%),
                table,
            };
            TITLE_CELL = {
                vertical_align(middle),
                table cell,
            };
        }
        html! {
            <center
                style = TITLE_CONTAINER
            >
                <div
                    style = TITLE_CELL
                >
                    {chart.spec().desc()}
                </div>
            </center>
        }
    }

    define_style! {
        button_container! = {
            height(100%),
            width({tiles_height_px} px),
        };
    }

    pub fn render_left(model: &Model, chart: &Chart, pos: ChartPos) -> Html {
        let chart_uid = chart.uid();

        define_style! {
            BUTTON_CONTAINER = {
                extends(button_container),
                float(right),
            };
        }

        let move_up = layout::button::img::arrow_up(
            tiles_height_px,
            "move_chart_up",
            if pos.is_first {
                None
            } else {
                Some(
                    model
                        .link
                        .callback(move |_| msg::ChartsMsg::move_up(chart_uid)),
                )
            },
            "move this chart up",
        );
        let move_down = layout::button::img::arrow_down(
            tiles_height_px,
            "move_chart_down",
            if pos.is_last {
                None
            } else {
                Some(
                    model
                        .link
                        .callback(move |_| msg::ChartsMsg::move_down(chart_uid)),
                )
            },
            "move this chart down",
        );

        html! {
            <>
                <div
                    id = "move_up_chart_button_container"
                    style = BUTTON_CONTAINER
                >
                    {move_up}
                </div>
                <div
                    id = "move_down_chart_button_container"
                    style = BUTTON_CONTAINER
                >
                    {move_down}
                </div>
            </>
        }
    }

    pub fn render_right(model: &Model, chart: &Chart) -> Html {
        let chart_uid = chart.uid();

        define_style! {
            BUTTON_CONTAINER = {
                extends(button_container),
                float(left),
            };
        }

        let close_button = layout::button::img::close(
            tiles_height_px,
            "close_chart_button",
            Some(
                model
                    .link
                    .callback(move |_| msg::ChartsMsg::destroy(chart_uid)),
            ),
            "delete this chart",
        );
        let collapse_expand_button = if chart.is_visible() {
            layout::button::img::collapse(
                tiles_height_px,
                "collapse_chart_button",
                Some(
                    model
                        .link
                        .callback(move |_| msg::ChartsMsg::toggle_visible(chart_uid)),
                ),
                "collapse this chart",
            )
        } else {
            layout::button::img::expand(
                tiles_height_px,
                "expand_chart_button",
                Some(
                    model
                        .link
                        .callback(move |_| msg::ChartsMsg::toggle_visible(chart_uid)),
                ),
                "expand this chart",
            )
        };

        html! {
            <>
                <div
                    id = "collapse_expand_chart_button_container"
                    style = BUTTON_CONTAINER
                >
                    {collapse_expand_button}
                </div>
                <div
                    id = "close_chart_button_container"
                    style = BUTTON_CONTAINER
                >
                    {close_button}
                </div>
            </>
        }
    }
}

pub mod filter_toggles {
    use super::*;
    use layout::tabs::{TabProps, Tabs};

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
                id = format!("toggle_bar_for_chart{}", chart.uid())
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
