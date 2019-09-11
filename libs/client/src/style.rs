//! Constants of the memthol client.

/// Style classes.
pub mod class {

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

            /// Class for a left/right `<li>` tab container.
            pub fn get(float_left: bool) -> &'static str {
                if float_left {
                    LEFT
                } else {
                    RIGHT
                }
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
    }

    /// Chart-related classes.
    pub mod chart {
        /// Header class.
        pub static HEADER: &str = "chart_header";
    }

    /// Filter-related classes.
    pub mod filter {
        /// Filter buttons class.
        pub static BUTTONS: &str = "filter_buttons";
        /// Filter line class.
        pub static LINE: &str = "filter_ul";

        /// Class for labels.
        pub static VALUE: &str = "filter_value";

        /// Filter line classes.
        pub mod line {
            /// Class of a filter line cell.
            pub static CELL: &str = "filter_li";
            /// Class of a (allocation) property cell.
            pub static PROP_CELL: &str = "filter_prop";
            /// Class of a comparator cell.
            pub static CMP_CELL: &str = "filter_cmp";
            /// Class of a value cell.
            pub static VAL_CELL: &str = "filter_val";
            /// Class of a label insertion element.
            pub static ADD_LABEL: &str = "filter_add_label";
        }
    }

    /// Footer-related classes.
    pub mod footer {
        /// Display window.
        pub static DISPLAY: &str = "footer_display";
    }

}

/// Style IDs.
pub mod id {
    /// Header id.
    pub static HEADER: &str = "header";
    /// Footer id.
    pub static FOOTER: &str = "footer";

    /// Footer tab container.
    pub static FOOTER_TABS: &str = "footer_tabs";
}
