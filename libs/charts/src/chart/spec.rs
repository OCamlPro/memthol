//! Chart specification.

use super::*;

/// A chart specification, for the client.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChartSpec {
    /// UID,
    uid: uid::Chart,
    /// X-axis.
    x_axis: XAxis,
    /// Y-axis.
    y_axis: YAxis,
}
impl ChartSpec {
    /// Creates a new chart spec.
    pub fn new(x_axis: XAxis, y_axis: YAxis) -> Self {
        Self {
            uid: uid::Chart::fresh(),
            x_axis,
            y_axis,
        }
    }

    /// Description of a chart.
    pub fn desc(&self) -> String {
        format!("{} over {}", self.y_axis.desc(), self.x_axis.desc())
    }

    /// UID accessor.
    pub fn uid(&self) -> uid::Chart {
        self.uid
    }

    /// X-axis accessor.
    pub fn x_axis(&self) -> &XAxis {
        &self.x_axis
    }
    /// Y-axis accessor.
    pub fn y_axis(&self) -> &YAxis {
        &self.y_axis
    }
}
