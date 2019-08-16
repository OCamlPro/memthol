//! Types and parsers for memthol's dump structures.

use std::{fmt, str::FromStr, time::Duration};

pub use num_bigint::BigUint;

use serde_derive::Serialize;

pub mod parser;

pub use parser::Parser;

lazy_static::lazy_static! {
    /// Span separator.
    static ref SPAN_SEP: char = '-';
    /// Span format.
    static ref SPAN_FMT: String = format!("<int>{}<int>", *SPAN_SEP);

    /// Location separator.
    static ref LOC_SEP: char = ':';
    /// Location counter separator.
    static ref LOC_COUNT_SEP: char = '#';
    /// Location format.
    static ref LOC_FMT: String = format!("<file>{}<line>{}{}", *LOC_SEP, *LOC_SEP, *SPAN_FMT);
    /// Location with count format.
    static ref LOC_COUNT_FMT: String = format!("{}{}<int>", *LOC_FMT, *LOC_COUNT_SEP);

    /// None format.
    static ref NONE_FMT: String = format!("<none>");

    /// Trace start.
    static ref TRACE_START: char = '[';
    /// Trace end.
    static ref TRACE_END: char = ']';
    /// Trace format.
    static ref TRACE_FMT: String = format!(
        "{} $( {} | `{}` )* {}",
        *TRACE_START, *LOC_COUNT_FMT, *NONE_FMT, *TRACE_END
    );
    /// Label format.
    static ref LABEL: String = "`<anything but `>`".into();
    /// Labels format.
    static ref LABELS_FMT: String = format!(
        "{} $( {} )* {}",
        *TRACE_START, *LABEL, *TRACE_END
    );

    /// Date format.
    static ref DATE_FMT: String = "<int>$(`.`<int>)?".into();

    /// Allocation kind format.
    static ref ALLOC_KIND_FMT: String = "`Minor`|`Major`|`MajorPostponed`|`Serialized`".into();

    /// Allocation separator.
    static ref ALLOC_SEP: char = ',';
    /// Allocation format.
    static ref ALLOC_FMT: String = format!(
        "<uid>: <kind>{} <size>{} <trace>{} <created_at> $({} <died_at>)?",
        *ALLOC_SEP, *ALLOC_SEP, *ALLOC_SEP, *ALLOC_SEP
    );

    /// Diff separator.
    static ref DIFF_SEP: char = ';';
    /// Diff inner separator.
    static ref DIFF_INNER_SEP: char = '|';
    /// Diff format.
    static ref DIFF_FMT: String = format!(
        "\
            <timestamp> {} `new` {{ \
                $(`{}` <alloc>)* `{}` \
            }} {} dead {{ \
                $(`{}` <uid>: <died_at>)* `{}` \
            }}\
        ",
        *DIFF_SEP, *DIFF_INNER_SEP, *DIFF_INNER_SEP, *DIFF_SEP, *DIFF_INNER_SEP, *DIFF_INNER_SEP
    );
}

macro_rules! error {
    (unwrap_option ( $($e:expr),* $(,)* ), $illegal:expr, msg: $msg:expr) => {{
        #[allow(unused_parens)]
        (
            $(
                match $e {
                    Some(res) => res,
                    None => error!($illegal, msg: $msg),
                }
            ),*
        )
    }};
    (unwrap ( $($e:expr),* $(,)* ), $illegal:expr, msg: $msg:expr) => {{
        #[allow(unused_parens)]
        (
            $(
                match $e {
                    Ok(res) => res,
                    Err(e) => error!($illegal, msg: format!("{} ({})", $msg, e)),
                }
            ),*
        )
    }};
    ($illegal:expr, msg: $msg:expr) => {
        return Err(format!("parse error: `{}` {}", $illegal, $msg));
    };
}

