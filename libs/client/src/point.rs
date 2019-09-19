//! Point-related stuff.

use crate::base::*;

pub use charts::point::{Point, Points};

use charts::point::PointVal;

impl<Key, Val> JsExt for Vec<Point<Key, Val>>
where
    Key: JsExt + fmt::Display,
    Val: JsExt + fmt::Display,
{
    fn as_js(self) -> JsVal {
        let list = js!(return []);
        for point in self {
            js!(@(no_return)
                @{&list}.push(@{point.as_js()})
            )
        }
        list
    }
}

impl<Key, Val> JsExt for Point<Key, Val>
where
    Key: JsExt + fmt::Display,
    Val: JsExt + fmt::Display,
{
    fn as_js(self) -> JsVal {
        let Point {
            key,
            vals: PointVal { filtered, rest },
        } = self;
        let point = js!(return { "x": @{key.as_js()}, "y": @{rest.as_js()}});
        for (index, val) in filtered.into_iter().enumerate() {
            js!(@(no_return)
                @{&point}[@{format!("y_{}", index)}] = @{val.as_js()};
            )
        }
        point
    }
}

impl JsExt for usize {
    fn as_js(self) -> JsVal {
        js!(return @{self.to_string()})
    }
}
impl JsExt for AllocDate {
    fn as_js(self) -> JsVal {
        AllocDate::as_js(&self)
    }
}
impl JsExt for SinceStart {
    fn as_js(self) -> JsVal {
        SinceStart::as_js(&self)
    }
}
