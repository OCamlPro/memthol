pub use std::{convert::TryInto, fmt};

pub use base::SVec16;

pub use alloc_data::{
    err::{bail, Res, ResExt},
    Uid,
};

pub use crate::{
    ast::{self, Span},
    err, loc,
    prelude::pos::Pos,
    RawParser,
};

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

pub fn id<T>(t: T) -> T {
    t
}
pub fn res_id<T, Err>(t: T) -> Result<T, Err> {
    Ok(t)
}
pub fn ignore<T>(_t: T) {}

pub type AllocId = u64;

pub type Pid = u64;

mod pos {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct Pos {
        pos: usize,
    }
    impl std::ops::Sub for Pos {
        type Output = usize;
        fn sub(self, other: Self) -> usize {
            self.pos - other.pos
        }
    }
    impl std::ops::Deref for Pos {
        type Target = usize;
        fn deref(&self) -> &usize {
            &self.pos
        }
    }
    impl std::fmt::Display for Pos {
        fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
            self.pos.fmt(fmt)
        }
    }
    impl<'data> crate::RawParser<'data> {
        /// Position accessor.
        pub fn pos(&self) -> Pos {
            Pos { pos: self.cursor }
        }
        /// Retrieves the byte at some position.
        pub fn get(&self, pos: Pos) -> Option<u8> {
            self.data.get(pos.pos).cloned()
        }
        /// Backtracks the parser to a **previous** position.
        pub fn backtrack(&mut self, pos: Pos) {
            debug_assert!(self.cursor >= pos.pos);
            self.cursor = pos.pos
        }
    }
    impl<'data> std::ops::Index<Pos> for crate::RawParser<'data> {
        type Output = u8;
        fn index(&self, pos: Pos) -> &u8 {
            &self.data[pos.pos]
        }
    }
}
