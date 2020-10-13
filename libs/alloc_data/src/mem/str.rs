//! Handles string sharing.
//!
//! #TODO
//!
//! - with some work, macro `crate::mem::new` could handle this case too

pub use std::sync::{Arc, RwLock};

prelude! {}

type Memory = crate::mem::Memory<[u8]>;

/// Stores a UID, cannot be constructed outside of the module it's declared in.
#[derive(Debug, Clone, Copy, Eq, PartialOrd, Ord, Serialize, Deserialize, Hash)]
pub struct Str {
    uid: usize,
}
impl Str {
    pub fn factory_mut<'a>() -> AsWrite<'a> {
        write()
    }
    pub fn factory<'a>() -> AsRead<'a> {
        read()
    }

    pub fn get(self) -> Arc<[u8]> {
        Self::factory().get_elm(self)
    }

    pub fn str_do<Res>(self, action: impl FnOnce(&str) -> Res) -> Res {
        let elm = self.get();
        let str = std::str::from_utf8(elm.as_ref()).unwrap_or_else(|e| {
            panic!(
                "shared string stored as bytes is not a legal string: {:?}\n{}",
                elm, e
            )
        });
        action(str)
    }

    pub fn new(str: &str) -> Str {
        Self::factory_mut().get_uid(str)
    }
}
impl std::fmt::Display for Str {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        self.str_do(|s| s.fmt(fmt))
    }
}
impl PartialEq<String> for Str {
    fn eq(&self, other: &String) -> bool {
        &*self.get() == other.as_bytes()
    }
}
impl PartialEq<Str> for String {
    fn eq(&self, other: &Str) -> bool {
        &*other.get() == self.as_bytes()
    }
}
impl PartialEq<str> for Str {
    fn eq(&self, other: &str) -> bool {
        &*self.get() == other.as_bytes()
    }
}
impl PartialEq<Str> for str {
    fn eq(&self, other: &Str) -> bool {
        &*other.get() == self.as_bytes()
    }
}
impl PartialEq<&str> for Str {
    fn eq(&self, other: &&str) -> bool {
        &*self.get() == other.as_bytes()
    }
}
impl PartialEq<Str> for &str {
    fn eq(&self, other: &Str) -> bool {
        &*other.get() == self.as_bytes()
    }
}
impl PartialEq<Str> for Str {
    fn eq(&self, other: &Str) -> bool {
        self.uid == other.uid
    }
}
impl<'a> PartialEq<Str> for &'a Str {
    fn eq(&self, other: &Str) -> bool {
        self.uid == other.uid
    }
}
impl<'a> PartialEq<&'a Str> for Str {
    fn eq(&self, other: &&'a Str) -> bool {
        self.uid == other.uid
    }
}

pub struct AsRead<'a> {
    mem: std::sync::RwLockReadGuard<'a, Memory>,
}
impl<'a> AsRead<'a> {
    pub fn get_elm(&self, uid: Str) -> Arc<[u8]> {
        self.mem.get_elm(uid.uid)
    }
}
pub struct AsWrite<'a> {
    mem: std::sync::RwLockWriteGuard<'a, Memory>,
}
impl<'a> AsWrite<'a> {
    pub fn get_uid(&mut self, s: &str) -> Str {
        Str {
            uid: self.mem.get_uid(s),
        }
    }
}

crate::prelude::lazy_static! {
    /// Memory.
    static ref MEM: RwLock<Memory> = RwLock::new(Memory::new());
}

/// Provides read-access to the memory.
///
/// Panics if the memory has been poisoned.
fn read<'a>() -> AsRead<'a> {
    AsRead {
        mem: MEM
            .read()
            .expect("fatal error: a data memory has been poisoned"),
    }
}
/// Provides write-access to the memory.
///
/// Panics if the memory has been poisoned.
fn write<'a>() -> AsWrite<'a> {
    AsWrite {
        mem: MEM
            .write()
            .expect("fatal error: a data memory has been poisoned"),
    }
}
