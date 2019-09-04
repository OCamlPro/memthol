//! Allocation data parsers.

use std::str::FromStr;

use swarkn::parse;
use swarkn::parse::err::*;
pub use swarkn::parse::{ParserErrorExt, ParserExt};

pub mod err {
    pub use swarkn::parse::err::*;
}

use crate::*;

/// Span separator.
static SPAN_SEP: &str = "-";
/// Location separator.
static LOC_SEP: &str = ":";
/// Location counter separator.
static LOC_COUNT_SEP: &str = "#";
/// None format.
static NONE_FMT: &str = "_";
/// Trace start.
static TRACE_START: &str = "[";
/// Trace end.
static TRACE_END: &str = "]";
/// Allocation kind format.
static ALLOC_KIND_FMT: &str = "\\( Minor | Major | MajorPostponed | Serialized | _ \\)";
/// Dead allocation format.
static DEAD_ALLOC_FMT: &str = "<uid>: <died_at: date>";
/// Diff format.
static DIFF_FMT: &str = "<timestamp> `new` {{ \
                         <alloc> ... \
                         }} dead {{ \
                         <dead alloc> ... \
                         }}\
                         ";
/// Label format.
static LABEL_FMT: &str = "`<anything but `>`";

lazy_static::lazy_static! {
    /// Span format.
    static ref SPAN_FMT: String = format!("<int>{}<int>", SPAN_SEP);

    /// Location format.
    static ref LOC_FMT: String = format!("<file>{}<line>{}{}", LOC_SEP, LOC_SEP, *SPAN_FMT);
    /// Location with count format.
    static ref LOC_COUNT_FMT: String = format!("{}{}<int>", *LOC_FMT, LOC_COUNT_SEP);

    /// Trace format.
    static ref TRACE_FMT: String = format!(
        "{} \\({} | {}\\) ... {}",
        TRACE_START, *LOC_COUNT_FMT, NONE_FMT, TRACE_END
    );
    /// Labels format.
    static ref LABELS_FMT: String = format!(
        "{} {} ... {}",
        TRACE_START, LABEL_FMT, TRACE_END
    );

    /// Date format.
    static ref DATE_FMT: String = "<int>.<int>".into();

    /// Allocation format.
    static ref ALLOC_FMT: String = format!(
        "<uid>: <kind> <size> <trace> <labels> <created_at: date> \\( <died_at: date> | {} \\)",
        NONE_FMT
    );
}

/// Zero-copy parser.
pub struct Parser<'txt> {
    /// Underlying parser.
    parser: parse::Parser<'txt>,
}
impl<'txt> ParserExt<'txt> for Parser<'txt> {
    fn parser(&self) -> &parse::Parser<'txt> {
        &self.parser
    }
    fn parser_mut(&mut self) -> &mut parse::Parser<'txt> {
        &mut self.parser
    }
}

impl<'txt> Parser<'txt> {
    /// Constructor.
    pub fn new(text: &'txt str) -> Self {
        Self {
            parser: parse::Parser::new(text),
        }
    }

