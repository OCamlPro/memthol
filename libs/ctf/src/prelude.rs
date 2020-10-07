pub use std::convert::TryInto;

// pub use log::info;

pub const VERB: bool = true;
pub const DEBUG_VERB: bool = false;

macro_rules! pinfo {
    ($parser:expr, $($blah:tt)*) => {if prelude::VERB {
        let (pos, max) = $parser.position();
        println!("[{}/{}] {}", pos, max, format_args!($($blah)*))
    }};
}
macro_rules! pdebug {
    ($parser:expr, $($blah:tt)*) => {if prelude::DEBUG_VERB {
        let (pos, max) = $parser.position();
        println!("[{}/{}] {}", pos, max, format_args!($($blah)*))
    }};
}

pub use crate::ast::{self, Span};

pub use pos::Pos;

pub type Res<T> = Result<T, String>;

pub trait ResExt {
    fn chain_err<F, S>(self, err: F) -> Self
    where
        S: AsRef<str>,
        F: FnOnce() -> S;
    fn subst_err<F, S>(self, err: F) -> Self
    where
        S: Into<String>,
        F: FnOnce() -> S;
}
impl<T> ResExt for Res<T> {
    fn chain_err<F, S>(mut self, err: F) -> Self
    where
        S: AsRef<str>,
        F: FnOnce() -> S,
    {
        if let Some(e) = self.as_mut().err() {
            e.push_str("\n");
            e.push_str(err().as_ref())
        }
        self
    }
    fn subst_err<F, S>(mut self, err: F) -> Self
    where
        S: Into<String>,
        F: FnOnce() -> S,
    {
        if let Some(e) = self.as_mut().err() {
            let _old = std::mem::replace(e, err().into());
        }
        self
    }
}

pub trait StringResExt<T> {
    fn to_res(self) -> Res<T>;
}
impl<T, E> StringResExt<T> for Result<T, E>
where
    E: std::fmt::Display,
{
    fn to_res(self) -> Res<T> {
        match self {
            Ok(res) => Ok(res),
            Err(e) => Err(e.to_string()),
        }
    }
}

pub type Clock = i64;

pub fn id<T>(t: T) -> T {
    t
}
pub fn res_id<T, Err>(t: T) -> Result<T, Err> {
    Ok(t)
}
pub fn ignore<T>(_t: T) {}

pub type AllocId = i64;

pub type Pid = i64;

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
