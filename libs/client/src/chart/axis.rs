//! Axes-related stuff.

use crate::base::*;

use stdweb::js;

/// Linear or logarithmic axis.
#[derive(Clone, Copy, Debug)]
pub enum Type {
    /// Linear.
    Lin,
    /// Logarithmic.
    Log,
    /// Time.
    Time,
}
impl Default for Type {
    fn default() -> Self {
        Type::Lin
    }
}
impl Type {
    /// JS version of an axis type.
    pub fn as_js(&self) -> Value {
        let js = match self {
            Type::Lin => js! { return "linear" },
            Type::Log => js! { return "logarithmic" },
            Type::Time => js! { return "time" },
        };
        js
    }
}

/// Position of the axis.
#[derive(Clone, Copy, Debug)]
pub enum Position {
    /// Top.
    Top,
    /// Bottom.
    Bot,
    /// Left.
    Lft,
    /// Right.
    Rgt,
}
impl Position {
    /// Default position for a x-axis.
    pub fn default_x() -> Self {
        Position::Bot
    }
    /// Default position for a y-axis.
    pub fn default_y() -> Self {
        Position::Rgt
    }

    /// JS version of a position.
    pub fn as_js(&self) -> Value {
        let js = match self {
            Position::Top => js! { return "top" },
            Position::Bot => js! { return "bottom" },
            Position::Lft => js! { return "left" },
            Position::Rgt => js! { return "right" },
        };
        js
    }
}

/// A trait requiring partial comparison, clone and subtraction.
pub trait CloneSubOrd:
    std::cmp::PartialOrd + std::ops::Sub<Self, Output = Self> + Clone + fmt::Display
{
}
impl CloneSubOrd for usize {}
impl CloneSubOrd for AllocDate {}

/// A range for an axis.
#[derive(Clone, Debug)]
pub enum Range<Val> {
    /// No range, autoscale.
    None,
    /// Min-only range.
    Min(Val),
    /// Max-only range.
    Max(Val),
    /// Min/max range.
    MinMax { min: Val, max: Val },
    /// Sliding range, lower bound of the range is maximum value on the axis minus this value.
    Sliding(Val),
}
impl<Val> Range<Val> {
    /// Min-only constructor.
    pub fn min(value: Val) -> Self {
        Range::Min(value)
    }
    /// Max-only constructor.
    pub fn max(value: Val) -> Self {
        Range::Max(value)
    }
    /// Min/max constructor.
    pub fn min_max(min: Val, max: Val) -> Self {
        Range::MinMax { min, max }
    }
    /// Sliding range constructor.
    pub fn sliding(size: Val) -> Self {
        Range::Sliding(size)
    }
}
impl<Val: CloneSubOrd> Range<Val> {
    /// Checks that a value is in a range.
    pub fn contains(&self, val: &Val, min: &Val, max: &Val) -> bool {
        use Range::*;
        match self {
            None => true,
            Min(min) => min <= val,
            Max(max) => val <= max,
            MinMax { min, max } => min <= val && val <= max,
            Sliding(size) => {
                if &(max.clone() - min.clone()) >= size {
                    let min = max.clone() - size.clone();
                    &min <= val
                } else {
                    min <= val
                }
            }
        }
    }

    /// Applies the range to a `chart.js` axis.
    pub fn apply_to_axis(&self, min: &Val, max: &Val, axis: &Value) {
        use Range::*;
        match self {
            None => (),
            Min(min) => js!(@(no_return)
                @{axis}.ticks.min = @{min.to_string()};
            ),
            Max(max) => js!(@(no_return)
                @{axis}.ticks.max = @{max.to_string()};
            ),
            MinMax { min, max } => js!(@(no_return)
                @{axis}.ticks.min = @{min.to_string()};
                @{axis}.ticks.max = @{max.to_string()};
            ),
            Sliding(size) => {
                let min = if &(max.clone() - min.clone()) >= size {
                    max.clone() - size.clone()
                } else {
                    min.clone()
                };
                js!(@(no_return)
                    @{axis}.ticks.min = @{min.to_string()};
                    @{axis}.ticks.max = @{max.to_string()};
                )
            }
        }
    }
}

/// Ticks of an axis.
#[derive(Clone, Debug)]
pub struct Ticks<Val> {
    /// Range of the ticks.
    range: Range<Val>,
    /// Tick callback, a function that filter-maps each tick.
    callback: Value,
}
impl<Val> Default for Ticks<Val> {
    fn default() -> Self {
        Self {
            range: Range::None,
            callback: Self::nop_callback(),
        }
    }
}
impl<Val> Ticks<Val> {
    // /// Sets the range of the ticks.
    // pub fn set_range(&mut self, range: Range<Val>) {
    //     self.range = range
    // }

