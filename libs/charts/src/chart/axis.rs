//! Axis-related stuff.

use crate::base::*;

/// X-axis spec.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
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
}

/// Y-axis spec.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
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
