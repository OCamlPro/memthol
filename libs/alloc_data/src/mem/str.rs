/*<LICENSE>
    This file is part of Memthol.

    Copyright (C) 2020 OCamlPro.

    Memthol is free software: you can redistribute it and/or modify
    it under the terms of the GNU General Public License as published by
    the Free Software Foundation, either version 3 of the License, or
    (at your option) any later version.

    Memthol is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
    GNU General Public License for more details.

    You should have received a copy of the GNU General Public License
    along with Memthol.  If not, see <https://www.gnu.org/licenses/>.
*/

//! Handles string sharing.
//!
//! #TODO
//!
//! - with some work, macro `crate::mem::new` could handle this case too

prelude! {}

type Memory = crate::mem::Memory<[u8]>;

/// Stores a UID, cannot be constructed outside of the module it's declared in.
#[derive(Debug, Clone, Copy, Eq, PartialOrd, Ord, Serialize, Deserialize, Hash)]
pub struct Str {
    /// String UID.
    uid: usize,
}
impl Str {
    /// Mutable factory accessor.
    pub fn factory_mut<'a>() -> AsWrite<'a> {
        write()
    }
    /// Immutable factory accessor.
    pub fn factory<'a>() -> AsRead<'a> {
        read()
    }

    /// String UID.
    pub fn uid(&self) -> usize {
        self.uid
    }

    /// Actual string accessor.
    pub fn get(self) -> Arc<[u8]> {
        Self::factory().get_elm(self)
    }

    /// Applies some action to the actual string.
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

    /// Registers a string in the factory.
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

/// Read-lock over the factory.
pub struct AsRead<'a> {
    mem: sync::RwLockReadGuard<'a, Memory>,
}
impl<'a> AsRead<'a> {
    /// Retrieves the bytes corresponding to a UID.
    pub fn get_elm(&self, uid: Str) -> Arc<[u8]> {
        self.mem.get_elm(uid.uid)
    }
}

/// Write-lock over the factory.
pub struct AsWrite<'a> {
    mem: sync::RwLockWriteGuard<'a, Memory>,
}
impl<'a> AsWrite<'a> {
    /// Creates/retrieves the UID of a string slice.
    pub fn get_uid(&mut self, s: &str) -> Str {
        Str {
            uid: self.mem.get_uid(s),
        }
    }
}

crate::prelude::lazy_static! {
    /// Memory.
    static ref MEM: sync::RwLock<Memory> = sync::RwLock::new(Memory::new());
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