    /// Tick callback which filters out non-integer ticks.
    pub fn only_ints_callback() -> Value {
        let js = js! {
            return function(value, index, values) {
                if (Math.floor(value) === value) {
                    return value;
                }
            }
        };
        js
    }
    /// Tick callback that does nothing.
    pub fn nop_callback() -> Value {
        let js = js! {
            return function(value, index, values) {
                return value
            }
        };
        js
    }
    /// Tick callback for bytes.
    pub fn bytes_callback() -> Value {
        let js = js! {
            return function(val, index, values) {
                let value = @{Self::only_ints_callback()}(val, index, values);
                if (value === undefined) {
                    return undefined
                } else if (value >= 1_000) {
                    let f_value = parseFloat(value);
                    if (value >= 1_000_000) {
                        if (value >= 1_000_000_000) {
                            return "" + (f_value / 1_000_000_000) + "GB"
                        } else {
                            return "" + (f_value / 1_000_000) + "MB"
                        }
                    } else {
                        return "" + (f_value / 1_000) + "KB"
                    }
                } else {
                    return value
                }
            }
        };
        js
    }

    /// Ticks of the axis.
    pub fn as_js(&self) -> Value {
        let ticks = js! {
            return {
                callback: @{&self.callback}
            }
        };
        ticks
    }
}

/// An axis.
#[derive(Clone, Debug)]
pub struct Axis<Val> {
    /// Type of the axis (linear, logarithmic).
    typ: Type,
    /// Position of the axis (top, bottom, left, right).
    pos: Position,
    /// Configuration of the ticks.
    ticks: Ticks<Val>,
    /// Label of the axis.
    label: String,
}
impl<Val> Axis<Val> {
    /// Default x-axis.
    pub fn default_x<Str>(label: Str) -> Self
    where
        Str: Into<String>,
    {
        let label = label.into();
        Self {
            typ: Type::default(),
            pos: Position::default_x(),
            ticks: Ticks::default(),
            label,
        }
    }
    /// Default y-axis.
    pub fn default_y<Str>(label: Str) -> Self
    where
        Str: Into<String>,
    {
        let label = label.into();
        Self {
            typ: Type::default(),
            pos: Position::default_y(),
            ticks: Ticks::default(),
            label,
        }
    }

    // /// Sets the range for the scale.
    // pub fn set_range(&mut self, range: Range<Val>) {
    //     self.ticks.set_range(range)
    // }

    /// Retrieves the range of the scale.
    pub fn range(&self) -> &Range<Val> {
        &self.ticks.range
    }

    /// Makes the axis logarithmic.
    pub fn log_scale(&mut self) {
        self.typ = Type::Log
    }
    /// Makes the axis linear.
    pub fn linear_scale(&mut self) {
        self.typ = Type::Lin
    }
    /// Makes the axis a time axis.
    pub fn time_scale(&mut self) {
        self.typ = Type::Time
    }

    // /// JS version of the axis.
    // fn as_js(&self) -> Value {
    //     let js = js! {
    //         return {
    //             type: @{self.typ.as_js()},
    //             time: {
    //                 unit: "millisecond",
    //                 tooltipFormat: "h:mm:ss a",
    //             },
    //             position: @{self.pos.as_js()},
    //             ticks: @{self.ticks.as_js()},
    //             gridLines: {
    //                 zeroLineColor: "black",
    //                 zeroLineWidth: 2,
    //             },
    //             scaleLabel: {
    //                 display: true,
    //                 labelString: @{&self.label},
    //             },
    //         }
    //     };
    //     js
    // }
}

pub trait XAxis: Default + Clone + fmt::Debug {
    type Value: Ord;
    type LiveInfo;

    const clear_map: bool;

    fn axis(&self) -> &Axis<Self::Value>;
    fn axis_mut(&mut self) -> &mut Axis<Self::Value>;

    // fn set_range(&mut self, range: Range<Self::Value>) {
    //     self.axis_mut().set_range(range)
    // }

    fn value_of_alloc(alloc: &Alloc) -> Self::Value;
    fn info_of_alloc(alloc: &Alloc) -> Self::LiveInfo;
    fn value_of_info(info: Self::LiveInfo, tod: &AllocDate) -> Self::Value;
    fn js_of_value(value: &Self::Value) -> Value;
    fn origin_value() -> Option<Self::Value>;
}

pub trait YAxis: Default + Clone + fmt::Debug {
    type Acc: Clone;
    type Value;
    type LiveInfo;

    fn axis(&self) -> &Axis<Self::Value>;
    fn axis_mut(&mut self) -> &mut Axis<Self::Value>;

    // fn set_range(&mut self, range: Range<Self::Value>) {
    //     self.axis_mut().set_range(range)
    // }

