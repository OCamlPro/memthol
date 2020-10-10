//! Types and parsers for memthol's dump structures.
//!
//! These types are used by memthol's client when loading up memthol diffs.
//!
//! Generally speaking, all the types in this crate are parsed, not created from scratch. There is
//! no [`Uid`] factory for instance, since we will not have to generate fresh `Uid`s. We will only
//! parse them, the fact that they're unique must be guaranteed by whoever generated them.
//!
//! The entry point in terms of parsing is [`Diff`], since (currently) the only way the client can
//! build the other types is when parsing a `Diff`.
//!
//! # Dealing With Time
//!
//! There are two types to handle time: [`Date`] and [`SinceStart`]. The former encodes an absolute
//! date, while the latter is a only a duration. Memthol's init file specifies the `Date` at which
//! the program we're profiling started. After that, all the allocation data relies on `SinceStart`
//! to refer to point in times relative to the start date.
//!
//! [`Diff`]: struct.diff.html (The Diff struct)
//! [`Date`]: struct.date.html (The Date struct)
//! [`SinceStart`]: struct.sincestart.html (The SinceStart struct)

pub extern crate chrono;
pub extern crate peg;

pub use error_chain::bail;
pub use num_bigint::BigUint;

#[macro_use]
pub mod prelude;

#[macro_use]
pub mod mem;

pub mod parser;
pub mod time;

mod fmt;

prelude! {}

pub use time::{Date, Duration, SinceStart};

/// Errors, handled by `error_chain`.
pub mod err {
    crate::prelude::error_chain::error_chain! {
        types {
            Err, ErrKind, ResExt, Res;
        }

        foreign_links {
            Peg(peg::error::ParseError<peg::str::LineCol>)
            /// Parse error from `peg`.
            ;
        }

        links {}
        errors {}
    }

    pub use crate::prelude::error_chain::bail;
}

#[cfg(test)]
mod test;

/// A big-uint UID.
///
/// # Construction From String Slices
///
/// ```rust
/// # alloc_data::prelude! {}
/// let s = "72430";
/// let uid = Uid::parse(s).unwrap();
/// # println!("uid: {}", uid);
/// assert_eq! { format!("{}", uid), s }
/// ```
///
/// ```rust
/// # alloc_data::prelude! {}
/// let s = "643128653641564321563425361425364523164523164";
/// let uid = Uid::parse(s).unwrap();
/// # println!("uid: {}", uid);
/// assert_eq! { format!("{}", uid), s }
/// ```
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Uid {
    /// The actual bigint.
    uid: BigUint,
}

impl std::ops::Deref for Uid {
    type Target = BigUint;
    fn deref(&self) -> &BigUint {
        &self.uid
    }
}
impl From<BigUint> for Uid {
    fn from(uid: BigUint) -> Self {
        Self { uid }
    }
}
impl From<u64> for Uid {
    fn from(uid: u64) -> Self {
        Self { uid: uid.into() }
    }
}

/// A span.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, base::From, Serialize, Deserialize)]
pub struct Span {
    /// Start of the span.
    pub start: usize,
    /// End of the span.
    pub end: usize,
}

/// A location.
///
/// # Construction From String Slices
///
/// ```rust
/// # alloc_data::prelude! {}
/// let s = "`blah/stuff/file.ml`:325:7-38";
/// let loc = Loc::parse(s).unwrap();
/// # println!("loc: {}", loc);
/// assert_eq! { format!("{}", loc), s }
/// assert_eq! { loc.file, "blah/stuff/file.ml" }
/// assert_eq! { loc.line, 325 }
/// assert_eq! { loc.span, (7, 38).into() }
/// ```
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Loc {
    /// File the location is for.
    pub file: Str,
    /// Line in the file.
    pub line: usize,
    /// Column span at that line in the file.
    pub span: Span,
}
impl Loc {
    /// Constructor.
    pub fn new(file: Str, line: usize, span: impl Into<Span>) -> Self {
        Self {
            file,
            line,
            span: span.into(),
        }
    }
}

/// A counted location.
///
/// Used in callstacks to represent a repetition of locations.
///
/// # Construction From String Slices
///
/// ```rust
/// # alloc_data::prelude! {}
/// let s = "`blah/stuff/file.ml`:325:7-38#5";
/// let CLoc { loc, cnt } = CLoc::parse(s).unwrap();
/// # println!("loc_count: {}#{}", loc, cnt);
/// assert_eq! { format!("{}", loc), s[0..s.len()-2] }
/// assert_eq! { loc.file, "blah/stuff/file.ml" }
/// assert_eq! { loc.line, 325 }
/// assert_eq! { loc.span, (7, 38).into() }
/// assert_eq! { cnt, 5 }
/// ```
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct CLoc {
    /// Location.
    pub loc: Loc,
    /// Number of times the location is repeated.
    pub cnt: usize,
}
impl CLoc {
    /// Constructor.
    pub fn new(loc: Loc, cnt: usize) -> Self {
        Self { loc, cnt }
    }
}

