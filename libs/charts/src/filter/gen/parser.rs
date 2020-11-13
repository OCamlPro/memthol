/*<LICENSE>
    This file is part of Memthol.

    Copyright (C) 2020 OCamlPro.

    Memthol is free software: you can redistribute it and/or modify
    it under the terms of the GNU General Public License as published by
    the Free Software Foundation, either version 3 of the License, or
    (at your option) any later version.

    Memthol is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
    GNU General Public License for more details.

    You should have received a copy of the GNU General Public License
    along with Memthol.  If not, see <https://www.gnu.org/licenses/>.
*/

//! Parser for filter generator arguments.

prelude! {}

/// Filter generation argument parser.
pub struct Parser<'input> {
    /// Input text.
    txt: &'input str,
    /// Current position.
    pos: usize,
}

impl<'input> Parser<'input> {
    /// Constructor.
    pub fn new(txt: &'input str) -> Self {
        Self {
            txt: txt.trim(),
            pos: 0,
        }
    }

    /// Creates a sub-parser for the remaining text, if any.
    pub fn sub(&self) -> Option<Self> {
        let txt_left = self.txt[self.pos..].trim();
        if txt_left.is_empty() {
            None
        } else {
            Some(Self::new(txt_left))
        }
    }

    /// True if the parser is at end-of-input.
    pub fn is_at_eoi(&self) -> bool {
        self.pos >= self.txt.len()
    }

    /// Remaining text to parse.
    pub fn rest(&self) -> &'input str {
        &self.txt[self.pos..]
    }
}

/// Convenience macro producing a `char` iterator over the remaining text in a parser.
macro_rules! chars {
    ($slf:ident) => {
        $slf.txt[$slf.pos..].chars()
    };
}

impl<'input> Parser<'input> {
    /// Advances the parser by `c.len_utf8()`.
    #[inline]
    pub fn inc(&mut self, c: char) {
        self.pos += c.len_utf8()
    }

    /// Consumes leading whitespaces.
    pub fn ws(&mut self) {
        for c in chars!(self) {
            if c.is_whitespace() {
                self.inc(c)
            } else {
                break;
            }
        }
    }

    /// Parses an integer as a slice over the input text.
    pub fn int(&mut self) -> Option<&'input str> {
        let mut chars = chars!(self);
        let first_char = chars.next()?;

        if !first_char.is_numeric() {
            return None;
        }

        if first_char == '0' && chars.next().map(char::is_numeric).unwrap_or(false) {
            return None;
        }

        let start = self.pos;
        self.inc(first_char);

        for c in chars {
            if c.is_numeric() {
                self.inc(c)
            } else {
                break;
            }
        }

        Some(&self.txt[start..self.pos])
    }

    /// Parses a `usize`.
    pub fn usize(&mut self) -> Option<usize> {
        if let Some(int) = self.int() {
            let res = usize::from_str(int).ok()?;
            Some(res)
        } else {
            None
        }
    }

    /// Parses a `bool`.
    pub fn bool(&mut self) -> Option<bool> {
        if self.id_tag("true") {
            Some(true)
        } else if self.id_tag("false") {
            Some(false)
        } else {
            None
        }
    }

    /// Parses an identifier.
    pub fn ident(&mut self) -> Option<&'input str> {
        let mut chars = chars!(self);
        let first_char = chars.next()?;

        if !first_char.is_alphabetic() {
            return None;
        }

        let start = self.pos;
        self.inc(first_char);

        while let Some(c) = chars.next() {
            if !(c.is_alphanumeric() || c == '_') {
                break;
            } else {
                self.inc(first_char);
            }
        }

        Some(&self.txt[start..self.pos])
    }

    /// Parses a tag, *i.e.* a specific string.
    pub fn tag(&mut self, tag: impl AsRef<str>) -> bool {
        let tag = tag.as_ref();
        if self.pos + tag.len() > self.txt.len() {
            false
        } else {
            if tag == &self.txt[self.pos..self.pos + tag.len()] {
                self.pos += tag.len();
                true
            } else {
                false
            }
        }
    }

    /// Parses an identifier not followed by an alphanumeric character.
    pub fn id_tag(&mut self, tag: impl AsRef<str>) -> bool {
        let start = self.pos;
        if self.tag(tag) {
            match self.rest().chars().next() {
                Some(c) if c.is_alphanumeric() => {
                    self.pos = start;
                    return false;
                }
                _ => true,
            }
        } else {
            false
        }
    }

    /// Parses a specific character.
    pub fn char(&mut self, c: char) -> bool {
        if chars!(self).next() == Some(c) {
            self.pos += c.len_utf8();
            true
        } else {
            false
        }
    }

    /// Extracts the content of a block `{ ... }` and generate a sub-parser.
    pub fn block(&mut self) -> Res<Option<Self>> {
        if !self.char('{') {
            return Ok(None);
        }
        let mut count = 1;
        let start = self.pos;
        let mut end = self.pos;

        for c in chars!(self) {
            if c == '}' {
                count -= 1;
            } else if c == '{' {
                count += 1;
            }

            if count == 0 {
                end = self.pos;
                self.inc(c);
                break;
            } else {
                self.inc(c);
            }
        }

        if count > 0 {
            bail!("ill-formed block, some braces are unmatched")
        }

        Ok(Some(Self::new(&self.txt[start..end])))
    }
}