    /// Applies a parser and fails if there are tokens left.
    pub fn parse_all<'str, T, Parse, Str>(txt: &'str str, parse: Parse, desc: Str) -> Res<T>
    where
        Parse: Fn(&'_ mut Parser<'str>) -> Res<T>,
        Str: AsRef<str>,
    {
        let mut parser = Parser::new(txt);
        parser.ws();
        let res = parse(&mut parser)?;
        parser.ws();
        if !parser.is_eoi() {
            bail!(parser.error(format!("unexpected tokens after {}", desc.as_ref())))
        }
        Ok(res)
    }

    /// Top-level parser.
    pub fn diff(&mut self) -> Res<Diff> {
        self.inner_diff()
            .chain_err(|| format!("while parsing diff `{}`", DIFF_FMT))
    }
    fn inner_diff(&mut self) -> Res<Diff> {
        self.ws();
        let date = self.date().chain_err(|| "while parsing diff's timestamp")?;
        self.ws();
        let new = self
            .new_allocs()
            .chain_err(|| "while parsing diff's new allocations")?;
        self.ws();
        let dead = self
            .dead()
            .chain_err(|| "while parsing diff's dead allocations")?;

        Ok(Diff::new(date, new, dead))
    }

    /// Parses the new allocation entry.
    pub fn new_allocs(&mut self) -> Res<Vec<Alloc>> {
        self.tag("new")?;
        self.ws();
        self.tag("{")?;
        self.ws();

        let mut vec = Vec::with_capacity(17);

        while !self.try_tag("}") {
            self.ws();
            vec.push(self.alloc()?);
            self.ws()
        }

        vec.shrink_to_fit();

        Ok(vec)
    }

    /// Parses a single allocation.
    ///
    /// ```rust
    /// use alloc_data::{AllocKind, Loc, parser::*};
    /// let mut parser = Parser::new(r#"523: Major 32 [ `file`:7:3-5#11 ] [ `label` ] 5.3 _"#);
    /// let alloc = parser.alloc().unwrap();
    /// assert_eq! { parser.rest(), "" }
    /// assert_eq! { alloc.uid.to_string(), "523" }
    /// assert_eq! { alloc.kind, AllocKind::Major }
    /// assert_eq! { alloc.size, 32 }
    /// assert_eq! { *alloc.trace, vec![ (Loc::new("file", 7, (3, 5)), 11) ] }
    /// assert_eq! { alloc.labels, vec![ "label".to_string() ] }
    /// assert_eq! { alloc.toc.to_string(), "5.3" }
    /// assert! { alloc.tod.is_none() }
    /// ```
    ///
    /// ```rust
    /// use alloc_data::{AllocKind, Loc, parser::*};
    /// let mut parser = Parser::new(
    ///     r#"523: Major 32 [ `file`:7:3-5#11 ] [`label_1` `label_2`] 5.3 7.3"#
    /// );
    /// let alloc = parser.alloc().unwrap();
    /// assert_eq! { parser.rest(), "" }
    /// assert_eq! { alloc.uid.to_string(), "523" }
    /// assert_eq! { alloc.kind, AllocKind::Major }
    /// assert_eq! { alloc.size, 32 }
    /// assert_eq! { *alloc.trace, vec![ (Loc::new("file", 7, (3, 5)), 11) ] }
    /// assert_eq! { alloc.labels, vec![ "label_1".to_string(), "label_2".to_string() ] }
    /// assert_eq! { alloc.toc.to_string(), "5.3" }
    /// assert_eq! { alloc.tod.unwrap().to_string(), "7.3" }
    /// ```
    pub fn alloc(&mut self) -> Res<Alloc> {
        let uid = self.uid()?;

        self.ws();
        self.tag(":")?;

        self.ws();
        let kind = self.kind()?;

        self.ws();
        let size = self.usize()?;

        self.ws();
        let trace = self.trace()?;

        self.ws();
        let labels = self.labels()?;

        self.ws();
        let toc = self.date()?;

        self.ws();
        let tod = self.date_opt()?;

        Ok(Alloc::new(uid, kind, size, trace, labels, toc, tod))
    }

    /// Parses an allocation kind.
    pub fn kind(&mut self) -> Res<AllocKind> {
        if self.try_tag("Minor") {
            Ok(AllocKind::Minor)
        } else if self.try_tag("MajorPostponed") {
            Ok(AllocKind::MajorPostponed)
        } else if self.try_tag("Major") {
            Ok(AllocKind::Major)
        } else if self.try_tag("Serialized") {
            Ok(AllocKind::Serialized)
        } else if self.try_tag("_") {
            Ok(AllocKind::Unknown)
        } else {
            let msg = format!("expected allocation kind `{}`", ALLOC_KIND_FMT);
            bail!(self.error(msg))
        }
    }

    /// Parses a uid.
    pub fn uid(&mut self) -> Res<Uid> {
        let start_pos = self.pos();
        if let Ok(int) = self.digits() {
            if let Some(uid) = BigUint::parse_bytes(int.as_bytes(), 10) {
                return Ok(uid.into());
            }
        }
        bail!(self.error_at(start_pos, "expected UID (big int)"))
    }

    /// Parses an optional date.
    ///
    /// ```rust
    /// use alloc_data::parser::*;
    /// let mut parser = Parser::new("_ 520.530 tail");
    /// assert_eq! { parser.date_opt().unwrap(), None }
    /// assert_eq! { parser.rest(), " 520.530 tail" }
    /// parser.ws();
    /// assert_eq! {
    ///     parser.date_opt().unwrap().unwrap(),
    ///     std::time::Duration::new(520, 530_000_000).into()
    /// }
    /// assert_eq! { parser.rest(), " tail" }
    /// ```
    pub fn date_opt(&mut self) -> Res<Option<SinceStart>> {
        if self.try_tag("_") {
            Ok(None)
        } else if let Ok(date) = self.date() {
            Ok(Some(date))
        } else {
            bail!(self.error(format!("expected date `{}`", *DATE_FMT)))
        }
    }

    /// Parses a date in seconds.
    ///
    /// ```rust
    /// use alloc_data::parser::*;
    /// let mut parser = Parser::new("772032.52 blah");
    /// assert_eq! { parser.date().unwrap(), std::time::Duration::new(772032, 520_000_000).into() }
    /// assert_eq! { parser.rest(), " blah" }
    /// ```
    pub fn date(&mut self) -> Res<SinceStart> {
        let err_msg = || format!("expected date `{}`", *DATE_FMT);
        let start_pos = self.pos();
        let num = self.digits()?;
        self.tag(".")
            .chain_err(|| self.error_at(start_pos, err_msg()))?;
        let dec = self.digits()?;
        let dec = format!("{:0<9}", dec);

        let num = if let Ok(num) = u64::from_str(num) {
            num
        } else {
            bail!(self.error_at(start_pos, err_msg()));
        };
        let dec = if let Ok(dec) = u32::from_str(&dec) {
            dec
        } else {
            bail!(self.error_at(start_pos, err_msg()));
        };

        Ok(Duration::new(num, dec).into())
    }

    /// Parses a trace of location/count pairs.
    pub fn trace(&mut self) -> Res<Trace> {
        self.tag("[")?;
        self.ws();
        let mut vec = Vec::with_capacity(17);
        while !self.is_eoi() && !self.try_tag("]") {
            let loc_count = self.loc_count()?;
            vec.push(loc_count);
            self.ws()
        }
        vec.shrink_to_fit();
        Ok(Trace::new(vec))
    }

    pub fn loc(&mut self) -> Res<Loc> {
        self.inner_loc()
            .chain_err(|| format!("while parsing location `{}`", *LOC_FMT))
    }
    fn inner_loc(&mut self) -> Res<Loc> {
        self.tag("`")?;
        let file = self.chars_until(|char| char == '`').to_string();
        self.tag("`")?;

        self.ws();
        self.tag(":")?;

        self.ws();
        let line = self.usize()?;

        self.ws();
        self.tag(":")?;

        self.ws();
        let start = self.usize()?;

        self.ws();
        self.tag("-")?;

        self.ws();
        let end = self.usize()?;

        let loc = Loc::new(file, line, (start, end));

        Ok(loc)
    }

    /// Parses an optional location/count pair.
    ///
    /// ```rust
    /// use alloc_data::{Loc, parser::*};
    /// let mut parser = Parser::new(r#"`my_file`:7:3-5#13"#);
    /// parser.ws();
    /// assert_eq! {
    ///     parser.loc_count().unwrap(),
    ///     (Loc::new("my_file", 7, (3, 5)), 13)
    /// }
    /// assert_eq! { parser.rest(), "" }
    /// ```
    pub fn loc_count(&mut self) -> Res<(Loc, usize)> {
        self.inner_loc_count()
            .chain_err(|| format!("while parsing location/count `{}`", *LOC_COUNT_FMT))
    }
    fn inner_loc_count(&mut self) -> Res<(Loc, usize)> {
        let loc = self.loc()?;

        self.ws();
        self.tag("#")
            .chain_err(|| "separating location from its count")?;

        self.ws();
        let count = self.usize()?;

        Ok((loc, count))
    }

    /// Parses some labels.
    pub fn labels(&mut self) -> Res<Vec<String>> {
        self.inner_labels()
            .chain_err(|| format!("while parsing labels `{}`", *LABELS_FMT))
    }
    fn inner_labels(&mut self) -> Res<Vec<String>> {
        self.tag("[")?;
        self.ws();

        let mut vec = vec![];

        while !self.try_tag("]") {
            self.ws();
            let label = self.label()?;
            vec.push(label);
            self.ws()
        }

        Ok(vec)
    }

    /// Parses a label.
    pub fn label(&mut self) -> Res<String> {
        self.tag("`").chain_err(|| "starting label")?;
        let label = self.chars_until(|char| char == '`').to_string();
        self.tag("`").chain_err(|| "ending label")?;
        Ok(label)
    }

    /// Parses a dead allocation info.
    fn dead_alloc(&mut self) -> Res<(Uid, SinceStart)> {
        self.inner_dead_alloc()
            .chain_err(|| format!("while parsing dead allocation `{}`", DEAD_ALLOC_FMT))
    }
    fn inner_dead_alloc(&mut self) -> Res<(Uid, SinceStart)> {
        let uid = self.uid()?;
        self.ws();
        self.tag(":")?;
        self.ws();
        let date = self.date()?;
        Ok((uid, date))
    }

    /// Parses dead allocation info.
    pub fn dead(&mut self) -> Res<Vec<(Uid, SinceStart)>> {
        self.tag("dead")?;
        self.ws();
        self.tag("{")?;
        self.ws();
        let mut vec = Vec::with_capacity(17);

        while !self.is_eoi() && !self.try_tag("}") {
            let dead = self.dead_alloc()?;
            vec.push(dead);
            self.ws()
        }

        if self.is_eoi() {
            bail!(self.error("reached end of input while parsing dead information"))
        }

        vec.shrink_to_fit();
        Ok(vec)
    }

    /// Parses memthol's init file.
    pub fn init(&mut self) -> Res<Init> {
        self.tag("start").chain_err(|| "starting the init file")?;
        self.ws();
        self.tag(":")?;
        self.ws();
        let err_msg = || format!("expected date `{}`", *DATE_FMT);
        let start_pos = self.pos();
        let num = self.digits()?;
        self.tag(".").chain_err(err_msg)?;
        let dec = self.digits()?;
        let dec = format!("{:0<9}", dec);

        let secs = if let Ok(num) = i64::from_str(num) {
            num
        } else {
            bail!(self.error_at(start_pos, err_msg()));
        };
        let nanos = if let Ok(dec) = u32::from_str(&dec) {
            dec
        } else {
            bail!(self.error_at(start_pos, err_msg()));
        };
        let start_time = Date::of_timestamp(secs, nanos);

        self.ws();
        self.tag("word_size")?;
        self.ws();
        self.tag(":")?;
        self.ws();
        let word_size = self
            .usize()
            .chain_err(|| "specifying the size of machine words (in bytes)")?;

        Ok(Init::new(start_time, word_size))
    }
}