    fn init_acc() -> Self::Acc;
    fn value_of_alloc(alloc: &Alloc) -> Self::Value;
    fn value_of_acc(acc: &Self::Acc) -> Self::Value;
    fn combine_value(acc: Self::Acc, value: Self::Value) -> Self::Acc;
    fn info_of_alloc(alloc: &Alloc) -> Self::LiveInfo;
    fn combine_info(acc: Self::Acc, info: Self::LiveInfo, tod: &AllocDate) -> Self::Acc;
    fn combine_acc(prev: &Self::Acc, current: &Self::Acc) -> Self::Acc;
    fn js_of_value(value: &Self::Value) -> Value;
    fn origin_value() -> Option<Self::Acc>;
}

#[derive(Clone, Debug)]
pub struct XTime {
    axis: Axis<AllocDate>,
}
impl Default for XTime {
    fn default() -> Self {
        let axis = Axis::default_x("time in seconds");
        Self { axis }
    }
}
impl XAxis for XTime {
    type Value = AllocDate;
    type LiveInfo = ();

    const clear_map: bool = true;

    fn axis(&self) -> &Axis<AllocDate> {
        &self.axis
    }
    fn axis_mut(&mut self) -> &mut Axis<AllocDate> {
        &mut self.axis
    }

    fn value_of_alloc(alloc: &Alloc) -> Self::Value {
        alloc.toc()
    }
    fn info_of_alloc(_alloc: &Alloc) -> Self::LiveInfo {
        ()
    }
    fn value_of_info(_info: Self::LiveInfo, tod: &AllocDate) -> Self::Value {
        tod.clone()
    }
    fn js_of_value(value: &Self::Value) -> Value {
        let js = js! { return @{value.to_string()} };
        js
    }

    fn origin_value() -> Option<Self::Value> {
        Some(Duration::new(0, 0).into())
    }
}

#[derive(Clone, Debug)]
pub struct XSize {
    axis: Axis<usize>,
}
impl Default for XSize {
    fn default() -> Self {
        let mut axis = Axis::default_x("size in machine bytes");
        axis.ticks.callback = Ticks::<usize>::bytes_callback();
        Self { axis }
    }
}
impl XAxis for XSize {
    type Value = usize;
    type LiveInfo = usize;

    const clear_map: bool = false;

    fn axis(&self) -> &Axis<usize> {
        &self.axis
    }
    fn axis_mut(&mut self) -> &mut Axis<usize> {
        &mut self.axis
    }

    fn value_of_alloc(alloc: &Alloc) -> Self::Value {
        alloc.size()
    }
    fn info_of_alloc(alloc: &Alloc) -> Self::Value {
        alloc.size()
    }
    fn value_of_info(info: Self::LiveInfo, _tod: &AllocDate) -> Self::Value {
        info
    }
    fn js_of_value(value: &Self::Value) -> Value {
        js!(return @{value.to_string()})
    }
    fn origin_value() -> Option<Self::Value> {
        Some(0)
    }
}

#[derive(Clone, Debug)]
pub struct YSizeSum {
    axis: Axis<usize>,
}
impl Default for YSizeSum {
    fn default() -> Self {
        let mut axis = Axis::default_y("amount of live data in bytes");
        axis.ticks.callback = Ticks::<usize>::bytes_callback();
        axis.log_scale();
        Self { axis }
    }
}
impl YAxis for YSizeSum {
    type Acc = isize;
    type Value = usize;
    type LiveInfo = usize;

    fn axis(&self) -> &Axis<usize> {
        &self.axis
    }
    fn axis_mut(&mut self) -> &mut Axis<usize> {
        &mut self.axis
    }

    fn init_acc() -> Self::Acc {
        0
    }
    fn value_of_alloc(alloc: &Alloc) -> Self::Value {
        alloc.size() * std::mem::size_of::<usize>()
    }
    fn value_of_acc(acc: &Self::Acc) -> Self::Value {
        *acc as usize
    }
    fn combine_value(acc: Self::Acc, value: Self::Value) -> Self::Acc {
        acc + (value as isize)
    }
    fn info_of_alloc(alloc: &Alloc) -> Self::LiveInfo {
        alloc.size() * std::mem::size_of::<usize>()
    }
    fn combine_info(acc: Self::Acc, info: Self::LiveInfo, _: &AllocDate) -> Self::Acc {
        acc - (info as isize)
    }
    fn combine_acc(prev: &Self::Acc, current: &Self::Acc) -> Self::Acc {
        *prev + *current
    }
    fn js_of_value(value: &Self::Value) -> Value {
        let js = js! { return @{value.to_string()} };
        js
    }

    fn origin_value() -> Option<Self::Acc> {
        Some(0)
    }
}
