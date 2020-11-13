/*<LICENSE>
    This file is part of Memthol.

    Copyright (C) 2020 OCamlPro.

    Memthol is free software: you can redistribute it and/or modify
    it under the terms of the GNU General Public License as published by
    the Free Software Foundation, either version 3 of the License, or
    (at your option) any later version.

    Memthol is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
    GNU General Public License for more details.

    You should have received a copy of the GNU General Public License
    along with Memthol.  If not, see <https://www.gnu.org/licenses/>.
*/

//! Axis-related stuff.

// use crate::common::*;

pub use charts::chart::axis::{XAxis, YAxis};

// pub trait AxisExt {
//     fn chart_apply(&self, chart: &JsValue);
//     fn series_apply(&self, series: &JsValue, uid: filter::LineUid);
// }

// impl AxisExt for XAxis {
//     fn chart_apply(&self, chart: &JsValue) {
//         match self {
//             XAxis::Time => js!(@(no_return)
//                 var x_axis = @{chart}.xAxes.push(new am4charts.DateAxis());
//                 x_axis.dateFormats.setKey("second", "ss");
//                 x_axis.dateFormats.setKey("millisecond", "nnn");
//                 x_axis.periodChangeDateFormats.setKey("second", "[bold]HH:mm a[/]");
//                 x_axis.periodChangeDateFormats.setKey("minute", "[bold]HH:mm a[/]");
//                 x_axis.periodChangeDateFormats.setKey("hour", "[bold]HH:mm a[/]");
//                 x_axis.tooltipDateFormat = "[bold]HH:mm:ss.nnn[/]";
//                 x_axis.interpolationDuration = @{cst::charts::INTERP_DURATION};
//                 x_axis.rangeChangeDuration = @{cst::charts::INTERP_DURATION};
//                 x_axis.extraMax = 0.05;
//                 x_axis.renderer.ticks.template.disabled = false;
//                 x_axis.renderer.ticks.template.strokeOpacity = 1;
//                 // x_axis.renderer.ticks.template.stroke = am4core.color("#495C43");
//                 x_axis.renderer.ticks.template.strokeWidth = 2;
//                 x_axis.renderer.ticks.template.length = 10;
//             ),
//         }
//     }

//     fn series_apply(&self, series: &JsValue, _: filter::LineUid) {
//         js!(@(no_return)
//             @{series}.dataFields.dateX = "x";
//         )
//     }
// }

// impl AxisExt for YAxis {
//     fn chart_apply(&self, chart: &JsValue) {
//         match self {
//             YAxis::TotalSize => js!(@(no_return)
//                 var y_axis = @{chart}.yAxes.push(new am4charts.ValueAxis());
//                 y_axis.interpolationDuration = @{cst::charts::INTERP_DURATION};
//                 y_axis.rangeChangeDuration = @{cst::charts::INTERP_DURATION};
//             ),
//         }
//     }

//     fn series_apply(&self, series: &JsValue, uid: filter::LineUid) {
//         let y_name = uid.y_axis_key();
//         js!(@(no_return)
//             let series = @{series};
//             series.dataFields.valueY = @{&y_name};
//             series.tooltipText = @{format!("{{{}}}", y_name)};
//         )
//     }
// }
