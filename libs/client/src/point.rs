//! Point-related stuff.

pub use charts::point::{Point, Points, TimePoints};

// use crate::common::*;

// use charts::point::PointVal;

// impl<Key, Val> JsExt for Point<Key, Val>
// where
//     Key: JsExt + fmt::Display,
//     Val: JsExt + fmt::Display,
// {
//     fn as_js(self) -> JsValue {
//         let Point {
//             key,
//             vals: PointVal { map },
//         } = self;
//         let point = js!(return { "x": @{key.as_js()} });
//         for (uid, val) in map.into_iter() {
//             js!(@(no_return)
//                 @{&point}[@{uid.y_axis_key()}] = @{val.as_js()};
//             )
//         }
//         point
//     }
// }

// impl JsExt for usize {
//     fn as_js(self) -> JsValue {
//         js!(return @{self.to_string()})
//     }
// }
// impl JsExt for AllocDate {
//     fn as_js(self) -> JsValue {
//         js!(
//             return new Date(Date.UTC(
//                 @{self.date.year()},
//                 @{self.date.month0()},
//                 @{self.date.day()},
//                 @{self.date.hour()},
//                 @{self.date.minute()},
//                 @{self.date.second()},
//                 @{self.date.nanosecond() / 1_000_000},
//             ))
//         )
//     }
// }
// impl JsExt for SinceStart {
//     fn as_js(self) -> JsValue {
//         js!(return @{format!("{}.{}", self.as_secs(), self.subsec_millis())})
//     }
// }
