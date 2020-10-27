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

#![deny(missing_docs)]

pub use error_chain::bail;
pub use num_bigint::BigUint;

#[macro_use]
pub mod prelude;

#[macro_use]
pub mod mem;

pub mod parser;

mod fmt;

prelude! {}

/// Errors, handled by `error_chain`.
pub mod err {
    pub use base::err::*;
}

/// Some tests, only active in `test` mode.
#[cfg(test)]
mod test;

/// A span.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Span {
    /// Start of the span.
    pub start: usize,
    /// End of the span.
    pub end: usize,
}

base::implement! {
    impl From for Span {
        from (usize, usize) => |(start, end)| Self { start, end }
    }
}

impl Span {
    /// Construtor.
    pub fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }
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
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
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
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AllocKind {
    /// Minor allocation.
    Minor,
    /// Major allocation.
    Major,
    /// Major postponed.
    MajorPostponed,
    /// Serialized.
    Serialized,
    /// Unknown allocation.
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

/// An allocation builder.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Builder {
    /// UID hint.
    pub uid_hint: Option<uid::Alloc>,
    /// Allocation kind.
    pub kind: AllocKind,
    /// Size of the allocation.
    pub size: u32,
    /// Sample count.
    pub nsamples: u32,
    /// Allocation-site callstack.
    trace: Trace,
    /// User-defined labels.
    labels: Labels,
    /// Time of creation.
    pub toc: time::SinceStart,
    /// Time of death.
    pub tod: Option<time::SinceStart>,
}
impl Builder {
    /// Constructor.
    pub fn new(
        uid_hint: Option<uid::Alloc>,
        kind: AllocKind,
        size: u32,
        trace: Trace,
        labels: Labels,
        toc: time::SinceStart,
        tod: Option<time::SinceStart>,
    ) -> Self {
        Self {
            uid_hint,
            kind,
            size,
            nsamples: size,
            trace,
            labels,
            toc,
            tod,
        }
    }

    /// Sets the number of samples.
    pub fn nsamples(mut self, nsamples: u32) -> Self {
        self.nsamples = nsamples;
        self
    }

    /// Builds an `Alloc`.
    pub fn build(self, uid: uid::Alloc) -> Res<Alloc> {
        let Self {
            uid_hint,
            kind,
            size,
            nsamples,
            trace,
            labels,
            toc,
            tod,
        } = self;
        match uid_hint {
            None => (),
            Some(hint) if uid == hint => (),
            Some(hint) => bail!(
                "failed alloc UID check: expected {}, but hint says {}",
                uid,
                hint,
            ),
        }
        Ok(Alloc {
            uid,
            kind,
            size,
            nsamples,
            trace,
            labels,
            toc,
            tod,
        })
    }
}

/// Some allocation information.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Alloc {
    /// Uid of the allocation.
    pub uid: uid::Alloc,
    /// Allocation kind.
    pub kind: AllocKind,
    /// Size of the allocation.
    pub size: u32,
    /// Sample count.
    pub nsamples: u32,
    /// Allocation-site callstack.
    trace: Trace,
    /// User-defined labels.
    labels: Labels,
    /// Time of creation.
    pub toc: time::SinceStart,
    /// Time of death.
    pub tod: Option<time::SinceStart>,
}

impl Alloc {
    /// Constructor.
    pub fn new(
        uid: impl Into<uid::Alloc>,
        kind: AllocKind,
        size: u32,
        trace: Trace,
        labels: Labels,
        toc: time::SinceStart,
        tod: Option<time::SinceStart>,
    ) -> Self {
        let uid = uid.into();
        Self {
            uid,
            kind,
            size,
            nsamples: size,
            trace,
            labels,
            toc,
            tod,
        }
    }

    /// Sets the number of samples.
    pub fn nsamples(mut self, nsamples: u32) -> Self {
        self.nsamples = nsamples;
        self
    }

    /// Sets the time of death.
    ///
    /// Bails if a time of death is already registered.
    pub fn set_tod(&mut self, tod: time::SinceStart) -> Result<(), String> {
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
    pub fn set_toc(&mut self, toc: time::SinceStart) {
        self.toc = toc
    }

    /// UID accessor.
    pub fn uid(&self) -> &uid::Alloc {
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
    pub fn trace(&self) -> Arc<Vec<CLoc>> {
        self.trace.get()
    }
    /// Allocation-site of the allocation.
    pub fn alloc_site_do<Res>(&self, action: impl FnOnce(Option<&CLoc>) -> Res) -> Res {
        let trace = self.trace();
        action(trace.last())
    }

    /// Labels accessor.
    pub fn labels(&self) -> Arc<Vec<Str>> {
        self.labels.get()
    }
    /// Time of creation accessor.
    pub fn toc(&self) -> time::SinceStart {
        self.toc
    }
    /// Time of death accessor.
    pub fn tod(&self) -> Option<time::SinceStart> {
        self.tod
    }
}

/// A diff.
///
/// **NB:** `Display` for this type is multi-line.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Diff {
    /// Timestamp.
    pub time: time::SinceStart,
    /// New allocations in this diff.
    pub new: Vec<Builder>,
    /// Data freed in this diff.
    pub dead: Vec<(uid::Alloc, time::SinceStart)>,
}

impl Diff {
    /// Constructor.
    pub fn new(
        time: time::SinceStart,
        new: Vec<Builder>,
        dead: Vec<(uid::Alloc, time::SinceStart)>,
    ) -> Self {
        Self { time, new, dead }
    }
}

/// Data from a memthol init file.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Init {
    /// The start time of the run: an absolute date.
    pub start_time: time::Date,
    /// Optional end time.
    pub end_time: Option<time::SinceStart>,
    /// Size of machine words in bytes.
    pub word_size: usize,
    /// True if the callstack go from `main` to allocation site, called *reversed order*.
    pub callstack_is_rev: bool,
    /// Sampling rate.
    pub sampling_rate: base::SampleRate,
}

impl Default for Init {
    fn default() -> Self {
        Self {
            start_time: time::Date::from_timestamp(0, 0),
            end_time: None,
            word_size: 8,
            callstack_is_rev: false,
            sampling_rate: 1.0.into(),
        }
    }
}

impl Init {
    /// Constructor.
    pub fn new(
        start_time: time::Date,
        end_time: Option<time::SinceStart>,
        word_size: usize,
        callstack_is_rev: bool,
    ) -> Self {
        Self {
            start_time,
            end_time,
            word_size,
            callstack_is_rev,
            sampling_rate: 1.0.into(),
        }
    }

    /// Sets the sampling rate.
    pub fn sampling_rate(mut self, rate: f64) -> Self {
        self.sampling_rate = rate.into();
        self
    }
}
