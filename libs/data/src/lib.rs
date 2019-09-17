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

pub use error_chain::bail;
pub use num_bigint::BigUint;
pub use serde_derive::{Deserialize, Serialize};

pub mod parser;
mod time;

pub use parser::err::ParseRes as Res;
pub use parser::Parser;
pub use time::{Date, Duration, SinceStart};

#[cfg(test)]
mod test;

/// A bigint UID.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Uid {
    /// The actual bigint.
    uid: BigUint,
}
swarkn::display! {
    impl for Uid {
        self, fmt => self.uid.fmt(fmt)
    }
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

impl Uid {
    /// Parses an UID.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use alloc_data::Uid;
    /// let s = "72430";
    /// let uid = Uid::from_str(s).unwrap();
    /// # println!("uid: {}", uid);
    /// assert_eq! { format!("{}", uid), s }
    /// ```
    ///
    /// ```rust
    /// use alloc_data::Uid;
    /// let s = "643128653641564321563425361425364523164523164";
    /// let uid = Uid::from_str(s).unwrap();
    /// # println!("uid: {}", uid);
    /// assert_eq! { format!("{}", uid), s }
    /// ```
    pub fn from_str<Str: AsRef<str>>(s: Str) -> Res<Self> {
        Parser::parse_all(s.as_ref(), Parser::uid, "uid")
    }
}

/// A location.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Loc {
    /// File the location is for.
    pub file: String,
    /// Line in the file.
    pub line: usize,
    /// Column span at that line in the file.
    pub span: (usize, usize),
}
swarkn::display! {
    impl for Loc {
        self, fmt => write!(
            fmt,
            "`{}`:{}:{}-{}",
            self.file, self.line, self.span.0, self.span.1
        )
    }
}

impl Loc {
    /// Constructor.
    pub fn new<S: Into<String>>(file: S, line: usize, span: (usize, usize)) -> Self {
        Self {
            file: file.into(),
            line,
            span,
        }
    }

    /// Parses a location.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use alloc_data::Loc;
    /// let s = "`blah/stuff/file.ml`:325:7-38";
    /// let loc = Loc::from_str(s).unwrap();
    /// # println!("loc: {}", loc);
    /// assert_eq! { format!("{}", loc), s }
    /// assert_eq! { loc.file, "blah/stuff/file.ml" }
    /// assert_eq! { loc.line, 325 }
    /// assert_eq! { loc.span, (7, 38) }
    /// ```
    pub fn from_str<Str: AsRef<str>>(s: Str) -> Res<Self> {
        Parser::parse_all(s.as_ref(), Parser::loc, "location")
    }

    /// Parses a location with a count.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use alloc_data::Loc;
    /// let s = "`blah/stuff/file.ml`:325:7-38#5";
    /// let (loc, count) = Loc::from_str_with_count(s).unwrap();
    /// # println!("loc_count: {}#{}", loc, count);
    /// assert_eq! { format!("{}", loc), s[0..s.len()-2] }
    /// assert_eq! { loc.file, "blah/stuff/file.ml" }
    /// assert_eq! { loc.line, 325 }
    /// assert_eq! { loc.span, (7, 38) }
    /// assert_eq! { count, 5 }
    /// ```
    pub fn from_str_with_count<Str: AsRef<str>>(s: Str) -> Res<(Self, usize)> {
        Parser::parse_all(s.as_ref(), Parser::loc_count, "location with count")
    }
}

/// A trace of locations.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Trace {
    /// The actual trace of locations.
    trace: Vec<(Loc, usize)>,
}
swarkn::display! {
    impl for Trace {
        self, fmt => {
            write!(fmt, "[")?;
            for (loc, count) in &self.trace {
                write!(fmt, " {}#{}", loc, count)?
            }
            write!(fmt, " ]")
        }
    }
}
impl std::ops::Deref for Trace {
    type Target = Vec<(Loc, usize)>;
    fn deref(&self) -> &Vec<(Loc, usize)> {
        &self.trace
    }
}

impl Trace {
    /// Trace constructor.
    pub fn new(trace: Vec<(Loc, usize)>) -> Self {
        Self { trace }
    }
}

/// A list of labels.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Labels {
    labels: Vec<String>,
}
swarkn::display! {
    impl for Labels {
        self, fmt => {
            write!(fmt, "[")?;
            for label in &self.labels {
                write!(fmt, " `{}`", label)?
            }
            write!(fmt, " ]")
        }
    }
}
impl Labels {
    /// Trace constructor.
    pub fn new(labels: Vec<String>) -> Self {
        Self { labels }
    }
}

