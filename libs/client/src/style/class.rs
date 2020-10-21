//! CSS classes.

/// Class of the body of the UI.
pub static BODY: &str = "body";
/// Class of the `<div>` of the UI containing the header, the body, and the footer.
pub static FULL_BODY: &str = "body_header_footer";

/// Tab-related classes.
///
/// Tabs are implemented as a `<ul>` with `block` display.
pub mod tabs {
    /// Class for the `<ul>` around the tabs.
    pub static UL: &str = "tab_list";

    /// Classes for `<li>` tab containers.
    pub mod li {
        /// Left tab.
        pub static LEFT: &str = "li_left";
        /// Right tab.
        pub static RIGHT: &str = "li_right";
        /// Center tab.
        pub static CENTER: &str = "li_center";

        /// Class for a left/right `<li>` tab container.
        pub fn get(float_left: bool) -> &'static str {
            if float_left {
                LEFT
            } else {
                RIGHT
            }
        }

        pub fn get_center() -> &'static str {
            CENTER
        }
    }

    /// Active tab.
    pub static ACTIVE: &str = "tab_active";
    /// Inactive tab.
    pub static INACTIVE: &str = "tab_inactive";

    /// Class for an (in)active tab.
    pub fn get(is_active: bool) -> &'static str {
        if is_active {
            ACTIVE
        } else {
            INACTIVE
        }
    }

    /// Class/style pair for an (in)active filter tab.
    pub fn footer_get(is_active: bool, color: &charts::color::Color) -> (&'static str, String) {
        if is_active {
            (
                ACTIVE,
                format!("background-image: linear-gradient({}, black);", color),
            )
        } else {
            (
                INACTIVE,
                format!("background-image: linear-gradient(black, {});", color),
            )
        }
    }

    /// Class of a tab separator,
    pub static SEP: &str = "tab_sep";
}

/// Button-related classes.
pub mod button {
    /// Close button.
    pub static CLOSE: &str = "close_button";
    /// Add button.
    pub static ADD: &str = "add_button";
    /// Move down button.
    pub static MOVE_DOWN: &str = "move_down_button";
    /// Move up button.
    pub static MOVE_UP: &str = "move_up_button";
    /// Expand button.
    pub static EXPAND: &str = "expand_button";
    /// Collapse button.
    pub static COLLAPSE: &str = "collapse_button";
    /// Refresh button.
    pub static REFRESH: &str = "refresh_button";
    /// Undo button.
    pub static UNDO: &str = "undo_button";
    /// Save button.
    pub static SAVE: &str = "save_button";
    /// Inactive tickbox button.
    pub static INACTIVE_TICK: &str = "inactive_tick_button";
    /// Active tickbox button.
    pub static ACTIVE_TICK: &str = "active_tick_button";
}

/// Chart-related classes.
pub mod chart {
    /// Header class.
    pub static HEADER: &str = "chart_header";
    /// Chart container class.
    pub static CONTAINER: &str = "memthol_chart_container";
    /// Class of a visible chart.
    pub static VISIBLE: &str = "chart_style";
    /// Class of a hidden chart.
    pub static HIDDEN: &str = "hidden_chart_style";
    /// Prefix for the class of a chart.
    ///
    /// Needs to be completed with the chart's uid.
    pub static PREFIX: &str = "memthol_chart_html_id_";
    pub static CANVAS_PREFIX: &str = "memthol_chart_canvas_html_id_";
    /// Chart axis selection class.
    pub static SELECT_AXIS: &str = "select_axis";

    /// Id of an actual chart.
    pub fn class(uid: base::uid::Chart) -> String {
        format!("{}{}", PREFIX, uid)
    }

    /// Id of a chart canvas.
    pub fn canvas(uid: base::uid::Chart) -> String {
        format!("{}{}", CANVAS_PREFIX, uid)
    }

    /// Class of an amchart depending on its visibility.
    pub fn style(visible: bool) -> &'static str {
        if visible {
            VISIBLE
        } else {
            HIDDEN
        }
    }

    /// Chart canvas stuff.
    pub mod canvas {
        /// Class of a chart canvas.
        static CLASS: &str = "chart_canvas_style";

        /// Class of a chart canvas.
        pub fn style() -> &'static str {
            CLASS
        }
    }

    /// Active filter button.
    pub static ACTIVE: &str = "chart_filter_button_active";
    /// Inactive filter button.
    pub static INACTIVE: &str = "chart_filter_button_inactive";

    /// Class/style pair for an (in)active filter button.
    pub fn filter_button_get(
        is_active: bool,
        color: &charts::color::Color,
    ) -> (&'static str, String) {
        if is_active {
            (ACTIVE, format!("background-color: {};", color))
        } else {
            (INACTIVE, format!("background-color: {};", color))
        }
    }
}

/// Filter-related classes.
pub mod filter {
    /// Filter buttons class.
    pub static BUTTONS_LEFT: &str = "filter_buttons_left";
    /// Filter buttons class.
    pub static BUTTONS_RIGHT: &str = "filter_buttons_right";
    /// Filter line class.
    pub static LINE: &str = "filter_ul";
    /// Separator.
    pub static SEP: &str = "separator";

    /// Filter line classes.
    pub mod line {
        /// Class of a filter line cell.
        pub static CELL: &str = "filter_li";
        /// Class of a filter setting cell.
        pub static SETTINGS_CELL: &str = "filter_setting";
        /// Class of a filter section.
        pub static SECTION_CELL: &str = "filter_section";
        /// Class of a (allocation) property cell.
        pub static PROP_CELL: &str = "filter_prop";
        /// Class of a comparator cell.
        pub static CMP_CELL: &str = "filter_cmp";
        /// Class of a value cell.
        pub static VAL_CELL: &str = "filter_val";
        /// Class of a settings button.
        pub static SETTINGS_BUTTON: &str = "filter_settings_button";
        /// Class of a settings value cell.
        pub static SETTINGS_VALUE_CELL: &str = "filter_settings_value";
        /// Class of a label insertion element.
        pub static ADD_LABEL: &str = "filter_add_label";
        /// Class for a label value.
        pub static TEXT_VALUE: &str = "filter_value";
    }
}

/// Footer-related classes.
pub mod footer {
    /// Display window.
    pub static DISPLAY: &str = "footer_display";
    /// Class of the tabs of the footer.
    pub static TABS: &str = "footer_tabs";

    /// Tab-related classes.
    pub mod tabs {
        /// Class of the left part of the tabs.
        pub static LEFT: &str = "footer_tabs_left";
        /// Class of the right part of the tabs.
        pub static RIGHT: &str = "footer_tabs_right";
        /// Class of the center part of the tabs.
        pub static CENTER: &str = "footer_tabs_center";
    }
}
