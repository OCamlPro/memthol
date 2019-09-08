//! Constants of the memthol client.

/// Style classes.
pub mod class {
    /// Tab-related classes.
    pub mod tabs {
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
}

/// Style IDs.
pub mod id {
    /// Header id.
    pub static HEADER: &str = "header";
    /// Footer id.
    pub static FOOTER: &str = "footer";
}
