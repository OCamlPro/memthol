pub use std::{convert::TryInto, fmt};

pub use base::{Either, SVec16};

pub use alloc_data::{
    err::{bail, Res, ResExt},
    time::Date,
    Uid,
};

pub use crate::{ast::Span, parse::CanParse, *};

pub fn date_of_timestamp(ts: u64) -> Date {
    let secs = ts / 1_000_000;
    let micros = ts - secs * 1_000_000;
    Date::from_timestamp(secs as i64, (micros as u32) * 1_000)
}

// pub use log::info;

pub fn destroy<T>(_: T) {}

/// Used to convert between integer representations.
pub fn convert<In, Out>(n: In, from: &'static str) -> Out
where
    In: TryInto<Out> + fmt::Display + Copy,
    In::Error: fmt::Display,
{
    match n.try_into() {
        Ok(res) => res,
        Err(e) => panic!("[fatal] while converting {} ({}): {}", n, from, e),
    }
}

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
