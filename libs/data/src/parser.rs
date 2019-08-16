//! Allocation data parsers.

use std::str::FromStr;

use crate::*;

lazy_static::lazy_static! {
    /// Span separator.
    static ref SPAN_SEP: &'static str = "-";
    /// Span format.
    static ref SPAN_FMT: String = format!("<int>{}<int>", *SPAN_SEP);

    /// Location separator.
    static ref LOC_SEP: &'static str = ":";
    /// Location counter separator.
    static ref LOC_COUNT_SEP: &'static str = "#";
    /// Location format.
    static ref LOC_FMT: String = format!("<file>{}<line>{}{}", *LOC_SEP, *LOC_SEP, *SPAN_FMT);
    /// Location with count format.
    static ref LOC_COUNT_FMT: String = format!("{}{}<int>", *LOC_FMT, *LOC_COUNT_SEP);

    /// None format.
    static ref NONE_FMT: String = format!("<none>");

    /// Trace start.
    static ref TRACE_START: &'static str = "[";
    /// Trace end.
    static ref TRACE_END: &'static str = "]";
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
    static ref ALLOC_SEP: &'static str = ",";
    /// Allocation format.
    static ref ALLOC_FMT: String = format!(
        "<uid>: <kind>{} <size>{} <trace>{} <created_at> $({} <died_at>)?",
        *ALLOC_SEP, *ALLOC_SEP, *ALLOC_SEP, *ALLOC_SEP
    );

    /// Diff separator.
    static ref DIFF_SEP: &'static str = ";";
    /// Diff inner separator.
    static ref DIFF_INNER_SEP: &'static str = "|";
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
    pub fn error<Str: AsRef<str>>(&self, blah: Str) -> String {
        self.error_at(self.cursor, blah)
    }

    /// Minimal error reporting.
    pub fn error_at<Str: AsRef<str>>(&self, pos: usize, blah: Str) -> String {
        let blah = blah.as_ref();

        let mut end_pos = pos;
        let mut chars = self.text[pos..].chars();

        while let Some(char) = chars.next() {
            if char.is_whitespace() {
                break;
            } else {
                end_pos += 1
            }
        }

        let text_bit = if end_pos == pos {
            "<end of input>"
        } else {
            &self.text[pos..end_pos]
        };
        format!("{}, found `{}`", blah, text_bit)
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
    pub fn tag(&mut self, tag: &str) -> Result<(), String> {
        if self.try_tag(tag) {
            Ok(())
        } else {
            Err(self.error(format!("expected `{}`", tag)))
        }
    }

    /// Top-level parser.
    pub fn diff(&mut self) -> Result<Diff, String> {
        let date = self.date()?;
        self.ws();
        let new = self.new_allocs()?;
        self.ws();
        let dead = self.dead()?;

        Ok(Diff::new(date, new, dead))
    }

    /// Parses the new allocation entry.
    pub fn new_allocs(&mut self) -> Result<Vec<Alloc>, String> {
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
    /// let mut parser = Parser::new(r#"523: Major 32 [ "file":7:3-5#11 ] 5.3 _"#);
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
    /// let mut parser = Parser::new(r#"523: Major 32 [ "file":7:3-5#11 ] 5.3 7.3"#);
    /// let alloc = parser.alloc().unwrap();
    /// assert_eq! { parser.rest(), "" }
    /// assert_eq! { alloc.uid.to_string(), "523" }
    /// assert_eq! { alloc.kind, AllocKind::Major }
    /// assert_eq! { alloc.size, 32 }
    /// assert_eq! { *alloc.trace, vec![ (Loc::new("file", 7, (3, 5)), 11) ] }
    /// assert_eq! { alloc.toc.to_string(), "5.3" }
    /// assert_eq! { alloc.tod.unwrap().to_string(), "7.3" }
    /// ```
    pub fn alloc(&mut self) -> Result<Alloc, String> {
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
    fn kind(&mut self) -> Result<AllocKind, String> {
        if self.try_tag("Minor") {
            Ok(AllocKind::Minor)
        } else if self.try_tag("MajorPostponed") {
            Ok(AllocKind::MajorPostponed)
        } else if self.try_tag("Major") {
            Ok(AllocKind::Major)
        } else if self.try_tag("Serialized") {
            Ok(AllocKind::Serialized)
        } else {
            Err(self.error(format!("expected allocation kind `{}`", *ALLOC_KIND_FMT)))
        }
    }

    /// Parses a uid.
    fn uid(&mut self) -> Result<Uid, String> {
        let start_pos = self.cursor;
        if let Ok(int) = self.int() {
            if let Some(uid) = BigUint::parse_bytes(int.as_bytes(), 10) {
                return Ok(uid.into());
            }
        }
        Err(self.error_at(start_pos, "expected UID (big int)"))
    }

    /// Parses a usize.
    ///
    /// ```rust
    /// use alloc_data::Parser;
    /// let mut parser = Parser::new("772032 blah");
    /// assert_eq! { parser.usize().unwrap(), 772032 }
    /// assert_eq! { parser.rest(), " blah" }
    /// ```
    pub fn usize(&mut self) -> Result<usize, String> {
        let size_pos = self.cursor;
        if let Ok(int) = self.int() {
            if let Ok(usize) = usize::from_str(int) {
                return Ok(usize);
            }
        }
        Err(self.error_at(size_pos, "expected integer (usize)"))
    }

    /// Parses an integer.
    ///
    /// ```rust
    /// use alloc_data::Parser;
    /// let mut parser = Parser::new("772032 blah");
    /// assert_eq! { parser.int().unwrap(), "772032" }
    /// assert_eq! { parser.rest(), " blah" }
    /// ```
    pub fn int(&mut self) -> Result<&'a str, String> {
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
            Err(self.error("expected integer"))
        } else {
            let res = &self.text[self.cursor..end];
            self.cursor = end;
            Ok(res)
        }
    }

    /// Parses an optional date.
    ///
    /// ```rust
    /// use alloc_data::Parser;
    /// let mut parser = Parser::new("_ 520.530 tail");
    /// assert_eq! { parser.date_opt(), Ok(None) }
    /// assert_eq! { parser.rest(), " 520.530 tail" }
    /// parser.ws();
    /// assert_eq! {
    ///     parser.date_opt().unwrap().unwrap(),
    ///     std::time::Duration::new(520, 530_000_000).into()
    /// }
    /// assert_eq! { parser.rest(), " tail" }
    /// ```
    pub fn date_opt(&mut self) -> Result<Option<Date>, String> {
        if self.try_tag("_") {
            Ok(None)
        } else if let Ok(date) = self.date() {
            Ok(Some(date))
        } else {
            Err(self.error(format!("expected date `{}`", *DATE_FMT)))
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
    pub fn date(&mut self) -> Result<Date, String> {
        let start_pos = self.cursor;
        let num = self.int()?;
        match self.chars().next() {
            Some('.') => self.cursor += 1,
            _ => return Err(self.error_at(start_pos, "expected date (1)")),
        }
        let dec = self.int()?;
        let dec = format!("{:0<9}", dec);

        println!("dec: {}", dec);

        let num = if let Ok(num) = u64::from_str(num) {
            num
        } else {
            return Err(self.error_at(start_pos, "expected date (2)"));
        };
        let dec = if let Ok(dec) = u32::from_str(&dec) {
            dec
        } else {
            return Err(self.error_at(start_pos, "expected date (3)"));
        };

        Ok(Duration::new(num, dec).into())
    }

    /// Parses a trace of location/count pairs.
    pub fn trace(&mut self) -> Result<Trace, String> {
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

    /// Parses an optional location/count pair.
    ///
    /// ```rust
    /// use alloc_data::{Parser, Loc};
    /// let mut parser = Parser::new(r#""my_file":7:3-5#13"#);
    /// parser.ws();
    /// assert_eq! {
    ///     parser.loc_count().unwrap(),
    ///     (Loc::new("my_file", 7, (3, 5)), 13)
    /// }
    /// assert_eq! { parser.rest(), "" }
    /// ```
    pub fn loc_count(&mut self) -> Result<(Loc, usize), String> {
        self.tag("\"")?;
        let mut end = self.cursor;
        for char in self.chars() {
            if char == '"' {
                break;
            } else {
                end += 1
            }
        }
        let file = self.text[self.cursor..end].to_string();
        self.cursor = end;
        self.tag("\"")?;

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

        self.ws();
        self.tag("#")?;

        self.ws();
        let count = self.usize()?;

        let loc = Loc::new(file, line, (start, end));

        Ok((loc, count))
    }

    pub fn dead(&mut self) -> Result<Vec<(Uid, Date)>, String> {
        self.tag("[")?;
        self.ws();
        let mut vec = Vec::with_capacity(17);

        while !self.is_eoi() && !self.try_tag("]") {
            let uid = self.uid()?;
            self.ws();
            self.tag(":")?;
            self.ws();
            let date = self.date()?;
            vec.push((uid, date));
            self.ws()
        }
        vec.shrink_to_fit();
        Ok(vec)
    }
}
