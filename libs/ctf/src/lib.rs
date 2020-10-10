pub extern crate bitlab;
pub extern crate log;

#[macro_use]
mod macros;

pub const VERSION: u16 = 2;

pub use alloc_data::err;

#[macro_use]
pub mod prelude;

pub mod ast;
pub mod btrace;
pub mod loc;
pub mod parse;

prelude! {}

/// Activates verbose parsing, only active in debug and test.
#[cfg(any(test, not(release)))]
const VERB: bool = false;
/// Activates debug parsing, only active in debug and test.
#[cfg(any(test, not(release)))]
const DEBUG_VERB: bool = false;

use ast::{event::Event, *};

pub use parse::parse_ctf as parse;

pub trait EventAction<'data>:
    FnMut(&ast::header::Packet, Clock, Event<'data>) -> err::Res<()>
{
}
impl<'data, T> EventAction<'data> for T where
    T: FnMut(&ast::header::Packet, Clock, Event<'data>) -> err::Res<()>
{
}