/// A bigint UID.
#[derive(Serialize, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Uid {
    /// The actual bigint.
    uid: BigUint,
}
impl fmt::Display for Uid {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        self.uid.fmt(fmt)
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
    /// let uid = Uid::of_str(s).unwrap();
    /// # println!("uid: {}", uid);
    /// assert_eq! { format!("{}", uid), s }
    /// ```
    ///
    /// ```rust
    /// use alloc_data::Uid;
    /// let s = "643128653641564321563425361425364523164523164";
    /// let uid = Uid::of_str(s).unwrap();
    /// # println!("uid: {}", uid);
    /// assert_eq! { format!("{}", uid), s }
    /// ```
    pub fn of_str(s: &str) -> Result<Self, String> {
        let uid = error!(
            unwrap_option(BigUint::parse_bytes(s.as_bytes(), 10)),
            s, msg: "expected UID (integer)"
        );
        Ok(Self { uid })
    }
}

/// A location.
#[derive(Serialize, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Loc {
    /// File the location is for.
    pub file: String,
    /// Line in the file.
    pub line: usize,
    /// Column span at that line in the file.
    pub span: (usize, usize),
}
impl fmt::Display for Loc {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(
            fmt,
            "{}:{}:{}-{}",
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
    /// let s = "blah/stuff/file.ml:325:7-38";
    /// let loc = Loc::of_str(s).unwrap();
    /// # println!("loc: {}", loc);
    /// assert_eq! { format!("{}", loc), s }
    /// assert_eq! { loc.file, "blah/stuff/file.ml" }
    /// assert_eq! { loc.line, 325 }
    /// assert_eq! { loc.span, (7, 38) }
    /// ```
    pub fn of_str(s: &str) -> Result<Self, String> {
        let err_msg = || format!("expected location: {}", *LOC_FMT);
        let mut subs = s.split(*LOC_SEP);
        let (file, line, span) = error!(
            unwrap_option(subs.next().map(str::to_string), subs.next(), subs.next(),),
            s,
            msg: err_msg()
        );

        let line = error!(unwrap(usize::from_str(line)), line, msg: err_msg());

        let mut subs = span.split(*SPAN_SEP);
        let (col_start, col_end) = error!(
            unwrap_option(subs.next(), subs.next()),
            span,
            msg: err_msg()
        );
        let span = error!(
            unwrap(usize::from_str(col_start), usize::from_str(col_end)),
            col_start,
            msg: err_msg()
        );

        Ok(Self { file, line, span })
    }

    /// Parses a location with a count.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use alloc_data::Loc;
    /// let s = "blah/stuff/file.ml:325:7-38#5";
    /// let (loc, count) = Loc::of_str_with_count(s).unwrap();
    /// # println!("loc_count: {}#{}", loc, count);
    /// assert_eq! { format!("{}", loc), s[0..s.len()-2] }
    /// assert_eq! { loc.file, "blah/stuff/file.ml" }
    /// assert_eq! { loc.line, 325 }
    /// assert_eq! { loc.span, (7, 38) }
    /// assert_eq! { count, 5 }
    /// ```
    pub fn of_str_with_count(s: &str) -> Result<(Self, usize), String> {
        let err_msg = || format!("expected location with count: {}", *LOC_COUNT_FMT);
        let mut subs = s.split(*LOC_COUNT_SEP);
        let (loc, count) = error!(unwrap_option(subs.next(), subs.next()), s, msg: err_msg());
        let loc = Self::of_str(loc)?;
        let count = error!(unwrap(usize::from_str(count)), s, msg: err_msg());
        Ok((loc, count))
    }
}

/// Parses a list between `TRACE_START` and `TRACE_END`.
pub trait TraceLike<Elm> {
    /// Name of the actual trace-like thing.
    const NAME: &'static str;
    /// Format of the inner element, for error-reporting.
    fn format<'a>() -> &'a str;

    /// Parses an inner element.
    fn parse_inner(s: &str) -> Result<Elm, String>;

