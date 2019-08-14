//! Types and parsers for memthol's dump structures.

use std::{fmt, str::FromStr, time::Duration};

pub use num_bigint::BigUint;

use serde_derive::Serialize;

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

/// A trace of locations.
#[derive(Serialize, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Trace {
    /// The actual trace of locations.
    trace: Vec<Option<(Loc, usize)>>,
}
impl fmt::Display for Trace {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "[")?;
        for loc in &self.trace {
            if let Some((loc, count)) = loc {
                write!(fmt, " {}#{}", loc, count)?
            } else {
                write!(fmt, " <none>")?
            }
        }
        write!(fmt, " ]")
    }
}
impl std::ops::Deref for Trace {
    type Target = Vec<Option<(Loc, usize)>>;
    fn deref(&self) -> &Vec<Option<(Loc, usize)>> {
        &self.trace
    }
}

impl Trace {
    /// Trace constructor.
    pub fn new(trace: Vec<Option<(Loc, usize)>>) -> Self {
        Self { trace }
    }

    /// Parses a trace of locations.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use alloc_data::Trace;
    /// let (loc_0, count_0) = ("blah/stuff/file.ml:325:7-38", 5);
    /// let (loc_1, count_1) = ("blah/other_file.ml:3:1-3", 2);
    /// let (loc_2, count_2) = ("blah/ya_file.ml:5243:70-72", 7);
    /// let (loc_3, count_3) = ("last_file.ml:73:3-3", 11);
    /// let s = format!(r#"[
    ///     {}#{}
    ///     {}#{}
    ///     {}#{}
    ///     {}#{}
    /// ]"#, loc_0, count_0, loc_1, count_1, loc_2, count_2, loc_3, count_3);
    /// let trace = Trace::of_str(&s).unwrap();
    /// # println!("trace: {}", trace);
    /// assert_eq! { format!("{}", trace[0].as_ref().unwrap().0), loc_0 }
    /// assert_eq! { trace[0].as_ref().unwrap().1, count_0 }
    /// assert_eq! { format!("{}", trace[1].as_ref().unwrap().0), loc_1 }
    /// assert_eq! { trace[1].as_ref().unwrap().1, count_1 }
    /// assert_eq! { format!("{}", trace[2].as_ref().unwrap().0), loc_2 }
    /// assert_eq! { trace[2].as_ref().unwrap().1, count_2 }
    /// assert_eq! { format!("{}", trace[3].as_ref().unwrap().0), loc_3 }
    /// assert_eq! { trace[3].as_ref().unwrap().1, count_3 }
    /// ```
    pub fn of_str(s: &str) -> Result<Self, String> {
        let err_msg = || format!("expected trace of locations: {}", *TRACE_FMT);

        let mut subs = s.split_whitespace();

        let trace_start = error!(unwrap_option(subs.next()), s, msg: err_msg());

        let mut trace = vec![];

        if !(trace_start.len() == 1 && trace_start.chars().nth(0) == Some(*TRACE_START)) {
            error!(s, msg: err_msg())
        }

        loop {
            let next = error!(unwrap_option(subs.next()), s, msg: err_msg());

            if next.len() == 1 && next.chars().nth(0) == Some(*TRACE_END) {
                break;
            }

            let loc_count = if next == *NONE_FMT {
                None
            } else {
                Some(Loc::of_str_with_count(next)?)
            };

            trace.push(loc_count)
        }

        Ok(Self { trace })
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
    uid: Uid,
    /// Allocation kind.
    kind: AllocKind,
    /// Size of the allocation.
    size: usize,
    /// Allocation-site callstack.
    trace: Trace,
    /// Time of creation.
    toc: Date,
    /// Time of death.
    tod: Option<Date>,
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

    /// Parses an allocation.
    ///
    /// # Examples
    ///
    /// Without a time of death:
    ///
    /// ```rust
    /// use alloc_data::*;
    /// let (uid, kind, size, trace, toc) = (
    ///     "7254", "Major", "8", "[ blah/stuff/file.ml:325:7-38#3 file.ml:754230:1-3#11 ]", "327.2"
    /// );
    /// let s = format!("{}: {}, {}, {}, {}", uid, kind, size, trace, toc);
    /// let alloc = Alloc::of_str(&s).unwrap();
    /// assert_eq! { format!("{}", alloc.uid()), uid }
    /// assert_eq! { format!("{}", alloc.kind()), kind }
    /// assert_eq! { format!("{}", alloc.size()), size }
    /// assert_eq! { format!("{}", alloc.trace()), trace }
    /// assert_eq! { format!("{}", alloc.toc()), toc }
    /// assert_eq! { alloc.tod(), None }
    /// ```
    ///
    /// With a time of death:
    ///
    /// ```rust
    /// use alloc_data::*;
    /// let (uid, kind, size, trace, toc, tod) = (
    ///     "7254", "Major", "8", "[ blah/stuff/file.ml:325:7-38#3 file.ml:754230:1-3#11 ]",
    ///     "5.2", "18.3"
    /// );
    /// let s = format!("{}: {}, {}, {}, {}, {}", uid, kind, size, trace, toc, tod);
    /// let alloc = Alloc::of_str(&s).unwrap();
    /// assert_eq! { format!("{}", alloc.uid()), uid }
    /// assert_eq! { format!("{}", alloc.kind()), kind }
    /// assert_eq! { format!("{}", alloc.size()), size }
    /// assert_eq! { format!("{}", alloc.trace()), trace }
    /// assert_eq! { format!("{}", alloc.toc()), toc }
    /// assert_eq! { format!("{}", alloc.tod().unwrap()), tod }
    /// ```
    pub fn of_str(s: &str) -> Result<Self, String> {
        let err_msg = || format!("expected allocation: {}", *ALLOC_FMT);;
        let mut subs = s.split(*ALLOC_SEP).map(str::trim);
        let (uid_kind, size, trace, toc) = error! {
            unwrap_option(
                subs.next(), subs.next(), subs.next(), subs.next()
            ), s, msg: err_msg()
        };
        let tod = subs.next();

        let mut subs = uid_kind.split(':').map(str::trim);
        let (uid, kind) = error! {
            unwrap_option(
                subs.next(), subs.next()
            ), s, msg: err_msg()
        };

        let uid = Uid::of_str(uid)?;

        let size = error!(unwrap(usize::from_str(size)), s, msg: err_msg());
        let kind = AllocKind::of_str(kind)?;
        let trace = Trace::of_str(trace)?;
        let toc = Date::of_str(toc)?;

        let tod = if let Some(tod) = tod {
            Some(Date::of_str(tod)?)
        } else {
            None
        };

        Ok(Self {
            uid,
            kind,
            size,
            trace,
            toc,
            tod,
        })
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
    /// Diff parser.
    pub fn of_str(s: &str) -> Result<Self, String> {
        let err = || format!("illegal diff, exected syntax: `{}`", *DIFF_FMT);
        let mut subs = s.split(*DIFF_SEP).map(str::trim);
        let (time, new, dead) = (
            subs.next().ok_or_else(err)?,
            subs.next().ok_or_else(err)?,
            subs.next().ok_or_else(err)?,
        );

        let time = Date::of_str(time)?;

        let new = {
            let mut subs = new.split(*DIFF_INNER_SEP).map(str::trim);

            let mut first_subs = subs.next().ok_or_else(err)?.split_whitespace();
            if first_subs.next().ok_or_else(err)? != "new"
                || first_subs.next().ok_or_else(err)? != "{"
            {
                return Err(err());
            }

            let mut new = vec![];

            loop {
                let next = subs.next().ok_or_else(err)?;
                if next == "}" {
                    break;
                }

                let alloc = Alloc::of_str(next)?;
                new.push(alloc)
            }

            new
        };

        let dead = {
            let mut subs = dead.split(*DIFF_INNER_SEP).map(str::trim);

            let first_sub = subs.next().ok_or_else(err)?;
            let mut first_subs = first_sub.split_whitespace();
            if first_subs.next().ok_or_else(err)? != "dead"
                || first_subs.next().ok_or_else(err)? != "{"
            {
                return Err(err());
            }

            let mut dead = vec![];

            loop {
                let next = subs.next().ok_or_else(err)?;
                if next == "}" {
                    break;
                }

                let mut subs = next.split(':').map(str::trim);
                let uid = subs.next().ok_or_else(err)?;
                let uid = Uid::of_str(uid)?;
                let tod = subs.next().ok_or_else(err)?;
                let tod = Date::of_str(tod)?;
                dead.push((uid, tod))
            }

            dead
        };

        Ok(Diff { time, new, dead })
    }
}

use stdweb::*;

js_serializable! { Uid }
js_serializable! { Date }
js_serializable! { Alloc }
js_serializable! { Diff }

#[cfg(test)]
mod diff {
    use super::*;

    static DIFF_1: &str = r#"
0.006 ;
new {
    | 48: Minor, 211, [ src/test.ml:36:24-64#89 src/test.ml:48:21-28#1 src/test.ml:55:12-21#2 src/test.ml:77:12-30#1 src/memthol.ml:96:22-26#1 src/test.ml:71:8-240#1 ], 0.006
    | 47: Minor, 586, [ src/test.ml:36:24-64#73 src/test.ml:48:21-28#1 src/test.ml:55:12-21#2 src/test.ml:77:12-30#1 src/memthol.ml:96:22-26#1 src/test.ml:71:8-240#1 ], 0.006
    | 46: Minor, 5, [ set.ml:130:21-28#1 set.ml:133:21-28#1 set.ml:130:21-28#2 set.ml:133:21-28#1 set.ml:130:21-28#2 set.ml:133:21-28#2 src/test.ml:36:24-46#1 src/test.ml:36:24-64#24 src/test.ml:48:21-28#1 src/test.ml:55:12-21#2 src/test.ml:77:12-30#1 src/memthol.ml:96:22-26#1 src/test.ml:71:8-240#1 ], 0.006
    | 45: Minor, 537, [ set.ml:117:25-41#1 set.ml:133:21-28#1 set.ml:130:21-28#3 set.ml:133:21-28#2 src/test.ml:36:24-46#1 src/test.ml:36:24-64#21 src/test.ml:48:21-28#1 src/test.ml:55:12-21#2 src/test.ml:77:12-30#1 src/memthol.ml:96:22-26#1 src/test.ml:71:8-240#1 ], 0.006
    | 44: Minor, 393, [ src/test.ml:36:24-64#18 src/test.ml:48:21-28#1 src/test.ml:55:12-21#2 src/test.ml:77:12-30#1 src/memthol.ml:96:22-26#1 src/test.ml:71:8-240#1 ], 0.006
    | 43: Minor, 411, [ set.ml:133:21-28#1 set.ml:130:21-28#1 src/test.ml:38:20-42#1 src/test.ml:36:24-64#11 src/test.ml:48:21-28#1 src/test.ml:55:12-21#2 src/test.ml:77:12-30#1 src/memthol.ml:96:22-26#1 src/test.ml:71:8-240#1 ], 0.006
    | 42: Minor, 354, [ src/test.ml:36:24-64#54 src/test.ml:48:21-28#1 src/test.ml:55:12-21#2 src/test.ml:77:12-30#1 src/memthol.ml:96:22-26#1 src/test.ml:71:8-240#1 ], 0.006
    | 41: Minor, 462, [ set.ml:133:21-28#1 set.ml:130:21-28#1 set.ml:133:21-28#5 set.ml:130:21-28#1 src/test.ml:36:24-46#1 src/test.ml:36:24-64#13 src/test.ml:48:21-28#1 src/test.ml:55:12-21#2 src/test.ml:77:12-30#1 src/memthol.ml:96:22-26#1 src/test.ml:71:8-240#1 ], 0.005
    | 40: Minor, 414, [ src/test.ml:36:24-64#15 src/test.ml:48:21-28#1 src/test.ml:55:12-21#2 src/test.ml:77:12-30#1 src/memthol.ml:96:22-26#1 src/test.ml:71:8-240#1 ], 0.005
    | 39: Minor, 110, [ src/test.ml:36:24-64#20 src/test.ml:48:21-28#1 src/test.ml:55:12-21#2 src/test.ml:77:12-30#1 src/memthol.ml:96:22-26#1 src/test.ml:71:8-240#1 ], 0.005
    | 38: Minor, 283, [ src/test.ml:36:24-64#33 src/test.ml:48:21-28#1 src/test.ml:55:12-21#2 src/test.ml:77:12-30#1 src/memthol.ml:96:22-26#1 src/test.ml:71:8-240#1 ], 0.005
    | 37: Minor, 560, [ src/test.ml:36:24-64#47 src/test.ml:48:21-28#1 src/test.ml:55:12-21#2 src/test.ml:77:12-30#1 src/memthol.ml:96:22-26#1 src/test.ml:71:8-240#1 ], 0.005
    | 36: Minor, 683, [ src/test.ml:36:24-64#20 src/test.ml:48:21-28#1 src/test.ml:55:12-21#2 src/test.ml:77:12-30#1 src/memthol.ml:96:22-26#1 src/test.ml:71:8-240#1 ], 0.005
    | 35: Minor, 37, [ src/test.ml:36:24-64#25 src/test.ml:48:21-28#1 src/test.ml:55:12-21#2 src/test.ml:77:12-30#1 src/memthol.ml:96:22-26#1 src/test.ml:71:8-240#1 ], 0.005
    | 34: Minor, 486, [ src/test.ml:36:24-64#27 src/test.ml:48:21-28#1 src/test.ml:55:12-21#2 src/test.ml:77:12-30#1 src/memthol.ml:96:22-26#1 src/test.ml:71:8-240#1 ], 0.005
    | 33: Minor, 16, [ src/test.ml:36:24-64#33 src/test.ml:48:21-28#1 src/test.ml:55:12-21#2 src/test.ml:77:12-30#1 src/memthol.ml:96:22-26#1 src/test.ml:71:8-240#1 ], 0.005
    | 32: Minor, 525, [ src/test.ml:36:24-64#42 src/test.ml:48:21-28#1 src/test.ml:55:12-21#2 src/test.ml:77:12-30#1 src/memthol.ml:96:22-26#1 src/test.ml:71:8-240#1 ], 0.005
    | 31: Minor, 548, [ src/test.ml:36:24-64#48 src/test.ml:48:21-28#1 src/test.ml:55:12-21#2 src/test.ml:77:12-30#1 src/memthol.ml:96:22-26#1 src/test.ml:71:8-240#1 ], 0.005
    | 30: Minor, 476, [ src/test.ml:36:24-64#36 src/test.ml:48:21-28#1 src/test.ml:55:12-21#2 src/test.ml:77:12-30#1 src/memthol.ml:96:22-26#1 src/test.ml:71:8-240#1 ], 0.005
    | 29: Minor, 8, [ src/test.ml:36:24-64#40 src/test.ml:48:21-28#1 src/test.ml:55:12-21#2 src/test.ml:77:12-30#1 src/memthol.ml:96:22-26#1 src/test.ml:71:8-240#1 ], 0.005
    | 28: Minor, 531, [ set.ml:130:21-28#1 set.ml:133:21-28#1 src/test.ml:36:24-46#1 src/test.ml:36:24-64#15 src/test.ml:48:21-28#1 src/test.ml:55:12-21#2 src/test.ml:77:12-30#1 src/memthol.ml:96:22-26#1 src/test.ml:71:8-240#1 ], 0.005
    | 27: Minor, 642, [ src/test.ml:36:24-64#15 src/test.ml:48:21-28#1 src/test.ml:55:12-21#2 src/test.ml:77:12-30#1 src/memthol.ml:96:22-26#1 src/test.ml:71:8-240#1 ], 0.004, 0.005
    | 26: Minor, 133, [ set.ml:133:21-28#1 set.ml:130:21-28#1 src/test.ml:38:20-42#1 src/test.ml:36:24-64#12 src/test.ml:48:21-28#1 src/test.ml:55:12-21#2 src/test.ml:77:12-30#1 src/memthol.ml:96:22-26#1 src/test.ml:71:8-240#1 ], 0.004, 0.005
    | 25: Minor, 552, [ set.ml:130:21-28#1 src/test.ml:38:20-42#1 src/test.ml:36:24-64#14 src/test.ml:48:21-28#1 src/test.ml:55:12-21#2 src/test.ml:77:12-30#1 src/memthol.ml:96:22-26#1 src/test.ml:71:8-240#1 ], 0.004, 0.005
    | 24: Minor, 687, [ set.ml:130:21-28#1 set.ml:133:21-28#1 src/test.ml:38:20-42#1 src/test.ml:36:24-64#1 src/test.ml:48:21-28#1 src/test.ml:55:12-21#2 src/test.ml:77:12-30#1 src/memthol.ml:96:22-26#1 src/test.ml:71:8-240#1 ], 0.004, 0.005
    | 23: Minor, 84, [ set.ml:133:21-28#5 set.ml:130:21-28#2 src/test.ml:38:20-42#1 src/test.ml:36:24-64#1 src/test.ml:48:21-28#1 src/test.ml:55:12-21#2 src/test.ml:77:12-30#1 src/memthol.ml:96:22-26#1 src/test.ml:71:8-240#1 ], 0.004, 0.005
    | 22: Minor, 163, [ src/test.ml:36:24-64#4 src/test.ml:48:21-28#1 src/test.ml:55:12-21#2 src/test.ml:77:12-30#1 src/memthol.ml:96:22-26#1 src/test.ml:71:8-240#1 ], 0.004, 0.005
    | 21: Minor, 611, [ set.ml:133:21-28#1 set.ml:130:21-28#1 set.ml:133:21-28#4 set.ml:130:21-28#1 src/test.ml:38:20-42#1 src/test.ml:36:24-64#1 src/test.ml:48:21-28#1 src/test.ml:55:12-21#2 src/test.ml:77:12-30#1 src/memthol.ml:96:22-26#1 src/test.ml:71:8-240#1 ], 0.004, 0.005
    | 20: Minor, 661, [ src/test.ml:36:24-64#31 src/test.ml:48:21-28#1 src/test.ml:55:12-21#2 src/test.ml:77:12-30#1 src/memthol.ml:96:22-26#1 src/test.ml:71:8-240#1 ], 0.004, 0.005
    | 19: Minor, 149, [ set.ml:133:21-28#4 set.ml:130:21-28#1 set.ml:133:21-28#1 src/test.ml:36:24-46#1 src/test.ml:48:21-28#1 src/test.ml:55:12-21#2 src/test.ml:77:12-30#1 src/memthol.ml:96:22-26#1 src/test.ml:71:8-240#1 ], 0.004, 0.005
    | 18: Minor, 526, [ set.ml:133:21-28#1 src/test.ml:38:20-42#1 src/test.ml:36:24-64#2 src/test.ml:48:21-28#1 src/test.ml:55:12-21#2 src/test.ml:77:12-30#1 src/memthol.ml:96:22-26#1 src/test.ml:71:8-240#1 ], 0.004, 0.005
    | 17: Minor, 167, [ src/test.ml:36:24-64#22 src/test.ml:48:21-28#1 src/test.ml:55:12-21#2 src/test.ml:77:12-30#1 src/memthol.ml:96:22-26#1 src/test.ml:71:8-240#1 ], 0.004, 0.005
    | 16: Minor, 64, [ src/test.ml:36:24-64#32 src/test.ml:48:21-28#1 src/test.ml:55:12-21#2 src/test.ml:77:12-30#1 src/memthol.ml:96:22-26#1 src/test.ml:71:8-240#1 ], 0.004, 0.005
    | 15: Minor, 509, [ src/test.ml:36:24-64#54 src/test.ml:48:21-28#1 src/test.ml:55:12-21#2 src/test.ml:77:12-30#1 src/memthol.ml:96:22-26#1 src/test.ml:71:8-240#1 ], 0.004, 0.005
    | 14: Minor, 384, [ set.ml:133:21-28#1 src/test.ml:36:24-46#1 src/test.ml:36:24-64#5 src/test.ml:48:21-28#1 src/test.ml:55:12-21#2 src/test.ml:77:12-30#1 src/memthol.ml:96:22-26#1 src/test.ml:71:8-240#1 ], 0.004, 0.005
    | 13: Minor, 542, [ set.ml:133:21-28#3 set.ml:130:21-28#1 set.ml:133:21-28#1 src/test.ml:36:24-46#1 src/test.ml:36:24-64#5 src/test.ml:48:21-28#1 src/test.ml:55:12-21#2 src/test.ml:77:12-30#1 src/memthol.ml:96:22-26#1 src/test.ml:71:8-240#1 ], 0.004, 0.005
    | 12: Minor, 695, [ src/test.ml:36:24-64#50 src/test.ml:48:21-28#1 src/test.ml:55:12-21#2 src/test.ml:77:12-30#1 src/memthol.ml:96:22-26#1 src/test.ml:71:8-240#1 ], 0.003, 0.005
    | 11: Minor, 395, [ src/test.ml:36:24-64#8 src/test.ml:48:21-28#1 src/test.ml:55:12-21#2 src/test.ml:77:12-30#1 src/memthol.ml:96:22-26#1 src/test.ml:71:8-240#1 ], 0.003, 0.005
    | 10: Minor, 510, [ src/test.ml:36:24-64#25 src/test.ml:48:21-28#1 src/test.ml:55:12-21#2 src/test.ml:77:12-30#1 src/memthol.ml:96:22-26#1 src/test.ml:71:8-240#1 ], 0.003, 0.005
    | 9: Minor, 29, [ src/test.ml:36:24-64#25 src/test.ml:48:21-28#1 src/test.ml:55:12-21#2 src/test.ml:77:12-30#1 src/memthol.ml:96:22-26#1 src/test.ml:71:8-240#1 ], 0.003, 0.005
    | 8: Minor, 349, [ src/test.ml:36:24-46#1 src/test.ml:36:24-64#6 src/test.ml:48:21-28#1 src/test.ml:55:12-21#2 src/test.ml:77:12-30#1 src/memthol.ml:96:22-26#1 src/test.ml:71:8-240#1 ], 0.003, 0.005
    | 7: Minor, 57, [ set.ml:133:21-28#2 set.ml:130:21-28#1 set.ml:133:21-28#2 set.ml:130:21-28#1 src/test.ml:38:20-42#1 src/test.ml:36:24-64#9 src/test.ml:48:21-28#1 src/test.ml:55:12-21#2 src/test.ml:77:12-30#1 src/memthol.ml:96:22-26#1 src/test.ml:71:8-240#1 ], 0.003, 0.005
    | 6: Minor, 389, [ set.ml:133:21-28#1 src/test.ml:36:24-46#1 src/test.ml:36:24-64#2 src/test.ml:48:21-28#1 src/test.ml:55:12-21#2 src/test.ml:77:12-30#1 src/memthol.ml:96:22-26#1 src/test.ml:71:8-240#1 ], 0.003, 0.005
    | 5: Minor, 665, [ set.ml:130:21-28#4 src/test.ml:36:24-46#1 src/test.ml:36:24-64#3 src/test.ml:48:21-28#1 src/test.ml:55:12-21#2 src/test.ml:77:12-30#1 src/memthol.ml:96:22-26#1 src/test.ml:71:8-240#1 ], 0.003, 0.005
    | 4: Minor, 649, [ set.ml:105:48-64#1 set.ml:133:21-28#2 set.ml:130:21-28#1 src/test.ml:38:20-42#1 src/test.ml:36:24-64#2 src/test.ml:48:21-28#1 src/test.ml:55:12-21#2 src/test.ml:77:12-30#1 src/memthol.ml:96:22-26#1 src/test.ml:71:8-240#1 ], 0.003, 0.005
    | 3: Minor, 270, [ set.ml:105:48-64#1 set.ml:130:21-28#1 src/test.ml:38:20-42#1 src/test.ml:36:24-64#1 src/test.ml:48:21-28#1 src/test.ml:55:12-21#2 src/test.ml:77:12-30#1 src/memthol.ml:96:22-26#1 src/test.ml:71:8-240#1 ], 0.003, 0.005
    | 2: Minor, 139, [ src/test.ml:36:24-64#53 src/test.ml:48:21-28#1 src/test.ml:55:12-21#2 src/test.ml:76:12-30#1 src/memthol.ml:96:22-26#1 src/test.ml:71:8-240#1 ], 0.003, 0.005
    | 1: Minor, 588, [ src/test.ml:36:24-64#26 src/test.ml:48:21-28#1 src/test.ml:55:12-21#2 src/test.ml:76:12-30#1 src/memthol.ml:96:22-26#1 src/test.ml:71:8-240#1 ], 0.002, 0.005
    | 0: Minor, 517, [ src/test.ml:36:24-64#12 src/test.ml:48:21-28#1 src/test.ml:55:12-21#2 src/test.ml:76:12-30#1 src/memthol.ml:96:22-26#1 src/test.ml:71:8-240#1 ], 0.002, 0.005
    |
} ; dead { | }
    "#;

    #[test]
    fn test_1() {
        if let Err(e) = Diff::of_str(DIFF_1) {
            println!("|===| Error:");
            for line in format!("{}", e).lines() {
                println!("| {}", line)
            }
            panic!("test failed")
        }
    }

    static DIFF_2: &str = r#"
0.039 ;
new {
    | 212: Minor, 182, [ set.ml:133:21-28#7 set.ml:130:21-28#2 src/test.ml:38:20-42#1 src/test.ml:36:24-64#26 src/test.ml:48:21-28#1 src/test.ml:55:12-21#2 src/test.ml:77:12-30#1 src/memthol.ml:96:22-26#1 src/test.ml:71:8-240#1 ], 0.039
    | 211: Minor, 620, [ set.ml:130:21-28#2 src/test.ml:38:20-42#1 src/test.ml:36:24-64#27 src/test.ml:48:21-28#1 src/test.ml:55:12-21#2 src/test.ml:77:12-30#1 src/memthol.ml:96:22-26#1 src/test.ml:71:8-240#1 ], 0.039
    | 210: Minor, 549, [ set.ml:133:21-28#1 set.ml:130:21-28#1 set.ml:133:21-28#7 set.ml:130:21-28#2 src/test.ml:38:20-42#1 src/test.ml:36:24-64#27 src/test.ml:48:21-28#1 src/test.ml:55:12-21#2 src/test.ml:77:12-30#1 src/memthol.ml:96:22-26#1 src/test.ml:71:8-240#1 ], 0.039
    | 209: Minor, 223, [ set.ml:105:25-43#1 set.ml:133:21-28#6 set.ml:130:21-28#2 src/test.ml:38:20-42#1 src/test.ml:36:24-64#28 src/test.ml:48:21-28#1 src/test.ml:55:12-21#2 src/test.ml:77:12-30#1 src/memthol.ml:96:22-26#1 src/test.ml:71:8-240#1 ], 0.039
    | 208: Minor, 269, [ src/test.ml:36:24-64#123 src/test.ml:48:21-28#1 src/test.ml:55:12-21#2 src/test.ml:77:12-30#1 src/memthol.ml:96:22-26#1 src/test.ml:71:8-240#1 ], 0.038
    | 207: Minor, 520, [ src/test.ml:36:24-64#157 src/test.ml:48:21-28#1 src/test.ml:55:12-21#2 src/test.ml:77:12-30#1 src/memthol.ml:96:22-26#1 src/test.ml:71:8-240#1 ], 0.038
    | 206: Minor, 277, [ src/test.ml:36:24-64#167 src/test.ml:48:21-28#1 src/test.ml:55:12-21#2 src/test.ml:77:12-30#1 src/memthol.ml:96:22-26#1 src/test.ml:71:8-240#1 ], 0.038
    | 205: Minor, 185, [ src/test.ml:36:24-64#36 src/test.ml:48:21-28#1 src/test.ml:55:12-21#2 src/test.ml:77:12-30#1 src/memthol.ml:96:22-26#1 src/test.ml:71:8-240#1 ], 0.038
    | 204: Minor, 166, [ src/test.ml:36:24-64#154 src/test.ml:48:21-28#1 src/test.ml:55:12-21#2 src/test.ml:77:12-30#1 src/memthol.ml:96:22-26#1 src/test.ml:71:8-240#1 ], 0.038
    | 203: Minor, 431, [ src/test.ml:36:24-64#162 src/test.ml:48:21-28#1 src/test.ml:55:12-21#2 src/test.ml:77:12-30#1 src/memthol.ml:96:22-26#1 src/test.ml:71:8-240#1 ], 0.037, 0.037
    | 202: Minor, 489, [ src/test.ml:36:24-64#188 src/test.ml:48:21-28#1 src/test.ml:55:12-21#2 src/test.ml:77:12-30#1 src/memthol.ml:96:22-26#1 src/test.ml:71:8-240#1 ], 0.037, 0.037
    | 201: Minor, 554, [ set.ml:133:21-28#5 set.ml:130:21-28#1 src/test.ml:36:24-46#1 src/test.ml:36:24-64#30 src/test.ml:48:21-28#1 src/test.ml:55:12-21#2 src/test.ml:77:12-30#1 src/memthol.ml:96:22-26#1 src/test.ml:71:8-240#1 ], 0.037, 0.037
    | 200: Minor, 594, [ src/test.ml:36:24-64#97 src/test.ml:48:21-28#1 src/test.ml:55:12-21#2 src/test.ml:77:12-30#1 src/memthol.ml:96:22-26#1 src/test.ml:71:8-240#1 ], 0.037, 0.037
    | 199: Minor, 671, [ src/test.ml:36:24-64#36 src/test.ml:48:21-28#1 src/test.ml:55:12-21#2 src/test.ml:77:12-30#1 src/memthol.ml:96:22-26#1 src/test.ml:71:8-240#1 ], 0.037, 0.037
    | 198: Minor, 277, [ src/test.ml:36:24-64#49 src/test.ml:48:21-28#1 src/test.ml:55:12-21#2 src/test.ml:77:12-30#1 src/memthol.ml:96:22-26#1 src/test.ml:71:8-240#1 ], 0.036, 0.037
    | 197: Minor, 235, [ src/test.ml:36:24-64#146 src/test.ml:48:21-28#1 src/test.ml:55:12-21#2 src/test.ml:77:12-30#1 src/memthol.ml:96:22-26#1 src/test.ml:71:8-240#1 ], 0.036, 0.037
    | 196: Minor, 177, [ src/test.ml:36:24-64#232 src/test.ml:48:21-28#1 src/test.ml:55:12-21#2 src/test.ml:77:12-30#1 src/memthol.ml:96:22-26#1 src/test.ml:71:8-240#1 ], 0.036, 0.037
    | 195: Minor, 672, [ src/test.ml:36:24-64#233 src/test.ml:48:21-28#1 src/test.ml:55:12-21#2 src/test.ml:77:12-30#1 src/memthol.ml:96:22-26#1 src/test.ml:71:8-240#1 ], 0.036, 0.037
    | 194: Minor, 38, [ src/test.ml:36:24-64#53 src/test.ml:48:21-28#1 src/test.ml:55:12-21#2 src/test.ml:77:12-30#1 src/memthol.ml:96:22-26#1 src/test.ml:71:8-240#1 ], 0.036, 0.037
    | 193: Minor, 430, [ src/test.ml:36:24-64#123 src/test.ml:48:21-28#1 src/test.ml:55:12-21#2 src/test.ml:77:12-30#1 src/memthol.ml:96:22-26#1 src/test.ml:71:8-240#1 ], 0.035, 0.037
    | 192: Minor, 557, [ src/test.ml:36:24-64#128 src/test.ml:48:21-28#1 src/test.ml:55:12-21#2 src/test.ml:77:12-30#1 src/memthol.ml:96:22-26#1 src/test.ml:71:8-240#1 ], 0.035, 0.035
    | 191: Minor, 120, [ src/test.ml:36:24-64#43 src/test.ml:48:21-28#1 src/test.ml:55:12-21#2 src/test.ml:77:12-30#1 src/memthol.ml:96:22-26#1 src/test.ml:71:8-240#1 ], 0.035, 0.035
    | 190: Minor, 494, [ src/test.ml:36:24-64#163 src/test.ml:48:21-28#1 src/test.ml:55:12-21#2 src/test.ml:77:12-30#1 src/memthol.ml:96:22-26#1 src/test.ml:71:8-240#1 ], 0.035, 0.035
    | 189: Minor, 57, [ src/test.ml:36:24-64#133 src/test.ml:48:21-28#1 src/test.ml:55:12-21#2 src/test.ml:77:12-30#1 src/memthol.ml:96:22-26#1 src/test.ml:71:8-240#1 ], 0.035, 0.035
    | 188: Minor, 177, [ src/test.ml:36:24-64#124 src/test.ml:48:21-28#1 src/test.ml:55:12-21#2 src/test.ml:77:12-30#1 src/memthol.ml:96:22-26#1 src/test.ml:71:8-240#1 ], 0.034, 0.035
    | 187: Minor, 387, [ src/test.ml:36:24-64#124 src/test.ml:48:21-28#1 src/test.ml:55:12-21#2 src/test.ml:77:12-30#1 src/memthol.ml:96:22-26#1 src/test.ml:71:8-240#1 ], 0.034, 0.035
    | 186: Minor, 675, [ src/test.ml:36:24-64#132 src/test.ml:48:21-28#1 src/test.ml:55:12-21#2 src/test.ml:77:12-30#1 src/memthol.ml:96:22-26#1 src/test.ml:71:8-240#1 ], 0.034, 0.035
    | 185: Minor, 664, [ set.ml:133:21-28#1 set.ml:130:21-28#1 set.ml:133:21-28#7 set.ml:130:21-28#1 src/test.ml:38:20-42#1 src/test.ml:36:24-64#39 src/test.ml:48:21-28#1 src/test.ml:55:12-21#2 src/test.ml:77:12-30#1 src/memthol.ml:96:22-26#1 src/test.ml:71:8-240#1 ], 0.034, 0.034
    | 184: Minor, 168, [ src/test.ml:36:24-64#136 src/test.ml:48:21-28#1 src/test.ml:55:12-21#2 src/test.ml:77:12-30#1 src/memthol.ml:96:22-26#1 src/test.ml:71:8-240#1 ], 0.034, 0.034
    | 183: Minor, 661, [ src/test.ml:36:24-64#212 src/test.ml:48:21-28#1 src/test.ml:55:12-21#2 src/test.ml:77:12-30#1 src/memthol.ml:96:22-26#1 src/test.ml:71:8-240#1 ], 0.033, 0.034
    | 182: Minor, 467, [ src/test.ml:36:24-64#188 src/test.ml:48:21-28#1 src/test.ml:55:12-21#2 src/test.ml:77:12-30#1 src/memthol.ml:96:22-26#1 src/test.ml:71:8-240#1 ], 0.033, 0.034
    | 181: Minor, 68, [ src/test.ml:36:24-64#122 src/test.ml:48:21-28#1 src/test.ml:55:12-21#2 src/test.ml:77:12-30#1 src/memthol.ml:96:22-26#1 src/test.ml:71:8-240#1 ], 0.033, 0.034
    | 180: Minor, 542, [ src/test.ml:36:24-64#77 src/test.ml:48:21-28#1 src/test.ml:55:12-21#2 src/test.ml:77:12-30#1 src/memthol.ml:96:22-26#1 src/test.ml:71:8-240#1 ], 0.033, 0.034
    | 179: Minor, 383, [ src/test.ml:36:24-64#47 src/test.ml:48:21-28#1 src/test.ml:55:12-21#2 src/test.ml:77:12-30#1 src/memthol.ml:96:22-26#1 src/test.ml:71:8-240#1 ], 0.032, 0.034
    | 178: Minor, 441, [ src/test.ml:36:24-64#122 src/test.ml:48:21-28#1 src/test.ml:55:12-21#2 src/test.ml:77:12-30#1 src/memthol.ml:96:22-26#1 src/test.ml:71:8-240#1 ], 0.032, 0.034
    | 177: Minor, 681, [ src/test.ml:36:24-64#132 src/test.ml:48:21-28#1 src/test.ml:55:12-21#2 src/test.ml:77:12-30#1 src/memthol.ml:96:22-26#1 src/test.ml:71:8-240#1 ], 0.031, 0.032
    | 176: Minor, 216, [ src/test.ml:36:24-64#213 src/test.ml:48:21-28#1 src/test.ml:55:12-21#2 src/test.ml:77:12-30#1 src/memthol.ml:96:22-26#1 src/test.ml:71:8-240#1 ], 0.031, 0.032
    | 175: Minor, 400, [ set.ml:133:21-28#1 set.ml:130:21-28#1 src/test.ml:36:24-46#1 src/test.ml:36:24-64#43 src/test.ml:48:21-28#1 src/test.ml:55:12-21#2 src/test.ml:77:12-30#1 src/memthol.ml:96:22-26#1 src/test.ml:71:8-240#1 ], 0.031, 0.032
    | 174: Minor, 515, [ set.ml:133:21-28#1 set.ml:130:21-28#1 src/test.ml:38:20-42#1 src/test.ml:36:24-64#43 src/test.ml:48:21-28#1 src/test.ml:55:12-21#2 src/test.ml:77:12-30#1 src/memthol.ml:96:22-26#1 src/test.ml:71:8-240#1 ], 0.031, 0.031
    | 173: Minor, 596, [ src/test.ml:36:24-64#195 src/test.ml:48:21-28#1 src/test.ml:55:12-21#2 src/test.ml:77:12-30#1 src/memthol.ml:96:22-26#1 src/test.ml:71:8-240#1 ], 0.031, 0.031
    | 172: Minor, 319, [ src/test.ml:36:24-64#177 src/test.ml:48:21-28#1 src/test.ml:55:12-21#2 src/test.ml:77:12-30#1 src/memthol.ml:96:22-26#1 src/test.ml:71:8-240#1 ], 0.031, 0.031
    | 171: Minor, 156, [ set.ml:133:21-28#5 set.ml:130:21-28#1 src/test.ml:36:24-46#1 src/test.ml:36:24-64#44 src/test.ml:48:21-28#1 src/test.ml:55:12-21#2 src/test.ml:77:12-30#1 src/memthol.ml:96:22-26#1 src/test.ml:71:8-240#1 ], 0.030, 0.031
    | 170: Minor, 615, [ src/test.ml:36:24-64#184 src/test.ml:48:21-28#1 src/test.ml:55:12-21#2 src/test.ml:77:12-30#1 src/memthol.ml:96:22-26#1 src/test.ml:71:8-240#1 ], 0.030, 0.031
    | 169: Minor, 661, [ src/test.ml:36:24-64#207 src/test.ml:48:21-28#1 src/test.ml:55:12-21#2 src/test.ml:77:12-30#1 src/memthol.ml:96:22-26#1 src/test.ml:71:8-240#1 ], 0.030, 0.031
    | 168: Minor, 274, [ src/test.ml:36:24-64#211 src/test.ml:48:21-28#1 src/test.ml:55:12-21#2 src/test.ml:77:12-30#1 src/memthol.ml:96:22-26#1 src/test.ml:71:8-240#1 ], 0.030, 0.031
    | 167: Minor, 3, [ src/test.ml:36:24-64#100 src/test.ml:48:21-28#1 src/test.ml:55:12-21#2 src/test.ml:77:12-30#1 src/memthol.ml:96:22-26#1 src/test.ml:71:8-240#1 ], 0.029, 0.030
    | 166: Minor, 583, [ src/test.ml:36:24-64#195 src/test.ml:48:21-28#1 src/test.ml:55:12-21#2 src/test.ml:77:12-30#1 src/memthol.ml:96:22-26#1 src/test.ml:71:8-240#1 ], 0.029, 0.030
    | 165: Minor, 288, [ src/test.ml:36:24-64#158 src/test.ml:48:21-28#1 src/test.ml:55:12-21#2 src/test.ml:77:12-30#1 src/memthol.ml:96:22-26#1 src/test.ml:71:8-240#1 ], 0.029, 0.030
    | 164: Minor, 300, [ set.ml:133:21-28#6 set.ml:130:21-28#2 src/test.ml:38:20-42#1 src/test.ml:36:24-64#51 src/test.ml:48:21-28#1 src/test.ml:55:12-21#2 src/test.ml:77:12-30#1 src/memthol.ml:96:22-26#1 src/test.ml:71:8-240#1 ], 0.028, 0.030
    | 163: Minor, 599, [ src/test.ml:36:24-64#99 src/test.ml:48:21-28#1 src/test.ml:55:12-21#2 src/test.ml:77:12-30#1 src/memthol.ml:96:22-26#1 src/test.ml:71:8-240#1 ], 0.028, 0.030
    | 162: Minor, 252, [ src/test.ml:36:24-64#169 src/test.ml:48:21-28#1 src/test.ml:55:12-21#2 src/test.ml:77:12-30#1 src/memthol.ml:96:22-26#1 src/test.ml:71:8-240#1 ], 0.028, 0.030
    | 161: Minor, 275, [ src/test.ml:36:24-64#164 src/test.ml:48:21-28#1 src/test.ml:55:12-21#2 src/test.ml:77:12-30#1 src/memthol.ml:96:22-26#1 src/test.ml:71:8-240#1 ], 0.028, 0.028
    | 160: Minor, 541, [ src/test.ml:36:24-64#181 src/test.ml:48:21-28#1 src/test.ml:55:12-21#2 src/test.ml:77:12-30#1 src/memthol.ml:96:22-26#1 src/test.ml:71:8-240#1 ], 0.028, 0.028
    | 159: Minor, 171, [ src/test.ml:36:24-64#181 src/test.ml:48:21-28#1 src/test.ml:55:12-21#2 src/test.ml:77:12-30#1 src/memthol.ml:96:22-26#1 src/test.ml:71:8-240#1 ], 0.027, 0.028
    | 158: Minor, 238, [ src/test.ml:36:24-64#196 src/test.ml:48:21-28#1 src/test.ml:55:12-21#2 src/test.ml:77:12-30#1 src/memthol.ml:96:22-26#1 src/test.ml:71:8-240#1 ], 0.027, 0.028
    | 157: Minor, 78, [ src/test.ml:36:24-64#129 src/test.ml:48:21-28#1 src/test.ml:55:12-21#2 src/test.ml:77:12-30#1 src/memthol.ml:96:22-26#1 src/test.ml:71:8-240#1 ], 0.027, 0.028
    | 156: Minor, 616, [ src/test.ml:36:24-64#105 src/test.ml:48:21-28#1 src/test.ml:55:12-21#2 src/test.ml:77:12-30#1 src/memthol.ml:96:22-26#1 src/test.ml:71:8-240#1 ], 0.027, 0.028
    | 155: Minor, 259, [ set.ml:130:21-28#1 set.ml:133:21-28#6 set.ml:130:21-28#2 src/test.ml:38:20-42#1 src/test.ml:36:24-64#56 src/test.ml:48:21-28#1 src/test.ml:55:12-21#2 src/test.ml:77:12-30#1 src/memthol.ml:96:22-26#1 src/test.ml:71:8-240#1 ], 0.026, 0.027
    | 154: Minor, 433, [ src/test.ml:36:24-64#194 src/test.ml:48:21-28#1 src/test.ml:55:12-21#2 src/test.ml:77:12-30#1 src/memthol.ml:96:22-26#1 src/test.ml:71:8-240#1 ], 0.026, 0.027
    | 153: Minor, 157, [ src/test.ml:36:24-64#171 src/test.ml:48:21-28#1 src/test.ml:55:12-21#2 src/test.ml:77:12-30#1 src/memthol.ml:96:22-26#1 src/test.ml:71:8-240#1 ], 0.026, 0.027
    | 152: Minor, 392, [ src/test.ml:36:24-64#177 src/test.ml:48:21-28#1 src/test.ml:55:12-21#2 src/test.ml:77:12-30#1 src/memthol.ml:96:22-26#1 src/test.ml:71:8-240#1 ], 0.026, 0.027
    | 151: Minor, 43, [ src/test.ml:36:24-64#161 src/test.ml:48:21-28#1 src/test.ml:55:12-21#2 src/test.ml:77:12-30#1 src/memthol.ml:96:22-26#1 src/test.ml:71:8-240#1 ], 0.025, 0.027
    | 150: Minor, 48, [ src/test.ml:36:24-64#112 src/test.ml:48:21-28#1 src/test.ml:55:12-21#2 src/test.ml:77:12-30#1 src/memthol.ml:96:22-26#1 src/test.ml:71:8-240#1 ], 0.025, 0.027
    | 149: Minor, 238, [ src/test.ml:36:24-64#141 src/test.ml:48:21-28#1 src/test.ml:55:12-21#2 src/test.ml:77:12-30#1 src/memthol.ml:96:22-26#1 src/test.ml:71:8-240#1 ], 0.025, 0.025
    | 148: Minor, 691, [ src/test.ml:36:24-64#147 src/test.ml:48:21-28#1 src/test.ml:55:12-21#2 src/test.ml:77:12-30#1 src/memthol.ml:96:22-26#1 src/test.ml:71:8-240#1 ], 0.025, 0.025
    | 147: Minor, 487, [ src/test.ml:36:24-64#179 src/test.ml:48:21-28#1 src/test.ml:55:12-21#2 src/test.ml:77:12-30#1 src/memthol.ml:96:22-26#1 src/test.ml:71:8-240#1 ], 0.024, 0.025
    | 146: Minor, 681, [ src/test.ml:36:24-46#1 src/test.ml:36:24-64#59 src/test.ml:48:21-28#1 src/test.ml:55:12-21#2 src/test.ml:77:12-30#1 src/memthol.ml:96:22-26#1 src/test.ml:71:8-240#1 ], 0.024, 0.024
    | 145: Minor, 23, [ set.ml:133:21-28#2 set.ml:130:21-28#1 src/test.ml:38:20-42#1 src/test.ml:36:24-64#61 src/test.ml:48:21-28#1 src/test.ml:55:12-21#2 src/test.ml:77:12-30#1 src/memthol.ml:96:22-26#1 src/test.ml:71:8-240#1 ], 0.024, 0.024
    | 144: Minor, 527, [ set.ml:133:21-28#5 set.ml:130:21-28#2 src/test.ml:38:20-42#1 src/test.ml:36:24-64#63 src/test.ml:48:21-28#1 src/test.ml:55:12-21#2 src/test.ml:77:12-30#1 src/memthol.ml:96:22-26#1 src/test.ml:71:8-240#1 ], 0.024, 0.024
    | 143: Minor, 476, [ set.ml:130:21-28#1 set.ml:133:21-28#6 set.ml:130:21-28#2 src/test.ml:38:20-42#1 src/test.ml:36:24-64#63 src/test.ml:48:21-28#1 src/test.ml:55:12-21#2 src/test.ml:77:12-30#1 src/memthol.ml:96:22-26#1 src/test.ml:71:8-240#1 ], 0.024, 0.024
    | 142: Minor, 680, [ src/test.ml:36:24-64#122 src/test.ml:48:21-28#1 src/test.ml:55:12-21#2 src/test.ml:77:12-30#1 src/memthol.ml:96:22-26#1 src/test.ml:71:8-240#1 ], 0.024, 0.024
    | 141: Minor, 482, [ src/test.ml:36:24-64#127 src/test.ml:48:21-28#1 src/test.ml:55:12-21#2 src/test.ml:77:12-30#1 src/memthol.ml:96:22-26#1 src/test.ml:71:8-240#1 ], 0.023, 0.024
    | 140: Minor, 371, [ src/test.ml:36:24-64#170 src/test.ml:48:21-28#1 src/test.ml:55:12-21#2 src/test.ml:77:12-30#1 src/memthol.ml:96:22-26#1 src/test.ml:71:8-240#1 ], 0.023, 0.024
    | 139: Minor, 549, [ src/test.ml:36:24-64#160 src/test.ml:48:21-28#1 src/test.ml:55:12-21#2 src/test.ml:77:12-30#1 src/memthol.ml:96:22-26#1 src/test.ml:71:8-240#1 ], 0.023, 0.024
    | 138: Minor, 316, [ src/test.ml:36:24-64#181 src/test.ml:48:21-28#1 src/test.ml:55:12-21#2 src/test.ml:77:12-30#1 src/memthol.ml:96:22-26#1 src/test.ml:71:8-240#1 ], 0.022, 0.022
    | 137: Minor, 610, [ src/test.ml:36:24-64#76 src/test.ml:48:21-28#1 src/test.ml:55:12-21#2 src/test.ml:77:12-30#1 src/memthol.ml:96:22-26#1 src/test.ml:71:8-240#1 ], 0.022, 0.022
    | 136: Minor, 365, [ src/test.ml:36:24-64#161 src/test.ml:48:21-28#1 src/test.ml:55:12-21#2 src/test.ml:77:12-30#1 src/memthol.ml:96:22-26#1 src/test.ml:71:8-240#1 ], 0.022, 0.022
    | 135: Minor, 517, [ src/test.ml:36:24-64#118 src/test.ml:48:21-28#1 src/test.ml:55:12-21#2 src/test.ml:77:12-30#1 src/memthol.ml:96:22-26#1 src/test.ml:71:8-240#1 ], 0.022, 0.022
    | 134: Minor, 603, [ src/test.ml:36:24-64#79 src/test.ml:48:21-28#1 src/test.ml:55:12-21#2 src/test.ml:77:12-30#1 src/memthol.ml:96:22-26#1 src/test.ml:71:8-240#1 ], 0.021, 0.022
    | 133: Minor, 632, [ src/test.ml:36:24-64#127 src/test.ml:48:21-28#1 src/test.ml:55:12-21#2 src/test.ml:77:12-30#1 src/memthol.ml:96:22-26#1 src/test.ml:71:8-240#1 ], 0.021, 0.022
    | 132: Minor, 393, [ src/test.ml:36:24-64#155 src/test.ml:48:21-28#1 src/test.ml:55:12-21#2 src/test.ml:77:12-30#1 src/memthol.ml:96:22-26#1 src/test.ml:71:8-240#1 ], 0.021, 0.022
    | 131: Minor, 3, [ src/test.ml:36:24-64#179 src/test.ml:48:21-28#1 src/test.ml:55:12-21#2 src/test.ml:77:12-30#1 src/memthol.ml:96:22-26#1 src/test.ml:71:8-240#1 ], 0.021, 0.021
    | 130: Minor, 446, [ src/test.ml:36:24-64#174 src/test.ml:48:21-28#1 src/test.ml:55:12-21#2 src/test.ml:77:12-30#1 src/memthol.ml:96:22-26#1 src/test.ml:71:8-240#1 ], 0.021, 0.021
    | 129: Minor, 211, [ src/test.ml:36:24-64#92 src/test.ml:48:21-28#1 src/test.ml:55:12-21#2 src/test.ml:77:12-30#1 src/memthol.ml:96:22-26#1 src/test.ml:71:8-240#1 ], 0.020, 0.021
    | 128: Minor, 665, [ src/test.ml:36:24-64#105 src/test.ml:48:21-28#1 src/test.ml:55:12-21#2 src/test.ml:77:12-30#1 src/memthol.ml:96:22-26#1 src/test.ml:71:8-240#1 ], 0.020, 0.021
    | 127: Minor, 535, [ src/test.ml:36:24-64#93 src/test.ml:48:21-28#1 src/test.ml:55:12-21#2 src/test.ml:77:12-30#1 src/memthol.ml:96:22-26#1 src/test.ml:71:8-240#1 ], 0.020, 0.021
    | 126: Minor, 423, [ src/test.ml:36:24-64#137 src/test.ml:48:21-28#1 src/test.ml:55:12-21#2 src/test.ml:77:12-30#1 src/memthol.ml:96:22-26#1 src/test.ml:71:8-240#1 ], 0.020, 0.021
    | 125: Minor, 594, [ src/test.ml:36:24-64#165 src/test.ml:48:21-28#1 src/test.ml:55:12-21#2 src/test.ml:77:12-30#1 src/memthol.ml:96:22-26#1 src/test.ml:71:8-240#1 ], 0.020, 0.021
    | 124: Minor, 219, [ src/test.ml:36:24-64#131 src/test.ml:48:21-28#1 src/test.ml:55:12-21#2 src/test.ml:77:12-30#1 src/memthol.ml:96:22-26#1 src/test.ml:71:8-240#1 ], 0.019, 0.019
    | 123: Minor, 58, [ src/test.ml:36:24-64#101 src/test.ml:48:21-28#1 src/test.ml:55:12-21#2 src/test.ml:77:12-30#1 src/memthol.ml:96:22-26#1 src/test.ml:71:8-240#1 ], 0.019, 0.019
    | 122: Minor, 466, [ src/test.ml:36:24-64#114 src/test.ml:48:21-28#1 src/test.ml:55:12-21#2 src/test.ml:77:12-30#1 src/memthol.ml:96:22-26#1 src/test.ml:71:8-240#1 ], 0.019, 0.019
    | 121: Minor, 623, [ src/test.ml:36:24-64#162 src/test.ml:48:21-28#1 src/test.ml:55:12-21#2 src/test.ml:77:12-30#1 src/memthol.ml:96:22-26#1 src/test.ml:71:8-240#1 ], 0.019, 0.019
    | 120: Minor, 127, [ src/test.ml:36:24-64#92 src/test.ml:48:21-28#1 src/test.ml:55:12-21#2 src/test.ml:77:12-30#1 src/memthol.ml:96:22-26#1 src/test.ml:71:8-240#1 ], 0.018, 0.019
    | 119: Minor, 88, [ src/test.ml:36:24-64#93 src/test.ml:48:21-28#1 src/test.ml:55:12-21#2 src/test.ml:77:12-30#1 src/memthol.ml:96:22-26#1 src/test.ml:71:8-240#1 ], 0.018, 0.019
    | 118: Minor, 614, [ src/test.ml:36:24-64#137 src/test.ml:48:21-28#1 src/test.ml:55:12-21#2 src/test.ml:77:12-30#1 src/memthol.ml:96:22-26#1 src/test.ml:71:8-240#1 ], 0.018, 0.019
    | 117: Minor, 295, [ src/test.ml:36:24-64#116 src/test.ml:48:21-28#1 src/test.ml:55:12-21#2 src/test.ml:77:12-30#1 src/memthol.ml:96:22-26#1 src/test.ml:71:8-240#1 ], 0.018, 0.019
    | 116: Minor, 377, [ src/test.ml:36:24-64#140 src/test.ml:48:21-28#1 src/test.ml:55:12-21#2 src/test.ml:77:12-30#1 src/memthol.ml:96:22-26#1 src/test.ml:71:8-240#1 ], 0.017, 0.018
    | 115: Minor, 561, [ src/test.ml:36:24-64#107 src/test.ml:48:21-28#1 src/test.ml:55:12-21#2 src/test.ml:77:12-30#1 src/memthol.ml:96:22-26#1 src/test.ml:71:8-240#1 ], 0.017, 0.018
    | 114: Minor, 655, [ src/test.ml:36:24-64#94 src/test.ml:48:21-28#1 src/test.ml:55:12-21#2 src/test.ml:77:12-30#1 src/memthol.ml:96:22-26#1 src/test.ml:71:8-240#1 ], 0.017, 0.018
    | 113: Minor, 615, [ src/test.ml:36:24-64#112 src/test.ml:48:21-28#1 src/test.ml:55:12-21#2 src/test.ml:77:12-30#1 src/memthol.ml:96:22-26#1 src/test.ml:71:8-240#1 ], 0.017, 0.018
    | 112: Minor, 358, [ src/test.ml:36:24-64#168 src/test.ml:48:21-28#1 src/test.ml:55:12-21#2 src/test.ml:77:12-30#1 src/memthol.ml:96:22-26#1 src/test.ml:71:8-240#1 ], 0.017, 0.017
    | 111: Minor, 78, [ src/test.ml:36:24-64#119 src/test.ml:48:21-28#1 src/test.ml:55:12-21#2 src/test.ml:77:12-30#1 src/memthol.ml:96:22-26#1 src/test.ml:71:8-240#1 ], 0.016, 0.017
    | 110: Minor, 205, [ src/test.ml:36:24-64#99 src/test.ml:48:21-28#1 src/test.ml:55:12-21#2 src/test.ml:77:12-30#1 src/memthol.ml:96:22-26#1 src/test.ml:71:8-240#1 ], 0.016, 0.017
    | 109: Minor, 9, [ src/test.ml:36:24-64#154 src/test.ml:48:21-28#1 src/test.ml:55:12-21#2 src/test.ml:77:12-30#1 src/memthol.ml:96:22-26#1 src/test.ml:71:8-240#1 ], 0.016, 0.017
    | 108: Minor, 679, [ src/test.ml:36:24-64#112 src/test.ml:48:21-28#1 src/test.ml:55:12-21#2 src/test.ml:77:12-30#1 src/memthol.ml:96:22-26#1 src/test.ml:71:8-240#1 ], 0.016, 0.017
    | 107: Minor, 29, [ src/test.ml:36:24-64#105 src/test.ml:48:21-28#1 src/test.ml:55:12-21#2 src/test.ml:77:12-30#1 src/memthol.ml:96:22-26#1 src/test.ml:71:8-240#1 ], 0.016, 0.017
    | 106: Minor, 619, [ src/test.ml:36:24-64#164 src/test.ml:48:21-28#1 src/test.ml:55:12-21#2 src/test.ml:77:12-30#1 src/memthol.ml:96:22-26#1 src/test.ml:71:8-240#1 ], 0.015, 0.017
    | 105: Minor, 24, [ src/test.ml:36:24-64#113 src/test.ml:48:21-28#1 src/test.ml:55:12-21#2 src/test.ml:77:12-30#1 src/memthol.ml:96:22-26#1 src/test.ml:71:8-240#1 ], 0.015, 0.015
    | 104: Minor, 582, [ src/test.ml:36:24-64#109 src/test.ml:48:21-28#1 src/test.ml:55:12-21#2 src/test.ml:77:12-30#1 src/memthol.ml:96:22-26#1 src/test.ml:71:8-240#1 ], 0.015, 0.015
    | 103: Minor, 82, [ src/test.ml:36:24-64#103 src/test.ml:48:21-28#1 src/test.ml:55:12-21#2 src/test.ml:77:12-30#1 src/memthol.ml:96:22-26#1 src/test.ml:71:8-240#1 ], 0.015, 0.015
    | 102: Minor, 533, [ src/test.ml:36:24-64#138 src/test.ml:48:21-28#1 src/test.ml:55:12-21#2 src/test.ml:77:12-30#1 src/memthol.ml:96:22-26#1 src/test.ml:71:8-240#1 ], 0.014, 0.015
    | 101: Minor, 461, [ src/test.ml:36:24-64#154 src/test.ml:48:21-28#1 src/test.ml:55:12-21#2 src/test.ml:77:12-30#1 src/memthol.ml:96:22-26#1 src/test.ml:71:8-240#1 ], 0.014, 0.015
    | 100: Minor, 359, [ src/test.ml:36:24-64#156 src/test.ml:48:21-28#1 src/test.ml:55:12-21#2 src/test.ml:77:12-30#1 src/memthol.ml:96:22-26#1 src/test.ml:71:8-240#1 ], 0.014, 0.015
    | 99: Minor, 677, [ src/test.ml:36:24-64#116 src/test.ml:48:21-28#1 src/test.ml:55:12-21#2 src/test.ml:77:12-30#1 src/memthol.ml:96:22-26#1 src/test.ml:71:8-240#1 ], 0.014, 0.015
    | 98: Minor, 396, [ src/test.ml:36:24-64#125 src/test.ml:48:21-28#1 src/test.ml:55:12-21#2 src/test.ml:77:12-30#1 src/memthol.ml:96:22-26#1 src/test.ml:71:8-240#1 ], 0.013, 0.013
    | 97: Minor, 429, [ src/test.ml:36:24-64#114 src/test.ml:48:21-28#1 src/test.ml:55:12-21#2 src/test.ml:77:12-30#1 src/memthol.ml:96:22-26#1 src/test.ml:71:8-240#1 ], 0.013, 0.013
    | 96: Minor, 145, [ src/test.ml:36:24-64#149 src/test.ml:48:21-28#1 src/test.ml:55:12-21#2 src/test.ml:77:12-30#1 src/memthol.ml:96:22-26#1 src/test.ml:71:8-240#1 ], 0.013, 0.013
    | 95: Minor, 283, [ src/test.ml:36:24-64#118 src/test.ml:48:21-28#1 src/test.ml:55:12-21#2 src/test.ml:77:12-30#1 src/memthol.ml:96:22-26#1 src/test.ml:71:8-240#1 ], 0.013, 0.013
    | 94: Minor, 379, [ src/test.ml:36:24-64#116 src/test.ml:48:21-28#1 src/test.ml:55:12-21#2 src/test.ml:77:12-30#1 src/memthol.ml:96:22-26#1 src/test.ml:71:8-240#1 ], 0.012, 0.013
    | 93: Minor, 635, [ src/test.ml:36:24-64#154 src/test.ml:48:21-28#1 src/test.ml:55:12-21#2 src/test.ml:77:12-30#1 src/memthol.ml:96:22-26#1 src/test.ml:71:8-240#1 ], 0.012, 0.013
    | 92: Minor, 177, [ src/test.ml:36:24-64#118 src/test.ml:48:21-28#1 src/test.ml:55:12-21#2 src/test.ml:77:12-30#1 src/memthol.ml:96:22-26#1 src/test.ml:71:8-240#1 ], 0.012, 0.013
    | 91: Minor, 257, [ src/test.ml:36:24-64#134 src/test.ml:48:21-28#1 src/test.ml:55:12-21#2 src/test.ml:77:12-30#1 src/memthol.ml:96:22-26#1 src/test.ml:71:8-240#1 ], 0.012, 0.012
    | 90: Minor, 206, [ src/test.ml:36:24-64#134 src/test.ml:48:21-28#1 src/test.ml:55:12-21#2 src/test.ml:77:12-30#1 src/memthol.ml:96:22-26#1 src/test.ml:71:8-240#1 ], 0.011, 0.012
    | 89: Minor, 626, [ src/test.ml:36:24-64#80 src/test.ml:48:21-28#1 src/test.ml:55:12-21#2 src/test.ml:77:12-30#1 src/memthol.ml:96:22-26#1 src/test.ml:71:8-240#1 ], 0.011, 0.012
    | 88: Minor, 132, [ src/test.ml:36:24-46#1 src/test.ml:36:24-64#60 src/test.ml:48:21-28#1 src/test.ml:55:12-21#2 src/test.ml:77:12-30#1 src/memthol.ml:96:22-26#1 src/test.ml:71:8-240#1 ], 0.011, 0.012
    | 87: Minor, 223, [ src/test.ml:36:24-46#1 src/test.ml:36:24-64#59 src/test.ml:48:21-28#1 src/test.ml:55:12-21#2 src/test.ml:77:12-30#1 src/memthol.ml:96:22-26#1 src/test.ml:71:8-240#1 ], 0.011, 0.012
    | 86: Minor, 9, [ set.ml:130:21-28#7 set.ml:133:21-28#2 src/test.ml:36:24-46#1 src/test.ml:36:24-64#43 src/test.ml:48:21-28#1 src/test.ml:55:12-21#2 src/test.ml:77:12-30#1 src/memthol.ml:96:22-26#1 src/test.ml:71:8-240#1 ], 0.011, 0.012
    | 85: Minor, 681, [ set.ml:130:21-28#1 set.ml:133:21-28#2 src/test.ml:36:24-46#1 src/test.ml:36:24-64#37 src/test.ml:48:21-28#1 src/test.ml:55:12-21#2 src/test.ml:77:12-30#1 src/memthol.ml:96:22-26#1 src/test.ml:71:8-240#1 ], 0.011, 0.012
    | 84: Minor, 57, [ src/test.ml:36:24-46#1 src/test.ml:36:24-64#19 src/test.ml:48:21-28#1 src/test.ml:55:12-21#2 src/test.ml:77:12-30#1 src/memthol.ml:96:22-26#1 src/test.ml:71:8-240#1 ], 0.011, 0.012
    | 83: Minor, 533, [ set.ml:130:21-28#1 set.ml:133:21-28#3 set.ml:130:21-28#1 set.ml:133:21-28#2 src/test.ml:36:24-46#1 src/test.ml:36:24-64#14 src/test.ml:48:21-28#1 src/test.ml:55:12-21#2 src/test.ml:77:12-30#1 src/memthol.ml:96:22-26#1 src/test.ml:71:8-240#1 ], 0.011, 0.012
    | 82: Minor, 93, [ set.ml:105:48-64#1 set.ml:133:21-28#5 set.ml:130:21-28#2 src/test.ml:38:20-42#1 src/test.ml:36:24-64#11 src/test.ml:48:21-28#1 src/test.ml:55:12-21#2 src/test.ml:77:12-30#1 src/memthol.ml:96:22-26#1 src/test.ml:71:8-240#1 ], 0.011, 0.012
    | 81: Minor, 474, [ src/test.ml:36:24-64#42 src/test.ml:48:21-28#1 src/test.ml:55:12-21#2 src/test.ml:77:12-30#1 src/memthol.ml:96:22-26#1 src/test.ml:71:8-240#1 ], 0.011, 0.012
    | 80: Minor, 253, [ src/test.ml:36:24-64#45 src/test.ml:48:21-28#1 src/test.ml:55:12-21#2 src/test.ml:77:12-30#1 src/memthol.ml:96:22-26#1 src/test.ml:71:8-240#1 ], 0.011, 0.012
    | 79: Minor, 687, [ src/test.ml:36:24-64#82 src/test.ml:48:21-28#1 src/test.ml:55:12-21#2 src/test.ml:77:12-30#1 src/memthol.ml:96:22-26#1 src/test.ml:71:8-240#1 ], 0.011, 0.012
    | 78: Minor, 290, [ src/test.ml:36:24-64#26 src/test.ml:48:21-28#1 src/test.ml:55:12-21#2 src/test.ml:77:12-30#1 src/memthol.ml:96:22-26#1 src/test.ml:71:8-240#1 ], 0.010, 0.010
    | 77: Minor, 667, [ src/test.ml:36:24-64#113 src/test.ml:48:21-28#1 src/test.ml:55:12-21#2 src/test.ml:77:12-30#1 src/memthol.ml:96:22-26#1 src/test.ml:71:8-240#1 ], 0.010, 0.010
    | 76: Minor, 33, [ src/test.ml:36:24-64#29 src/test.ml:48:21-28#1 src/test.ml:55:12-21#2 src/test.ml:77:12-30#1 src/memthol.ml:96:22-26#1 src/test.ml:71:8-240#1 ], 0.010, 0.010
    | 75: Minor, 288, [ src/test.ml:36:24-64#94 src/test.ml:48:21-28#1 src/test.ml:55:12-21#2 src/test.ml:77:12-30#1 src/memthol.ml:96:22-26#1 src/test.ml:71:8-240#1 ], 0.010, 0.010
    | 74: Minor, 497, [ src/test.ml:36:24-64#55 src/test.ml:48:21-28#1 src/test.ml:55:12-21#2 src/test.ml:77:12-30#1 src/memthol.ml:96:22-26#1 src/test.ml:71:8-240#1 ], 0.010, 0.010
    | 73: Minor, 474, [ set.ml:130:21-28#1 set.ml:133:21-28#7 set.ml:130:21-28#1 src/test.ml:38:20-42#1 src/test.ml:36:24-64#13 src/test.ml:48:21-28#1 src/test.ml:55:12-21#2 src/test.ml:77:12-30#1 src/memthol.ml:96:22-26#1 src/test.ml:71:8-240#1 ], 0.010, 0.010
    | 72: Minor, 629, [ set.ml:133:21-28#6 set.ml:130:21-28#2 src/test.ml:38:20-42#1 src/test.ml:36:24-64#15 src/test.ml:48:21-28#1 src/test.ml:55:12-21#2 src/test.ml:77:12-30#1 src/memthol.ml:96:22-26#1 src/test.ml:71:8-240#1 ], 0.010, 0.010
    | 71: Minor, 477, [ src/test.ml:36:24-64#114 src/test.ml:48:21-28#1 src/test.ml:55:12-21#2 src/test.ml:77:12-30#1 src/memthol.ml:96:22-26#1 src/test.ml:71:8-240#1 ], 0.010, 0.010
    | 70: Minor, 329, [ src/test.ml:36:24-64#26 src/test.ml:48:21-28#1 src/test.ml:55:12-21#2 src/test.ml:77:12-30#1 src/memthol.ml:96:22-26#1 src/test.ml:71:8-240#1 ], 0.009, 0.010
    | 69: Minor, 326, [ set.ml:130:21-28#2 src/test.ml:38:20-42#1 src/test.ml:36:24-64#18 src/test.ml:48:21-28#1 src/test.ml:55:12-21#2 src/test.ml:77:12-30#1 src/memthol.ml:96:22-26#1 src/test.ml:71:8-240#1 ], 0.009, 0.010
    | 68: Minor, 506, [ src/test.ml:36:24-64#68 src/test.ml:48:21-28#1 src/test.ml:55:12-21#2 src/test.ml:77:12-30#1 src/memthol.ml:96:22-26#1 src/test.ml:71:8-240#1 ], 0.009, 0.010
    | 67: Minor, 116, [ set.ml:130:21-28#2 src/test.ml:38:20-42#1 src/test.ml:36:24-64#21 src/test.ml:48:21-28#1 src/test.ml:55:12-21#2 src/test.ml:77:12-30#1 src/memthol.ml:96:22-26#1 src/test.ml:71:8-240#1 ], 0.009, 0.010
    | 66: Minor, 624, [ src/test.ml:36:24-64#56 src/test.ml:48:21-28#1 src/test.ml:55:12-21#2 src/test.ml:77:12-30#1 src/memthol.ml:96:22-26#1 src/test.ml:71:8-240#1 ], 0.009, 0.010
    | 65: Minor, 99, [ src/test.ml:36:24-64#117 src/test.ml:48:21-28#1 src/test.ml:55:12-21#2 src/test.ml:77:12-30#1 src/memthol.ml:96:22-26#1 src/test.ml:71:8-240#1 ], 0.009, 0.010
    | 64: Minor, 285, [ src/test.ml:36:24-64#34 src/test.ml:48:21-28#1 src/test.ml:55:12-21#2 src/test.ml:77:12-30#1 src/memthol.ml:96:22-26#1 src/test.ml:71:8-240#1 ], 0.009, 0.010
    | 63: Minor, 644, [ src/test.ml:36:24-64#46 src/test.ml:48:21-28#1 src/test.ml:55:12-21#2 src/test.ml:77:12-30#1 src/memthol.ml:96:22-26#1 src/test.ml:71:8-240#1 ], 0.009, 0.010
    | 62: Minor, 70, [ src/test.ml:36:24-64#92 src/test.ml:48:21-28#1 src/test.ml:55:12-21#2 src/test.ml:77:12-30#1 src/memthol.ml:96:22-26#1 src/test.ml:71:8-240#1 ], 0.008, 0.010
    | 61: Minor, 119, [ src/test.ml:36:24-64#37 src/test.ml:48:21-28#1 src/test.ml:55:12-21#2 src/test.ml:77:12-30#1 src/memthol.ml:96:22-26#1 src/test.ml:71:8-240#1 ], 0.008, 0.010
    | 60: Minor, 118, [ set.ml:133:21-28#1 set.ml:130:21-28#1 src/test.ml:38:20-42#1 src/test.ml:36:24-64#32 src/test.ml:48:21-28#1 src/test.ml:55:12-21#2 src/test.ml:77:12-30#1 src/memthol.ml:96:22-26#1 src/test.ml:71:8-240#1 ], 0.008, 0.010
    | 59: Minor, 531, [ src/test.ml:36:24-64#40 src/test.ml:48:21-28#1 src/test.ml:55:12-21#2 src/test.ml:77:12-30#1 src/memthol.ml:96:22-26#1 src/test.ml:71:8-240#1 ], 0.008, 0.010
    | 58: Minor, 264, [ src/test.ml:36:24-64#36 src/test.ml:48:21-28#1 src/test.ml:55:12-21#2 src/test.ml:77:12-30#1 src/memthol.ml:96:22-26#1 src/test.ml:71:8-240#1 ], 0.008, 0.010
    | 57: Minor, 242, [ set.ml:112:21-36#1 set.ml:133:21-28#4 set.ml:130:21-28#2 src/test.ml:38:20-42#1 src/test.ml:36:24-64#36 src/test.ml:48:21-28#1 src/test.ml:55:12-21#2 src/test.ml:77:12-30#1 src/memthol.ml:96:22-26#1 src/test.ml:71:8-240#1 ], 0.008, 0.010
    | 56: Minor, 191, [ src/test.ml:36:24-64#81 src/test.ml:48:21-28#1 src/test.ml:55:12-21#2 src/test.ml:77:12-30#1 src/memthol.ml:96:22-26#1 src/test.ml:71:8-240#1 ], 0.008, 0.010
    | 55: Minor, 421, [ src/test.ml:36:24-64#102 src/test.ml:48:21-28#1 src/test.ml:55:12-21#2 src/test.ml:77:12-30#1 src/memthol.ml:96:22-26#1 src/test.ml:71:8-240#1 ], 0.008, 0.008
    | 54: Minor, 475, [ src/test.ml:36:24-64#50 src/test.ml:48:21-28#1 src/test.ml:55:12-21#2 src/test.ml:77:12-30#1 src/memthol.ml:96:22-26#1 src/test.ml:71:8-240#1 ], 0.008, 0.008
    | 53: Minor, 659, [ src/test.ml:36:24-64#52 src/test.ml:48:21-28#1 src/test.ml:55:12-21#2 src/test.ml:77:12-30#1 src/memthol.ml:96:22-26#1 src/test.ml:71:8-240#1 ], 0.007, 0.008
    | 52: Minor, 589, [ set.ml:133:21-28#4 set.ml:130:21-28#1 src/test.ml:36:24-46#1 src/test.ml:36:24-64#40 src/test.ml:48:21-28#1 src/test.ml:55:12-21#2 src/test.ml:77:12-30#1 src/memthol.ml:96:22-26#1 src/test.ml:71:8-240#1 ], 0.007, 0.008
    | 51: Minor, 62, [ src/test.ml:36:24-64#88 src/test.ml:48:21-28#1 src/test.ml:55:12-21#2 src/test.ml:77:12-30#1 src/memthol.ml:96:22-26#1 src/test.ml:71:8-240#1 ], 0.007, 0.008
    | 50: Minor, 476, [ src/test.ml:36:24-64#52 src/test.ml:48:21-28#1 src/test.ml:55:12-21#2 src/test.ml:77:12-30#1 src/memthol.ml:96:22-26#1 src/test.ml:71:8-240#1 ], 0.007, 0.008
    | 49: Minor, 488, [ src/test.ml:36:24-64#64 src/test.ml:48:21-28#1 src/test.ml:55:12-21#2 src/test.ml:77:12-30#1 src/memthol.ml:96:22-26#1 src/test.ml:71:8-240#1 ], 0.007, 0.008
    |
} ;
dead {
    | 48: 0.007
    | 47: 0.007
    | 46: 0.007
    | 45: 0.007
    | 44: 0.007
    | 43: 0.007
    | 42: 0.007
    | 41: 0.007
    | 40: 0.007
    | 39: 0.007
    | 38: 0.007
    | 37: 0.007
    | 36: 0.007
    | 35: 0.007
    | 34: 0.007
    | 33: 0.007
    | 32: 0.007
    | 31: 0.007
    | 30: 0.007
    | 29: 0.007
    | 28: 0.007
    |
}
    "#;

    #[test]
    fn test_2() {
        if let Err(e) = Diff::of_str(DIFF_2) {
            println!("|===| Error:");
            for line in format!("{}", e).lines() {
                println!("| {}", line)
            }
            panic!("test failed")
        }
    }
}

#[cfg(test)]
mod date {
    use super::*;

    fn date_of(sec: u64, nanos: u32) -> Date {
        Duration::new(sec, nanos).into()
    }

    #[test]
    fn display() {
        let duration = date_of(1, 0);
        assert_eq! { &duration.to_string(), "1" }
        let duration = date_of(0, 700_000);
        assert_eq! { &duration.to_string(), "0.0007" }
        let duration = date_of(5, 427_030_000);
        assert_eq! { &duration.to_string(), "5.42703" }
    }
}
