//! Parser for filter generator arguments.

prelude! {}

pub struct Parser<'input> {
    txt: &'input str,
    pos: usize,
}
impl<'input> Parser<'input> {
    pub fn new(txt: &'input str) -> Self {
        Self {
            txt: txt.trim(),
            pos: 0,
        }
    }

    pub fn sub(&self) -> Option<Self> {
        let txt_left = self.txt[self.pos..].trim();
        if txt_left.is_empty() {
            None
        } else {
            Some(Self::new(txt_left))
        }
    }

    pub fn is_at_eoi(&self) -> bool {
        self.pos >= self.txt.len()
    }

    pub fn rest(&self) -> &'input str {
        &self.txt[self.pos..]
    }
}

macro_rules! chars {
    ($slf:ident) => {
        $slf.txt[$slf.pos..].chars()
    };
}

impl<'input> Parser<'input> {
    #[inline]
    pub fn inc(&mut self, c: char) {
        self.pos += c.len_utf8()
    }

    pub fn ws(&mut self) {
        for c in chars!(self) {
            if c.is_whitespace() {
                self.inc(c)
            } else {
                break;
            }
        }
    }

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

    pub fn usize(&mut self) -> Option<usize> {
        if let Some(int) = self.int() {
            let res = usize::from_str(int).ok()?;
            Some(res)
        } else {
            None
        }
    }

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

    pub fn tag(&mut self, tag: impl AsRef<str>) -> bool {
        let tag = tag.as_ref();
        if self.pos + tag.len() > self.txt.len() {
            false
        } else {
            if tag == &self.txt[self.pos..tag.len()] {
                self.pos += tag.len();
                true
            } else {
                false
            }
        }
    }

    pub fn char(&mut self, c: char) -> bool {
        if chars!(self).next() == Some(c) {
            self.pos += c.len_utf8();
            true
        } else {
            false
        }
    }

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