    /// Parses a trace-like string.
    fn parse(s: &str) -> Result<Vec<Elm>, String> {
        let err_msg = || format!("expected {}: {}", Self::NAME, Self::format());
        let mut chars = s.chars();
        if chars.next() != Some(*TRACE_START) || chars.next_back() != Some(*TRACE_END) {
            return Err(err_msg());
        }
        let s = s[1..s.len() - 1].trim();

        let mut vec = Vec::with_capacity(13);
        for inner in s.split_whitespace() {
            vec.push(Self::parse_inner(inner)?)
        }
        Ok(vec)
    }
}

/// A trace of locations.
#[derive(Serialize, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Trace {
    /// The actual trace of locations.
    trace: Vec<(Loc, usize)>,
}
impl fmt::Display for Trace {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "[")?;
        for (loc, count) in &self.trace {
            write!(fmt, " {}#{}", loc, count)?
        }
        write!(fmt, " ]")
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
#[derive(Serialize, Debug, Clone)]
pub struct Labels {
    labels: Vec<String>,
}
impl fmt::Display for Labels {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "[")?;
        for label in &self.labels {
            write!(fmt, " `{}`", label)?
        }
        write!(fmt, " ]")
    }
}
impl Labels {
    /// Trace constructor.
    pub fn new(labels: Vec<String>) -> Self {
        Self { labels }
    }
}
impl TraceLike<String> for Labels {
    const NAME: &'static str = "labels";

    fn format<'a>() -> &'a str {
        "`<anything but `>`"
    }

    /// Parses an inner element.
    fn parse_inner(s: &str) -> Result<String, String> {
        let mut chars = s.chars();
        if chars.next() != Some('`') || chars.next_back() != Some('`') {
            error!(s, msg: format!("expected label: {}", *LABELS_FMT));
        }

        Ok(s[1..s.len() - 1].into())
    }
}

/// A kind of allocation.
#[derive(Serialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum AllocKind {
    Minor,
    Major,
    MajorPostponed,
    Serialized,
}
impl fmt::Display for AllocKind {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{}", self.as_str())
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
    ///     let kind = AllocKind::of_str(s).unwrap();
    ///     assert_eq! { kind, *exp }
    /// }
    /// ```
    pub fn of_str(s: &str) -> Result<Self, String> {
        use AllocKind::*;
        match s {
            "Minor" => Ok(Minor),
            "Major" => Ok(Major),
            "MajorPostponed" => Ok(MajorPostponed),
            "Serialized" => Ok(Serialized),
            s => Err(format!(
                "found `{}` while parsing allocation kind {}",
                s, *ALLOC_KIND_FMT
            )),
        }
    }
}

/// Wrapper around a duration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, PartialOrd, Ord)]
pub struct Date {
    /// Actual duration.
    duration: Duration,
}
impl fmt::Display for Date {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        let mut nanos = format!(".{:>09}", self.duration.subsec_nanos());
        // Remove trailing zeros.
        loop {
            match nanos.pop() {
                // Remove zeros.
                Some('0') => (),
                // There was nothing but zeros, remove dot as well (last character).
                Some('.') => break,
                // Otherwise it's a number, we must keep it and stop removing stuff.
                Some(c) => {
                    nanos.push(c);
                    break;
                }
                None => unreachable!(),
            }
        }
        write!(fmt, "{}{}", self.duration.as_secs(), nanos)
    }
}
impl std::ops::Deref for Date {
    type Target = Duration;
    fn deref(&self) -> &Duration {
        &self.duration
    }
}
impl From<std::time::Duration> for Date {
    fn from(duration: Duration) -> Self {
        Self { duration }
    }
}
impl std::ops::Sub for Date {
    type Output = Self;
    fn sub(self, other: Self) -> Self {
        Date {
            duration: self.duration - other.duration,
        }
    }
}

