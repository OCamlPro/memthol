//! Provides a generic factory-like type to share labels and locations across allocations.

use std::{collections::BTreeMap as Map, sync::Arc};

prelude! {}

/// Convenience macro that creates a lazy-static-rw-locked memory and some accessors.
macro_rules! new {
    (mod $mod:ident for $ty:ty, uid: $uid:ident) => {
        mod $mod {
            prelude! {}
            pub use std::sync::{Arc, RwLock};

            /// Stores a UID, cannot be constructed outside of the module it's declared in.
            #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
            pub struct $uid {
                uid: usize,
            }
            impl $uid {
                pub fn factory_mut<'a>() -> AsWrite<'a> {
                    write()
                }
                pub fn factory<'a>() -> AsRead<'a> {
                    read()
                }

                pub fn get(self) -> Arc<$ty> {
                    Self::factory().get_elm(self)
                }

                pub fn new(elm: $ty) -> Self {
                    Self::factory_mut().get_uid(elm)
                }
            }

            /// Type of the memory structure.
            type Memory = $crate::mem::Memory<$ty>;

            pub struct AsRead<'a> {
                mem: std::sync::RwLockReadGuard<'a, Memory>,
            }
            impl<'a> AsRead<'a> {
                pub fn get_elm(&self, uid: $uid) -> Arc<$ty> {
                    self.mem.get_elm(uid.uid)
                }
            }
            pub struct AsWrite<'a> {
                mem: std::sync::RwLockWriteGuard<'a, Memory>,
            }
            impl<'a> AsWrite<'a> {
                pub fn get_uid(&mut self, elm: $ty) -> $uid {
                    $uid {
                        uid: self.mem.get_uid(elm),
                    }
                }
                pub fn get_elm(&self, uid: $uid) -> Arc<$ty> {
                    self.mem.get_elm(uid.uid)
                }
            }

            $crate::prelude::lazy_static! {
                /// Memory.
                static ref MEM: RwLock<Memory> = RwLock::new(Memory::new());
            }

            /// Provides read-access to the memory.
            ///
            /// Panics if the memory has been poisoned.
            pub fn read<'a>() -> AsRead<'a> {
                AsRead {
                    mem: MEM
                        .read()
                        .expect("fatal error: a data memory has been poisoned"),
                }
            }
            /// Provides write-access to the memory.
            ///
            /// Panics if the memory has been poisoned.
            pub fn write<'a>() -> AsWrite<'a> {
                AsWrite {
                    mem: MEM
                        .write()
                        .expect("fatal error: a data memory has been poisoned"),
                }
            }
        }
    };
}

pub mod labels;
pub mod str;
pub mod trace;

/// Factory for string, labels and trace creation.
pub struct Factory<'a> {
    str: str::AsWrite<'a>,
    labels: labels::AsWrite<'a>,
    trace: trace::AsWrite<'a>,
    /// Indicates whether the callstacks are in reverse order.
    ///
    /// If true, callstacks must be reversed when registering them.
    callstack_is_rev: bool,
}
impl<'a> Factory<'a> {
    /// Constructor.
    pub fn new(callstack_is_rev: bool) -> Self {
        Self {
            str: Str::factory_mut(),
            labels: Labels::factory_mut(),
            trace: Trace::factory_mut(),
            callstack_is_rev,
        }
    }

    pub fn register_str(&mut self, s: &str) -> Str {
        self.str.get_uid(s)
    }
    pub fn register_labels(&mut self, labels: Vec<Str>) -> Labels {
        self.labels.get_uid(labels)
    }
    pub fn register_trace(&mut self, mut trace: Vec<CLoc>) -> Trace {
        if self.callstack_is_rev {
            trace.reverse()
        }
        self.trace.get_uid(trace)
    }
}

/// A structure mapping some elements to UIDs and back.
///
/// This type is very biased towards a particular situation: new elements are very rare compared to
///
/// - insertion of already-known elements, and
/// - query over already-known elements.
pub struct Memory<Elm: ?Sized> {
    map: Map<Arc<Elm>, usize>,
    vec: Vec<Arc<Elm>>,
}

impl<Elm: ?Sized + Ord> Memory<Elm> {
    /// Constructor.
    pub fn new() -> Self {
        Self {
            map: Map::new(),
            vec: Vec::with_capacity(103),
        }
    }

    /// Retrieves an element from its UID.
    pub fn get_elm(&self, uid: usize) -> Arc<Elm> {
        self.vec[uid].clone()
    }
}

impl<Elm> Memory<Elm>
where
    Elm: Ord + Sized,
{
    /// The UID associated to some element.
    ///
    /// Generates a fresh one if none exists. Biased towards the case when the element is already
    /// registered.
    pub fn get_uid(&mut self, elm: Elm) -> usize {
        if let Some(uid) = self.map.get(&elm) {
            *uid
        } else {
            let uid = self.vec.len();
            let elm = Arc::new(elm);
            self.vec.push(elm);
            let prev = self.map.insert(self.vec[uid].clone(), uid);
            debug_assert_eq!(prev, None);
            uid
        }
    }
}
impl Memory<[u8]> {
    fn get_uid(&mut self, s: &str) -> usize {
        if let Some(uid) = self.map.get(s.as_bytes()) {
            *uid
        } else {
            let uid = self.vec.len();
            let elm = s.to_owned().into_boxed_str().into_boxed_bytes().into();
            self.vec.push(elm);
            let prev = self.map.insert(self.vec[uid].clone(), uid);
            debug_assert_eq!(prev, None);
            uid
        }
    }
}
