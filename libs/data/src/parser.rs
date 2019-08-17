//! Allocation data parsers.

use std::str::FromStr;

use crate::{err::ResultExt, *};

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
static ALLOC_KIND_FMT: &str = "\\( Minor | Major | MajorPostponed | Serialized \\)";
/// Dead allocation format.
static DEAD_ALLOC_FMT: &str = "<uid>: <died_at: date>";
/// Diff format.
static DIFF_FMT: &str = "<timestamp> `new` {{ \
                         <alloc> ... \
                         }} dead {{ \
                         <dead alloc> ... \
                         }}\
                         ";

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
    /// Label format.
    static ref LABEL: String = "`<anything but `>`".into();
    /// Labels format.
    static ref LABELS_FMT: String = format!(
        "{} {} ... {}",
        TRACE_START, *LABEL, TRACE_END
    );

    /// Date format.
    static ref DATE_FMT: String = "<int>.<int>".into();

    /// Allocation format.
    static ref ALLOC_FMT: String = format!(
        "<uid>: <kind> <size> <trace> <created_at: date> \\( <died_at: date> | {} \\)",
        NONE_FMT
    );
}

/// Zero-copy parser.
pub struct Parser<'a> {
    /// Text we're parsing.
    text: &'a str,
    /// Position in the text.
    cursor: usize,
}

impl<'a> Parser<'a> {
    /// Constructor.
    pub fn new(text: &'a str) -> Self {
        Self { text, cursor: 0 }
    }

    /// True if there is no more text to parse.
    ///
    /// ```rust
    /// use alloc_data::Parser;
    /// let parser = Parser::new("");
    /// assert! { parser.is_eoi() }
    /// ```
    pub fn is_eoi(&self) -> bool {
        self.cursor >= self.text.len()
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

    /// Returns the text from the cursor to the end.
    ///
    /// ```rust
    /// use alloc_data::Parser;
    /// let mut parser = Parser::new(" \tblah");
    /// assert_eq! { parser.rest(), " \tblah" }
    /// parser.ws();
    /// assert_eq! { parser.rest(), "blah" }
    /// ```
    pub fn rest(&self) -> &'a str {
        &self.text[self.cursor..]
    }