/// A kind of allocation.
///
/// # Construction From String Slices
///
/// ```rust
/// # alloc_data::prelude! {}
/// let s_list = [
///     ("Minor", AllocKind::Minor),
///     ("Major", AllocKind::Major),
///     ("MajorPostponed", AllocKind::MajorPostponed),
///     ("Serialized", AllocKind::Serialized),
/// ];
/// for (s, exp) in &s_list {
///     let kind = AllocKind::parse(*s).unwrap();
///     assert_eq! { kind, *exp }
/// }
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AllocKind {
    Minor,
    Major,
    MajorPostponed,
    Serialized,
    Unknown,
}

impl AllocKind {
    /// String representation of an allocation kind.
    pub fn as_str(&self) -> &'static str {
        use AllocKind::*;
        match self {
            Minor => "Minor",
            Major => "Major",
            MajorPostponed => "MajorPostponed",
            Serialized => "Serialized",
            Unknown => "_",
        }
    }
}

/// Some allocation information.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Alloc {
    /// Uid of the allocation.
    pub uid: Uid,
    /// Allocation kind.
    pub kind: AllocKind,
    /// Size of the allocation.
    pub size: u32,
    /// Allocation-site callstack.
    trace: Trace,
    /// User-defined labels.
    labels: Labels,
    /// Time of creation.
    pub toc: SinceStart,
    /// Time of death.
    pub tod: Option<SinceStart>,
}

impl Alloc {
    /// Constructor.
    pub fn new(
        uid: impl Into<Uid>,
        kind: AllocKind,
        size: u32,
        trace: Trace,
        labels: Labels,
        toc: SinceStart,
        tod: Option<SinceStart>,
    ) -> Self {
        let uid = uid.into();
        Self {
            uid,
            kind,
            size,
            trace,
            labels,
            toc,
            tod,
        }
    }

    /// Sets the time of death.
    ///
    /// Bails if a time of death is already registered.
    pub fn set_tod(&mut self, tod: SinceStart) -> Result<(), String> {
        if self.tod.is_some() {
            Err("\
                 trying to set the time of death, \
                 but a tod is already registered for this allocation\
                 "
            .into())
        } else {
            self.tod = Some(tod);
            Ok(())
        }
    }

    /// Sets the time of creation.
    pub fn set_toc(&mut self, toc: SinceStart) {
        self.toc = toc
    }

    /// UID accessor.
    pub fn uid(&self) -> &Uid {
        &self.uid
    }
    /// Kind accessor.
    pub fn kind(&self) -> &AllocKind {
        &self.kind
    }
    /// Size accessor (in machine words).
    pub fn size(&self) -> u32 {
        self.size
    }

    /// Trace accessor.
    pub fn trace(&self) -> std::sync::Arc<Vec<CLoc>> {
        self.trace.get()
    }
    /// Allocation-site of the allocation.
    pub fn alloc_site_do<Res>(&self, action: impl FnOnce(Option<&CLoc>) -> Res) -> Res {
        let trace = self.trace();
        action(trace.last())
    }

    /// Labels accessor.
    pub fn labels(&self) -> std::sync::Arc<Vec<Str>> {
        self.labels.get()
    }
    /// Time of creation accessor.
    pub fn toc(&self) -> SinceStart {
        self.toc
    }
    /// Time of death accessor.
    pub fn tod(&self) -> Option<SinceStart> {
        self.tod
    }
}

/// A diff.
///
/// **NB:** `Display` for this type is multi-line.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Diff {
    /// Timestamp.
    pub time: SinceStart,
    /// New allocations in this diff.
    pub new: Vec<Alloc>,
    /// Data freed in this diff.
    pub dead: Vec<(Uid, SinceStart)>,
}

impl Diff {
    /// Constructor.
    pub fn new(time: SinceStart, new: Vec<Alloc>, dead: Vec<(Uid, SinceStart)>) -> Self {
        Self { time, new, dead }
    }
}

/// Data from a memthol init file.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Init {
    /// The start time of the run: an absolute date.
    pub start_time: Date,
    /// Size of machine words in bytes.
    pub word_size: usize,
    /// True if the callstack go from `main` to allocation site, called *reversed order*.
    pub callstack_is_rev: bool,
}

impl Default for Init {
    fn default() -> Self {
        Self {
            start_time: Date::from_timestamp(0, 0),
            word_size: 8,
            callstack_is_rev: false,
        }
    }
}

impl Init {
    /// Constructor.
    pub fn new(start_time: Date, word_size: usize, callstack_is_rev: bool) -> Self {
        Self {
            start_time,
            word_size,
            callstack_is_rev,
        }
    }
}
