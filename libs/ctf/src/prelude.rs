pub use base::prelude::*;

pub use crate::{ast::Span, parse::CanParse, *};

pub type Clock = u64;
pub type DeltaClock = u64;

pub fn id<T>(t: T) -> T {
    t
}
pub fn res_id<T, Err>(t: T) -> Result<T, Err> {
    Ok(t)
}
pub fn ignore<T>(_t: T) {}

pub type AllocId = u64;

pub type Pid = u64;
