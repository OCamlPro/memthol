//! Error-handling types.

pub use error_chain::bail;

// use crate::common::Msg;

pub use charts::err::*;

// /// Turns a result into a message.
// ///
// /// If it's an error, it becomes an error message.
// pub fn msg_of_res(res: Res<Msg>) -> Msg {
//     match res {
//         Ok(msg) => msg,
//         Result::Err(e) => e.into(),
//     }
// }
