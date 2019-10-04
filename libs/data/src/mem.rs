//! Provides a generic factory-like type to share labels and locations across allocations.

use std::{collections::BTreeMap as Map, sync::Arc};

pub use serde_derive::{Deserialize, Serialize};

/// A UID.
///
/// Cannot be constructed outside of [`alloc_data::mem`].
///
/// [`alloc_data::mem`]: index.html (The mem module in alloc_data)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize, Serialize)]
pub struct Uid {
    /// Actual index.
    index: usize,
}

/// A structure mapping some elements to UIDs and back.
///
/// This type is very biased towards a particular situation: new elements are very rare compared to
///
/// - insertion of already-known elements, and
/// - query over already-known elements.
pub struct Memory<Elm>
where
    Elm: Ord,
{
    map: Map<Arc<Elm>, usize>,
    vec: Vec<Arc<Elm>>,
}
impl<Elm> Memory<Elm>
where
    Elm: Ord,
{
    /// Constructor.
    pub fn new() -> Self {
        Self {
            map: Map::new(),
            vec: Vec::with_capacity(103),
        }
    }

    /// The UID associated to some element.
    ///
    /// Generates a fresh one if none exists.
    pub fn get_uid(&mut self, elm: Elm) -> Uid
    where
        Elm: Clone,
    {
        let elm = Arc::new(elm);
        let next_index = self.vec.len();
        let index = *self.map.entry(elm.clone()).or_insert(next_index);
        // Insert in `self.vec` if new.
        if index == next_index {
            self.vec.push(elm)
        }
        Uid { index }
    }

    /// Retrieves an element from its UID.
    pub fn get_elm(&self, uid: Uid) -> Arc<Elm> {
        self.vec[uid.index].clone()
    }
}

/// Convenience macro that creates a lazy-static-rw-locked memory and some accessors.
macro_rules! new {
    (mod $mod:ident for $ty:ty) => {
        mod $mod {
            pub use std::sync::{Arc, RwLock};
            pub use $crate::mem::Uid;

            /// Type of the memory structure.
            pub type Memory = $crate::mem::Memory<$ty>;

            lazy_static::lazy_static! {
                /// Memory.
                static ref MEM: RwLock<Memory> = RwLock::new(Memory::new());
            }

            /// Provides read-access to the memory.
            ///
            /// Panics if the memory has been poisoned.
            pub fn read<'a>() -> std::sync::RwLockReadGuard<'a, Memory> {
                MEM.read()
                    .expect("fatal error: a data memory has been poisoned")
            }
            /// Provides write-access to the memory.
            ///
            /// Panics if the memory has been poisoned.
            pub fn write<'a>() -> std::sync::RwLockWriteGuard<'a, Memory> {
                MEM.write()
                    .expect("fatal error: a data memory has been poisoned")
            }
        }
    };
}
