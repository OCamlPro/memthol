///! Filter over ordered (number-like) quantities.
use crate::base::*;

/// A filter over number-like values.
///
/// Internally represented as an interval that can be unspecified in either directions.
#[derive(Debug, Clone)]
pub struct NumFilter<Num> {
    /// Lower-bound.
    lb: Option<Num>,
    /// Upper-bound.
    ub: Option<Num>,
}
impl<Num> crate::filter::FilterSpec<Num> for NumFilter<Num>
where
    Num: PartialOrd,
{
    fn apply(&self, _: &Storage, data: &Num) -> bool {
        self.lb.as_ref().map(|val| val <= data).unwrap_or(true)
            && self.ub.as_ref().map(|val| data <= val).unwrap_or(true)
    }

    fn render(&self) -> Html {
        unimplemented!()
    }
}
impl<Num> Default for NumFilter<Num> {
    fn default() -> Self {
        Self { lb: None, ub: None }
    }
}

impl<Num> NumFilter<Num>
where
    Num: Clone,
{
    /// Constructor.
    pub fn new(op: rendering::Op<Num>) -> Self {
        use rendering::{Op::*, UnOp::*};
        let (lb, ub) = match op {
            Unary { op: Eq, val } => (Some(val.clone()), Some(val)),
            Unary { op: Le, val } => (None, Some(val)),
            Unary { op: Ge, val } => (Some(val), None),
            Between { lb, ub } => (Some(lb), Some(ub)),
        };
        Self { lb, ub }
    }
}

impl<Num> NumFilter<Num>
where
    Num: PartialOrd,
{
    /// Returns the filter as a function over values.
    pub fn get<'a>(&'a self) -> impl Fn(&Num) -> bool + 'a {
        move |value| {
            self.lb.as_ref().map(|val| val <= value).unwrap_or(true)
                && self.ub.as_ref().map(|val| value <= val).unwrap_or(true)
        }
    }
}

/// Rendering and construction of filters over ordered quantities.
pub mod rendering {
    use super::*;

    /// Unary filter.
    pub enum UnOp {
        /// Equal to.
        Eq,
        /// Greater than or equal to.
        Ge,
        /// Less than or equal to.
        Le,
    }
    impl fmt::Display for UnOp {
        fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
            match self {
                UnOp::Eq => write!(fmt, "="),
                UnOp::Ge => write!(fmt, ">="),
                UnOp::Le => write!(fmt, "<="),
            }
        }
    }

    /// A filter operator.
    pub enum Op<Num> {
        /// Equal to.
        Unary {
            /// Operator.
            op: UnOp,
            /// Value to compare with.
            val: Num,
        },
        /// Interval, inclusive on both ends.
        Between {
            /// Lower bound.
            lb: Num,
            /// Upper bound.
            ub: Num,
        },
    }
    impl<Num> fmt::Display for Op<Num>
    where
        Num: fmt::Display,
    {
        fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
            match self {
                Op::Unary { op, val } => write!(fmt, "{} {}", op, val),
                Op::Between { lb, ub } => write!(fmt, "⋲ [{}, {}]", lb, ub),
            }
        }
    }
}