    /// Iterator over characters from the current position.
    ///
    /// ```rust
    /// use alloc_data::Parser;
    /// let mut parser = Parser::new(" \t blah");
    /// let mut chars = parser.chars();
    /// assert_eq! { chars.next(), Some(' ') }
    /// assert_eq! { chars.next(), Some('\t') }
    /// assert_eq! { chars.next(), Some(' ') }
    /// assert_eq! { chars.next(), Some('b') }
    /// assert_eq! { chars.next(), Some('l') }
    /// assert_eq! { chars.next(), Some('a') }
    /// assert_eq! { chars.next(), Some('h') }
    /// parser.ws();
    /// let mut chars = parser.chars();
    /// assert_eq! { chars.next(), Some('b') }
    /// assert_eq! { chars.next(), Some('l') }
    /// assert_eq! { chars.next(), Some('a') }
    /// assert_eq! { chars.next(), Some('h') }
    /// ```
    pub fn chars(&self) -> std::str::Chars<'a> {
        self.rest().chars()
    }

    /// Parses whispaces.
    ///
    /// ```rust
    /// use alloc_data::Parser;
    /// let mut parser = Parser::new(" \n\t     blah");
    /// parser.ws();
    /// assert_eq! { parser.rest(), "blah" }
    /// ```
    pub fn ws(&mut self) {
        let mut chars = self.chars();
        while let Some(char) = chars.next() {
            if char.is_whitespace() {
                self.cursor += 1
            } else {
                break;
            }
        }
    }

    /// Minimal error reporting at the current position.
    pub fn error<S: Into<String>>(&self, blah: S) -> err::Err {
        self.error_at(self.cursor, blah)
    }

    /// Minimal error reporting.
    pub fn error_at<S: Into<String>>(&self, pos: usize, blah: S) -> err::Err {
        let position = self.position_details(pos);
        err::ErrorKind::ParseErr(position, blah.into()).into()
    }

    /// Tries to parse a tag.
    ///
    /// ```rust
    /// use alloc_data::Parser;
    /// let mut parser = Parser::new("blah42");
    /// assert! { !parser.try_tag("doesn't match") }
    /// assert_eq! { parser.rest(), "blah42" }
    /// assert! { parser.try_tag("blah") }
    /// assert_eq! { parser.rest(), "42" }
    /// ```
    pub fn try_tag(&mut self, tag: &str) -> bool {
        let end = self.cursor + tag.len();
        if end <= self.text.len() {
            if &self.text[self.cursor..end] == tag {
                self.cursor += tag.len();
                return true;
            }
        }
        false
    }

    /// Parses a tag.
    ///
    /// ```rust
    /// use alloc_data::Parser;
    /// let mut parser = Parser::new("blah42_");
    /// assert! { parser.tag("doesn't match").is_err() }
    /// assert_eq! { parser.rest(), "blah42_" }
    /// assert! { parser.tag("blah").is_ok() }
    /// assert_eq! { parser.rest(), "42_" }
    /// assert! { parser.tag("42").is_ok() }
    /// assert! { parser.tag("_").is_ok() }
    /// assert_eq! { parser.rest(), "" }
    /// ```
    pub fn tag(&mut self, tag: &str) -> Res<()> {
        if self.try_tag(tag) {
            Ok(())
        } else {
            bail!(self.error(format!("expected `{}`", tag)))
        }
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
    /// use alloc_data::{AllocKind, Loc, Parser};
    /// let mut parser = Parser::new(r#"523: Major 32 [ `file`:7:3-5#11 ] 5.3 _"#);
    /// let alloc = parser.alloc().unwrap();
    /// assert_eq! { parser.rest(), "" }
    /// assert_eq! { alloc.uid.to_string(), "523" }
    /// assert_eq! { alloc.kind, AllocKind::Major }
    /// assert_eq! { alloc.size, 32 }
    /// assert_eq! { *alloc.trace, vec![ (Loc::new("file", 7, (3, 5)), 11) ] }
    /// assert_eq! { alloc.toc.to_string(), "5.3" }
    /// assert! { alloc.tod.is_none() }
    /// ```
    ///
    /// ```rust
    /// use alloc_data::{AllocKind, Loc, Parser};
    /// let mut parser = Parser::new(r#"523: Major 32 [ `file`:7:3-5#11 ] 5.3 7.3"#);
    /// let alloc = parser.alloc().unwrap();
    /// assert_eq! { parser.rest(), "" }
    /// assert_eq! { alloc.uid.to_string(), "523" }
    /// assert_eq! { alloc.kind, AllocKind::Major }
    /// assert_eq! { alloc.size, 32 }
    /// assert_eq! { *alloc.trace, vec![ (Loc::new("file", 7, (3, 5)), 11) ] }
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

        // self.ws();
        // let labels = self.labels()?;

        self.ws();
        let toc = self.date()?;

        self.ws();
        let tod = self.date_opt()?;

        Ok(Alloc::new(uid, kind, size, trace, toc, tod))
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
        } else {
            let msg = format!("expected allocation kind `{}`", ALLOC_KIND_FMT);
            bail!(self.error(msg))
        }
    }

    /// Parses a uid.
    pub fn uid(&mut self) -> Res<Uid> {
        let start_pos = self.cursor;
        if let Ok(int) = self.int() {
            if let Some(uid) = BigUint::parse_bytes(int.as_bytes(), 10) {
                return Ok(uid.into());
            }
        }
        bail!(self.error_at(start_pos, "expected UID (big int)"))
    }

    /// Parses a usize.
    ///
    /// ```rust
    /// use alloc_data::Parser;
    /// let mut parser = Parser::new("772032 blah");
    /// assert_eq! { parser.usize().unwrap(), 772032 }
    /// assert_eq! { parser.rest(), " blah" }
    /// ```
    pub fn usize(&mut self) -> Res<usize> {
        let size_pos = self.cursor;
        if let Ok(int) = self.int() {
            if let Ok(usize) = usize::from_str(int) {
                return Ok(usize);
            }
        }
        bail!(self.error_at(size_pos, "expected integer (usize)"))
    }

    /// Parses an integer.
    ///
    /// ```rust
    /// use alloc_data::Parser;
    /// let mut parser = Parser::new("772032 blah");
    /// assert_eq! { parser.int().unwrap(), "772032" }
    /// assert_eq! { parser.rest(), " blah" }
    /// ```
    pub fn int(&mut self) -> Res<&'a str> {
        let mut chars = self.chars();
        let mut end = self.cursor;

        while let Some(char) = chars.next() {
            if char.is_numeric() {
                end += 1
            } else {
                break;
            }
        }

        if end == self.cursor {
            bail!(self.error("expected integer"))
        } else {
            let res = &self.text[self.cursor..end];
            self.cursor = end;
            Ok(res)
        }
    }

    /// Parses an optional date.
    ///
    /// ```rust
    /// use alloc_data::{Parser};
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
    pub fn date_opt(&mut self) -> Res<Option<Date>> {
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
    /// use alloc_data::Parser;
    /// let mut parser = Parser::new("772032.52 blah");
    /// assert_eq! { parser.date().unwrap(), std::time::Duration::new(772032, 520_000_000).into() }
    /// assert_eq! { parser.rest(), " blah" }
    /// ```
    pub fn date(&mut self) -> Res<Date> {
        let err_msg = || format!("expected date `{}`", *DATE_FMT);
        let start_pos = self.cursor;
        let num = self.int()?;
        match self.chars().next() {
            Some('.') => self.cursor += 1,
            _ => bail!(self.error_at(start_pos, err_msg())),
        }
        let dec = self.int()?;
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
        let mut end = self.cursor;
        for char in self.chars() {
            if char == '`' {
                break;
            } else {
                end += 1
            }
        }
        let file = self.text[self.cursor..end].to_string();
        self.cursor = end;
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
    /// use alloc_data::{Parser, Loc};
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

    /// Parses a dead allocation info.
    fn dead_alloc(&mut self) -> Res<(Uid, Date)> {
        self.inner_dead_alloc()
            .chain_err(|| format!("while parsing dead allocation `{}`", DEAD_ALLOC_FMT))
    }
    fn inner_dead_alloc(&mut self) -> Res<(Uid, Date)> {
        let uid = self.uid()?;
        self.ws();
        self.tag(":")?;
        self.ws();
        let date = self.date()?;
        Ok((uid, date))
    }

    /// Parses dead allocation info.
    pub fn dead(&mut self) -> Res<Vec<(Uid, Date)>> {
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
}

impl<'a> Parser<'a> {
    /// Retrieves the row, column, and line of a position.
    pub fn position_details(&self, pos: usize) -> err::Position {
        let (mut row, mut count, mut col_line) = (0, 0, None);

        for line in self.text.lines() {
            debug_assert! { pos >= count }

            // Is the position in the current line, or at the end of the current line?
            if pos <= count + line.len() + 1 {
                col_line = Some((pos - count, line.to_string()));
                break;
            } else {
                // Position is not in the current line, move on.
                row += 1;
                count += line.len() + 1
            }
        }

        if let Some((col, line)) = col_line {
            err::Position::new(row, col, line)
        } else {
            panic!("could not find position `{}` in input text, while retrieving position details")
        }
    }
}