impl Date {
    /// Date parser.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use std::time::Duration;
    /// use alloc_data::Date;
    /// let s_list = [
    ///     ("320.74", Duration::new(320, 740_000_000)),
    ///     ("703470.0074", Duration::new(703470, 7_400_000)),
    ///     ("0.2", Duration::new(0, 200_000_000)),
    ///     ("7", Duration::new(7, 0)),
    /// ];
    /// for (s, exp) in &s_list {
    ///     let date = Date::of_str(s).unwrap();
    ///     assert_eq! { &*date, exp }
    /// }
    /// ```
    pub fn of_str(s: &str) -> Result<Self, String> {
        let err_msg = || format!("found `{}`, expected date {}", s, *DATE_FMT);
        let mut subs = s.split('.');
        let secs = error!(unwrap_option(subs.next()), s, msg: err_msg());
        let secs = error!(unwrap(u64::from_str(secs)), s, msg: err_msg());
        let nanos = match subs.next() {
            None => 0,
            Some(nanos) => {
                let nanos = &format!("{:0<9}", nanos);
                if nanos.len() > 9 {
                    error!(s, msg: err_msg())
                }

                error!(unwrap(u32::from_str(nanos)), s, msg: err_msg())
            }
        };
        let duration = Duration::new(secs, nanos);
        Ok(Self { duration })
    }
}

/// Some allocation information.
#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
pub struct Alloc {
    /// Uid of the allocation.
    pub uid: Uid,
    /// Allocation kind.
    pub kind: AllocKind,
    /// Size of the allocation.
    pub size: usize,
    /// Allocation-site callstack.
    pub trace: Trace,
    /// Time of creation.
    pub toc: Date,
    /// Time of death.
    pub tod: Option<Date>,
}
impl fmt::Display for Alloc {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(
            fmt,
            "{}: {}, {}, {}, {}",
            self.uid, self.kind, self.size, self.trace, self.toc
        )?;
        if let Some(tod) = &self.tod {
            write!(fmt, ", {}", tod)?
        }
        write!(fmt, " }}")
    }
}

impl Alloc {
    /// Constructor.
    pub fn new(
        uid: Uid,
        kind: AllocKind,
        size: usize,
        trace: Trace,
        toc: Date,
        tod: Option<Date>,
    ) -> Self {
        Self {
            uid,
            kind,
            size,
            trace,
            toc,
            tod,
        }
    }

    /// Sets the time of death.
    ///
    /// Bails if a time of death is already registered.
    pub fn set_tod(&mut self, tod: Date) -> Result<(), String> {
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
    /// Size accessor.
    pub fn size(&self) -> usize {
        self.size
    }
    /// Trace accessor.
    pub fn trace(&self) -> &Trace {
        &self.trace
    }
    /// Time of creation accessor.
    pub fn toc(&self) -> Date {
        self.toc
    }
    /// Time of death accessor.
    pub fn tod(&self) -> Option<Date> {
        self.tod
    }

    /// Parser.
    pub fn of_str<Str: AsRef<str>>(txt: Str) -> Result<Self, String> {
        let mut parser = Parser::new(txt.as_ref());
        let alloc = parser.alloc()?;
        parser.ws();
        if !parser.is_eoi() {
            return Err(parser.error("expected end of input"));
        }
        Ok(alloc)
    }
}

/// A diff.
///
/// **NB:** `Display` for this type is multi-line.
#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
pub struct Diff {
    /// Timestamp.
    pub time: Date,
    /// New allocations in this diff.
    pub new: Vec<Alloc>,
    /// Data freed in this diff.
    pub dead: Vec<(Uid, Date)>,
}
impl fmt::Display for Diff {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
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

impl Diff {
    /// Constructor.
    pub fn new(time: Date, new: Vec<Alloc>, dead: Vec<(Uid, Date)>) -> Self {
        Self { time, new, dead }
    }

    /// Parser.
    pub fn of_str<Str: AsRef<str>>(txt: Str) -> Result<Self, String> {
        let mut parser = Parser::new(txt.as_ref());
        let diff = parser.diff()?;
        parser.ws();
        if !parser.is_eoi() {
            return Err(parser.error("expected end of input"));
        }
        Ok(diff)
    }
}

use stdweb::*;

js_serializable! { Uid }
js_serializable! { Date }
js_serializable! { Alloc }
js_serializable! { Diff }
