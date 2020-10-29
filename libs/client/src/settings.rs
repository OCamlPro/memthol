//! Settings part of the client.

/// Linear display mode.
#[derive(Debug, Clone, Copy)]
pub enum DisplayMode {
    /// Collapsed.
    Collapsed,
    /// Expanded for some *depth*.
    Expanded(u8),
}
impl DisplayMode {
    const MAX: u8 = 0;

    /// Constructor.
    pub fn new() -> Self {
        Self::Collapsed
    }

    /// Augments the display mode.
    pub fn inc(&mut self) {
        *self = match *self {
            Self::Collapsed => Self::Expanded(0),
            Self::Expanded(mut n) => {
                if n < Self::MAX {
                    n += 1
                }
                Self::Expanded(n)
            }
        }
    }

    /// Decreases the display mode.
    pub fn dec(&mut self) {
        *self = match *self {
            Self::Collapsed | Self::Expanded(0) => Self::Collapsed,
            Self::Expanded(mut n) => {
                debug_assert!(n > 0);
                n -= 1;
                Self::Expanded(n)
            }
        }
    }
}

/// Stores the settings state.
#[derive(Debug, Clone)]
pub struct Settings {
    /// Current display mode.
    display_mode: DisplayMode,
}

impl Settings {
    /// Constructor.
    pub fn new() -> Self {
        Self {
            display_mode: DisplayMode::new(),
        }
    }
}
