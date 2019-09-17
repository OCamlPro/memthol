//! Points.

use charts::point::{Point, PointVal};

use crate::base::*;

/// Trait extending `Vec<Point>` for JS conversion.
pub trait JsExt {
    /// Turns some points in a JS list.
    fn to_js(self) -> JsVal;
}

impl<Key, Val> JsExt for Vec<Point<Key, Val>>
where
    Key: JsExt,
    Val: JsExt,
{
    fn to_js(self) -> JsVal {
        let list = js!(return []);
        for Point {
            key,
            vals: PointVal { filtered, rest },
        } in self
        {
            let point = js!(return { "x": @{key.to_js()}, "y": @{rest.to_js()} });
            for (index, val) in filtered.into_iter().enumerate() {
                js!(@(no_return)
                    @{&point}[@{format!("y_{}", index)}] = @{val.to_js()};
                )
            }
            js!(@(no_return)
                @{&list}.push(@{point})
            )
        }
        list
    }
}

impl JsExt for usize {
    fn to_js(self) -> JsVal {
        js!(return @{self.to_string()})
    }
}
impl JsExt for AllocDate {
    fn to_js(self) -> JsVal {
        self.as_js()
    }
}
impl JsExt for SinceStart {
    fn to_js(self) -> JsVal {
        self.as_js()
    }
}
