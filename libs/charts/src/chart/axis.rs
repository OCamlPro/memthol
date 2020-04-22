//! Axis-related stuff.

use crate::common::*;

/// X-axis spec.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, strum_macros::EnumIter)]
pub enum XAxis {
    /// Time.
    Time,
}
impl XAxis {
    /// Description of a x-axis.
    pub fn desc(&self) -> &'static str {
        match self {
            Self::Time => "time",
        }
    }

    /// The legal y-axes that can be combined with this x-axis.
    pub fn y_axes(&self) -> Vec<YAxis> {
        match self {
            Self::Time => vec![YAxis::TotalSize],
        }
    }

    /// A list of all the x-axes.
    pub fn all() -> Vec<XAxis> {
        use strum::IntoEnumIterator;
        Self::iter().collect()
    }
}

impl fmt::Display for XAxis {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        self.desc().fmt(fmt)
    }
}

impl Default for XAxis {
    fn default() -> Self {
        XAxis::Time
    }
}

/// Y-axis spec.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum YAxis {
    /// Total size.
    TotalSize,
    // /// Highest lifetime.
    // MaxLifetime,
}
impl YAxis {
    /// Description of a y-axis.
    pub fn desc(&self) -> &'static str {
        match self {
            Self::TotalSize => "total size",
            // Self::MaxLifetime => "highest lifetime",
        }
    }
}

impl fmt::Display for YAxis {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        self.desc().fmt(fmt)
    }
}
