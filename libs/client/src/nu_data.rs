//! Client's global data.

use std::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};

use crate::base::*;

lazy_static::lazy_static! {
    /// Global data.
    static ref DATA: RwLock<Data> = RwLock::new(Data::new());
}

/// Global state accessor.
pub fn get<'a>() -> Res<RwLockReadGuard<'a, Data>> {
    DATA.read()
        .map_err(|e| {
            let e: err::Err = e.to_string().into();
            e
        })
        .chain_err(|| "while reading the global state")
}

/// Global state mutable accessor.
fn get_mut<'a>() -> Res<RwLockWriteGuard<'a, Data>> {
    DATA.write()
        .map_err(|e| {
            let e: err::Err = e.to_string().into();
            e
        })
        .chain_err(|| "while reading the global state")
}

/// Global data.
pub struct Data {
    /// Start time of the run.
    pub start_time: Option<AllocDate>,
}
impl Data {
    /// Constructor.
    pub fn new() -> Self {
        Self { start_time: None }
    }
}

/// Sets the start time of the run.
///
/// Fails if
///
/// - the global data was poisoned.
pub fn set_start_time(time: AllocDate) -> Res<()> {
    let mut data = get_mut().chain_err(|| "while writing run start time")?;
    data.start_time = Some(time);
    Ok(())
}

/// Retrieves the start time of the run.
///
/// Fails if either
///
/// - the global data was poisoned, or
/// - no start time is available.
pub fn start_time() -> Res<AllocDate> {
    let data = get().chain_err(|| "while reading run start time")?;
    data.start_time
        .clone()
        .ok_or_else(|| "trying to read run start time, but none is available".into())
}
