//! Chart rendering.

prelude! {}

const border_size_px: usize = 2;

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
                {tiles::render(model, chart)}
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
    use layout::tabs::{TabProps, Tabs};

    const tab_color: &str = "#accbff";
    const title_color: &str = "#00b1ff";

    pub fn render(model: &Model, chart: &Chart) -> Html {
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
            &format!("{} ({})", chart.spec().desc(), chart.uid()),
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