/// A kind of allocation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AllocKind {
    Minor,
    Major,
    MajorPostponed,
    Serialized,
    Unknown,
}
swarkn::display! {
    impl for AllocKind {
        self, fmt => write!(fmt, "{}", self.as_str())
    }
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

    /// Parses an allocation kind.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use alloc_data::{AllocKind, AllocKind::*};
    /// let s_list = [
    ///     ("Minor", Minor),
    ///     ("Major", Major),
    ///     ("MajorPostponed", MajorPostponed),
    ///     ("Serialized", Serialized),
    /// ];
    /// for (s, exp) in &s_list {
    ///     let kind = AllocKind::from_str(s).unwrap();
    ///     assert_eq! { kind, *exp }
    /// }
    /// ```
    pub fn from_str<Str: AsRef<str>>(s: Str) -> Res<Self> {
        Parser::parse_all(s.as_ref(), Parser::kind, "allocation kind")
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
    pub size: usize,
    /// Allocation-site callstack.
    pub trace: Trace,
    /// User-defined labels.
    pub labels: Vec<String>,
    /// Time of creation.
    pub toc: SinceStart,
    /// Time of death.
    pub tod: Option<SinceStart>,
}
swarkn::display! {
    impl for Alloc {
        self, fmt => {
            let mut labels = "[".to_string();
            for label in &self.labels {
                labels.push_str(" ");
                labels.push_str(label)
            }
            labels.push_str(" ]");
            write!(
                fmt,
                "{}: {}, {}, {}, {}, {}, ",
                self.uid, self.kind, self.size, self.trace, labels, self.toc
            )?;
            if let Some(tod) = &self.tod {
                write!(fmt, "{}", tod)?
            } else {
                write!(fmt, "_")?
            }
            write!(fmt, " }}")
        }
    }
}

impl Alloc {
    /// Constructor.
    pub fn new(
        uid: Uid,
        kind: AllocKind,
        size: usize,
        trace: Trace,
        labels: Vec<String>,
        toc: SinceStart,
        tod: Option<SinceStart>,
    ) -> Self {
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

    /// UID accessor.
    pub fn uid(&self) -> &Uid {
        &self.uid
    }
    /// Kind accessor.
    pub fn kind(&self) -> &AllocKind {
        &self.kind
    }
    /// Size accessor (in machine words).
    pub fn size(&self) -> usize {
        self.size
    }
    /// Trace accessor.
    pub fn trace(&self) -> &Trace {
        &self.trace
    }
    /// Time of creation accessor.
    pub fn toc(&self) -> SinceStart {
        self.toc
    }
    /// Time of death accessor.
    pub fn tod(&self) -> Option<SinceStart> {
        self.tod
    }

    /// Parser.
    pub fn from_str<Str: AsRef<str>>(s: Str) -> Res<Self> {
        Parser::parse_all(s.as_ref(), Parser::alloc, "allocation")
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
swarkn::display! {
    impl for Diff {
        self, fmt => {
            write!(fmt, "{}; new: {{\n", self.time)?;
            for alloc in &self.new {
                write!(fmt, "    {},\n", alloc)?
            }
            write!(fmt, "}};\ndead {{\n")?;
            for (uid, date) in &self.dead {
                write!(fmt, "    #{}: {},\n", uid, date)?
            }
            write!(fmt, "}}\n")
        }
    }
}

impl Diff {
    /// Constructor.
    pub fn new(time: SinceStart, new: Vec<Alloc>, dead: Vec<(Uid, SinceStart)>) -> Self {
        Self { time, new, dead }
    }

    /// Parser.
    pub fn from_str<Str: AsRef<str>>(s: Str) -> Res<Self> {
        Parser::parse_all(s.as_ref(), Parser::diff, "diff")
    }
}

/// Data from a memthol init file.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Init {
    /// The start time of the run: an absolute date.
    pub start_time: Date,
    /// Size of machine words in bytes.
    pub word_size: usize,
}
impl Init {
    /// Constructor.
    pub fn new(start_time: Date, word_size: usize) -> Self {
        Self {
            start_time,
            word_size,
        }
    }

    /// Parses a string to construct itself.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use alloc_data::Init;
    /// let txt = "start: 1566489242.007000572\nword_size: 4\n";
    /// let init = Init::from_str(txt).unwrap();
    /// assert_eq! { init.to_string(), "start: 2019-08-22 15:54:02.007000572 UTC\nword_size: 4\n" }
    /// ```
    pub fn from_str<Str>(txt: Str) -> Res<Self>
    where
        Str: AsRef<str>,
    {
        let txt = txt.as_ref();
        let mut parser = Parser::new(txt);
        parser.init()
    }
}
swarkn::display! {
    impl for Init {
        self, fmt => {
            writeln!(fmt, "start: {}", self.start_time)?;
            writeln!(fmt, "word_size: {}", self.word_size)?;
            Ok(())
        }
    }
}

/// Trait for types that can be parsed.
pub trait Parseable: Sized {
    /// Parses something.
    fn parse<Str>(text: Str) -> Res<Self>
    where
        Str: AsRef<str>;
}
impl Parseable for usize {
    fn parse<Str>(text: Str) -> Res<Self>
    where
        Str: AsRef<str>,
    {
        use swarkn::parse::ParserExt;
        Parser::parse_all(text.as_ref(), Parser::usize, "usize")
    }
}

impl Parseable for SinceStart {
    fn parse<Str>(text: Str) -> Res<Self>
    where
        Str: AsRef<str>,
    {
        Self::from_str(text)
    }
}
