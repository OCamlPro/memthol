//! Axis-related stuff.

use crate::base::*;

pub use charts::chart::axis::{XAxis, YAxis};

pub trait AxisExt {
    fn chart_apply(&self, chart: &JsVal);
    fn series_apply(&self, series: &JsVal, index: Option<usize>);
}

impl AxisExt for XAxis {
    fn chart_apply(&self, chart: &JsVal) {
        match self {
            XAxis::Time => js!(@(no_return)
                var x_axis = @{chart}.xAxes.push(new am4charts.DateAxis());
                x_axis.dateFormats.setKey("second", "ss");
                x_axis.dateFormats.setKey("millisecond", "nnn");
                x_axis.periodChangeDateFormats.setKey("second", "[bold]h:mm a");
                x_axis.periodChangeDateFormats.setKey("minute", "[bold]h:mm a");
                x_axis.periodChangeDateFormats.setKey("hour", "[bold]h:mm a");
                x_axis.interpolationDuration = @{cst::charts::INTERP_DURATION};
                x_axis.rangeChangeDuration = @{cst::charts::INTERP_DURATION};
                x_axis.extraMax = 0.05;
            ),
        }
    }

    fn series_apply(&self, series: &JsVal, _: Option<usize>) {
        js!(@(no_return)
            @{series}.dataFields.dateX = "x";
        )
    }
}

impl AxisExt for YAxis {
    fn chart_apply(&self, chart: &JsVal) {
        match self {
            YAxis::TotalSize => js!(@(no_return)
                var y_axis = @{chart}.yAxes.push(new am4charts.ValueAxis());
                y_axis.interpolationDuration = @{cst::charts::INTERP_DURATION};
                y_axis.rangeChangeDuration = @{cst::charts::INTERP_DURATION};
            ),
        }
    }

    fn series_apply(&self, series: &JsVal, index: Option<usize>) {
        let y_name = if let Some(index) = index {
            format!("y_{}", index)
        } else {
            "y".into()
        };
        js!(@(no_return)
            let series = @{series};
            series.dataFields.valueY = @{&y_name};
            series.tooltipText = @{format!("{{{}}}", y_name)};
        )
    }
}
