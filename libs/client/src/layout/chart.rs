//! Chart rendering.

prelude! {}

const border_size_px: usize = 2;

const chart_height_px: usize = 500;
const collapsed_chart_height_px: usize = 40;

const tiles_height_px: usize = 50;
const filter_toggles_height_px: usize = 30;

const menu_bg_color: &str = "#dbe9ff";

/// Abstract position information for a chart.
///
/// Only store whether the chart is first/last.
#[derive(Clone, Copy)]
pub struct ChartPos {
    is_first: bool,
    is_last: bool,
}
impl ChartPos {
    /// Constructor from a position `pos` in a list of length `len`.
    pub fn from_pos_and_len(pos: usize, len: usize) -> Self {
        Self {
            is_first: pos == 0,
            is_last: pos + 1 == len,
        }
    }
}

/// Renders a chart given a position.
pub fn render(model: &Model, chart: &Chart, pos: ChartPos) -> Html {
    define_style! {
        CONTAINER_STYLE = {
            block,
            margin(0%, 1%, 2%, 1%),
        };
        MAIN_CONTAINER_STYLE = {
            block,
            width(100%),
            overflow(hidden),
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
                {settings::render(model, chart)}
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

/// Renders a chart.
fn render_chart(_model: &Model, chart: &Chart) -> Html {
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

/// Tile rendering.
pub mod tiles {
    use super::*;

    /// Renders everything around a chart (top-menu and tile).
    pub fn render(model: &Model, chart: &Chart, pos: ChartPos) -> Html {
        const left_tile_width: usize = 20;
        const right_tile_width: usize = left_tile_width;
        const center_tile_width: usize = 100 - (left_tile_width + right_tile_width);

        define_style! {
            top_tiles! = {
                height({tiles_height_px} px),
                width(100%),
                bg({menu_bg_color}),
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

    /// Renders the top/center tabs of the tile.
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
                overflow(x: auto),
            };
        }

        let mut title = chart.title().to_string();
        if !chart.settings().display_mode().is_normal() {
            title.push_str(" | ");
            title.push_str(chart.settings().display_mode().desc());
        }

        html! {
            <center
                style = TITLE_CONTAINER
            >
                <div
                    style = TITLE_CELL
                >
                    {title}
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

    /// Renders the top/left tabs.
    pub fn render_left(model: &Model, chart: &Chart, pos: ChartPos) -> Html {
        let chart_uid = chart.uid();

        define_style! {
            BUTTON_CONTAINER = {
                extends(button_container),
                float(right),
            };
        }

        let move_up = layout::button::img::arrow_up(
            None,
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
            None,
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

    /// Renders the top/right tabs.
    pub fn render_right(model: &Model, chart: &Chart) -> Html {
        let chart_uid = chart.uid();

        define_style! {
            BUTTON_CONTAINER = {
                extends(button_container),
                float(left),
            };
        }

        let close_button = layout::button::img::close(
            None,
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
                None,
                "collapse_chart_button",
                Some(model.link.callback(move |_| {
                    msg::ChartSettingsMsg::toggle_visible::<msg::ChartsMsg>(chart_uid)
                })),
                "collapse this chart",
            )
        } else {
            layout::button::img::expand(
                None,
                "expand_chart_button",
                Some(model.link.callback(move |_| {
                    msg::ChartSettingsMsg::toggle_visible::<msg::ChartsMsg>(chart_uid)
                })),
                "expand this chart",
            )
        };

        let settings_button = layout::button::img::dots(
            None,
            "settings_chart_button",
            Some(
                model
                    .link
                    .callback(move |_| msg::ChartMsg::settings_toggle_visible(chart_uid)),
            ),
            if chart.is_settings_visible() {
                "collapse the chart's settings"
            } else {
                "expand the chart's settings"
            },
        );

        html! {
            <>
                <div
                    id = "settings_chart_button_container"
                    style = BUTTON_CONTAINER
                >
                    {settings_button}
                </div>
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

/// Chart settings rendering.
pub mod settings {
    use super::*;

    const LINE_HEIGHT_PX: usize = 40;

    /// Renders the chart settings.
    pub fn render(model: &Model, chart: &Chart) -> Html {
        define_style! {
            SETTINGS_STYLE = {
                border(bottom, 2 px, black),
                padding(10 px, 0 px),
                font_size(120%),
                bg({menu_bg_color}),
            };
        }

        if !chart.is_settings_visible() {
            return html! {};
        }

        html! {
            <div
                style = SETTINGS_STYLE
            >
                {layout::section_title("Settings")}
                <br/>

                { title(model, chart) }
                { options(model, chart) }
            </div>
        }
    }

    /// Renders the chart's title setting row.
    pub fn title(model: &Model, chart: &Chart) -> Html {
        let mut title = layout::table::TableRow::new_menu(true, html! { "title" })
            .black_sep()
            .height_px(LINE_HEIGHT_PX);
        title.push_single_value({
            let uid = chart.uid();
            layout::input::string_input(model, chart.title(), move |new_title_res| {
                new_title_res
                    .map(|new_title| msg::ChartSettingsMsg::change_title(uid, new_title))
                    .into()
            })
        });
        title.render()
    }

    /// Renders the chart's option settings.
    pub fn options(model: &Model, chart: &Chart) -> Html {
        let settings = chart.settings();

        if let Some(modes) = settings.legal_display_modes() {
            let chart_uid = chart.uid();
            let mut row = layout::table::TableRow::new_menu(false, html! { "display" })
                .black_sep()
                .height_px(LINE_HEIGHT_PX);
            let mut is_first = true;

            let select_mode = html! {
                <>
                    {for modes.into_iter().map(|mode| {
                        let radio = layout::input::radio(
                            mode == settings.display_mode(),
                            format!("chart_{}_{}", chart_uid, mode.to_uname()),
                            mode.desc(),
                            model.link.callback(move |_| {
                                msg::ChartSettingsMsg::set_display_mode::<msg::ChartsMsg>(
                                    chart_uid, mode
                                )
                            }),
                            model.link.callback(move |_| {
                                msg::ChartSettingsMsg::set_display_mode::<msg::ChartsMsg>(
                                    chart_uid, mode
                                )
                            }),
                            !is_first,
                        );
                        is_first = false;
                        radio
                    })}
                </>
            };
            row.push_single_value(select_mode);
            row.render()
        } else {
            html!()
        }
    }
}

/// Filter tabs (bottom) rendering.
pub mod filter_toggles {
    use super::*;
    use layout::tabs::{TabProps, Tabs};

    const tab_container_width: usize = 96;

    /// Renders the bottom filter tabs.
    pub fn render(model: &Model, chart: &Chart) -> Html {
        define_style! {
            TOGGLE_BAR = {
                width(100%),
                height(auto),
                // bg(red),
            };
            TOGGLE_CONTAINER = {
                width({tab_container_width}%),
                height(auto),
                margin(0, auto),
                overflow(x: auto),
                scrollbar_width(thin),
            };
            SUBCONTAINER = {
                height({filter_toggles_height_px} px),
                width(auto),
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
                .callback(move |_| msg::ChartMsg::filter_toggle_visible(chart_uid, uid))
        };

        macro_rules! render_line {
            (active: $active:expr =>
                $everything_opt:expr,
                $filters_opt:expr,
                $catch_all_opt:expr $(,)?
            ) => {{
                let mut tabs = None;
                if let Some(everything) = $everything_opt {
                    render_line!(@push(tabs) everything, $active)
                }
                if let Some(filters) = $filters_opt {
                    for (idx, filter) in filters.enumerate() {
                        if idx == 0 {
                            render_line!(@push_sep(tabs))
                        }
                        render_line!(@push(tabs) filter.spec(), $active);
                    }
                }
                if let Some(catch_all) = $catch_all_opt {
                    if !model.is_catch_all_empty() {
                        render_line!(@push_sep(tabs));
                        render_line!(@push(tabs) catch_all, $active);
                    }
                }

                if let Some(tabs) = tabs {
                    html! {
                        <div
                            id = if $active {
                                format!("active_filter_toggle_bar_for_chart{}", chart.uid())
                            } else {
                                format!("inactive_filter_toggle_bar_for_chart{}", chart.uid())
                            }
                            style = TOGGLE_BAR
                        >
                            <center
                                style = TOGGLE_CONTAINER
                                class = "h_scroll"
                            >
                                <div
                                    style = SUBCONTAINER
                                >
                                    {tabs.render()}
                                </div>
                            </center>
                        </div>
                    }
                } else {
                    html! {}
                }
            }};

            (@push($tabs:expr) $filter_spec:expr, $active:expr) => {{
                let tabs = $tabs.get_or_insert_with(Tabs::new);
                tabs.push_tab(
                    model,
                    $filter_spec.name(),
                    TabProps::new($filter_spec.color().to_string())
                        .set_dimmed(!$active)
                        .set_rev()
                        .set_round(!$active),
                    callback($filter_spec),
                );
            }};
            (@push_sep($tabs:expr)) => {
                if let Some(tabs) = $tabs.as_mut() {
                    tabs.push_sep()
                }
            };
        }

        let (e, f, c) = model.filters().active_filters_to_render(&is_active);
        let active = render_line!(active: true => e, f, c);
        let (e, f, c) = model.filters().inactive_filters_to_render(&is_active);
        let inactive = render_line!(active: false => e, f, c);

        html! {
            <> {active} {inactive} </>
        }
    }
}
