pub extern crate bitlab;
pub extern crate log;

#[macro_use]
mod macros;

pub const VERSION: u16 = 2;

#[cfg(test)]
mod test;

#[macro_use]
pub mod prelude;

pub mod ast;
pub mod btrace;
pub mod loc;

prelude! {}

use ast::{event::Event, *};

pub struct RawParser<'data> {
    data: &'data [u8],
    cursor: usize,
    big_endian: bool,
}

/// Basic functions.
impl<'data> RawParser<'data> {
    /// Constructor.
    pub fn new(data: &'data [u8]) -> Self {
        Self {
            data: data.into(),
            cursor: 0,
            big_endian: false,
        }
    }

    /// Data accessor.
    pub fn data(&self) -> &[u8] {
        &self.data
    }
}

/// RawParser helpers.
impl<'data> RawParser<'data> {
    pub fn check(&self, can_parse: usize, err: impl FnOnce() -> String) -> Res<()> {
        if self.cursor + can_parse <= self.data.len() {
            Ok(())
        } else {
            Err(err())
        }
    }

    /// Extracts the next byte and applies some action.
    pub fn next_byte_do<T>(&mut self, action: impl FnOnce(Option<u8>) -> T) -> T {
        let next_byte = self.data[self.cursor..].iter().next().cloned();
        if next_byte.is_some() {
            self.cursor += 1
        }
        action(next_byte)
    }

    /// True if the parser is at the end of its input.
    pub fn is_eof(&self) -> bool {
        self.cursor == self.data.len()
    }

    /// Yields the current position and the total length of the input text.
    pub(crate) fn position(&self) -> (usize, usize) {
        (self.cursor, self.data.len())
    }

    /// Yields a single line concise description of the current position.
    pub fn state(&self) -> String {
        if self.cursor < self.data.len() {
            format!(
                "currently at {} (of {}): `{:x}`",
                self.cursor,
                self.data.len(),
                self.data[self.cursor],
            )
        } else {
            "currently at EOF".into()
        }
    }
}

/// Basic parsers.
impl<'data> RawParser<'data> {
    /// Parses and checks the CTF magic number, and sets the big-endian flag.
    pub fn magic(&mut self) -> Res<()> {
        pinfo!(self, "    parsing magic number");
        const MAGIC: u32 = 0xc1fc1fc1;

        let start = self.pos();

        let magic_le = self.u32_le()?;

        if magic_le == MAGIC {
            self.big_endian = false
        } else {
            self.backtrack(start);
            let magic_be = self.u32_be()?;

            if magic_be == MAGIC {
                self.big_endian = true
            } else {
                bail!(
                    "not a legal CTF packet, expected magic number `{}`, \
                    got `{}` (le) or `{}` (be)",
                    MAGIC,
                    magic_le,
                    magic_be
                );
            }
        }
        Ok(())
    }

    /// Parses a string.
    pub fn string(&mut self) -> Res<&'data str> {
        pdebug!(self, "        parsing string");
        let start = self.cursor;
        let mut end = None;
        for (cnt, byte) in self.data[self.cursor..].iter().enumerate() {
            if *byte == 0 {
                end = Some(self.cursor + cnt);
                break;
            }
        }
        if let Some(end) = end {
            match std::str::from_utf8(&self.data[start..end]) {
                Ok(res) => {
                    self.cursor = end + 1;
                    Ok(res)
                }
                Err(e) => bail!(err!(expected format!("legal utf8 string: {}", e))),
            }
        } else {
            bail!(err!(expected "string"))
        }
    }

    /// Parses a `u8`.
    pub fn u8(&mut self) -> Res<u8> {
        pdebug!(self, "        parsing u8");
        if self.big_endian {
            self.u8_be()
        } else {
            self.u8_le()
        }
    }

    /// Parses a `u8`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use ctf::RawParser;
    /// let data = 213u8.to_le_bytes();
    /// let mut parser = RawParser::new(&data);
    /// # println!("state: {}", parser.state());
    /// assert_eq!(parser.u8_le(), Ok(213));
    /// assert!(parser.is_eof());
    /// ```
    pub fn u8_le(&mut self) -> Res<u8> {
        pdebug!(self, "        parsing u8 (le)");
        self.check(1, err!(|| expected "u8"))?;
        let res = u8::from_le_bytes([self.data[self.cursor]]);
        self.cursor += 1;
        Ok(res)
    }

    /// Parses a `u8`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use ctf::RawParser;
    /// let data = 213u8.to_be_bytes();
    /// let mut parser = RawParser::new(&data);
    /// # println!("state: {}", parser.state());
    /// assert_eq!(parser.u8_be(), Ok(213));
    /// assert!(parser.is_eof());
    /// ```
    pub fn u8_be(&mut self) -> Res<u8> {
        pdebug!(self, "        parsing u8 (be)");
        self.check(1, err!(|| expected "u8"))?;
        let res = u8::from_be_bytes([self.data[self.cursor]]);
        self.cursor += 1;
        Ok(res)
    }

    /// Parses a `u16`.
    pub fn u16(&mut self) -> Res<u16> {
        pdebug!(self, "        parsing u16");
        if self.big_endian {
            self.u16_be()
        } else {
            self.u16_le()
        }
    }

    /// Parses a `u16`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use ctf::RawParser;
    /// let data = 1_213u16.to_le_bytes();
    /// let mut parser = RawParser::new(&data);
    /// # println!("state: {}", parser.state());
    /// assert_eq!(parser.u16_le(), Ok(1_213));
    /// assert!(parser.is_eof());
    /// ```
    pub fn u16_le(&mut self) -> Res<u16> {
        pdebug!(self, "        parsing u16 (le)");
        self.check(2, err!(|| expected "u16"))?;
        let res = u16::from_le_bytes([self.data[self.cursor], self.data[self.cursor + 1]]);
        self.cursor += 2;
        Ok(res)
    }

    /// Parses a `u16`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use ctf::RawParser;
    /// let data = 1_213u16.to_be_bytes();
    /// let mut parser = RawParser::new(&data);
    /// # println!("state: {}", parser.state());
    /// assert_eq!(parser.u16_be(), Ok(1_213));
    /// assert!(parser.is_eof());
    /// ```
    pub fn u16_be(&mut self) -> Res<u16> {
        pdebug!(self, "        parsing u16 (be)");
        self.check(2, err!(|| expected "u16"))?;
        let res = u16::from_be_bytes([self.data[self.cursor], self.data[self.cursor + 1]]);
        self.cursor += 2;
        Ok(res)
    }

    /// Parses a `u32`.
    pub fn u32(&mut self) -> Res<u32> {
        pdebug!(self, "        parsing u32");
        if self.big_endian {
            self.u32_be()
        } else {
            self.u32_le()
        }
    }

    /// Parses a `u32`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use ctf::RawParser;
    /// let data = 1_701_213u32.to_le_bytes();
    /// let mut parser = RawParser::new(&data);
    /// # println!("state: {}", parser.state());
    /// assert_eq!(parser.u32_le(), Ok(1_701_213));
    /// assert!(parser.is_eof());
    /// ```
    pub fn u32_le(&mut self) -> Res<u32> {
        pdebug!(self, "        parsing u32 (le)");
        self.check(4, err!(|| expected "u32"))?;
        let res = u32::from_le_bytes([
            self.data[self.cursor],
            self.data[self.cursor + 1],
            self.data[self.cursor + 2],
            self.data[self.cursor + 3],
        ]);
        self.cursor += 4;
        Ok(res)
    }

    /// Parses a `u32`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use ctf::RawParser;
    /// let data = 1_701_213u32.to_be_bytes();
    /// let mut parser = RawParser::new(&data);
    /// # println!("state: {}", parser.state());
    /// assert_eq!(parser.u32_be(), Ok(1_701_213));
    /// assert!(parser.is_eof());
    /// ```
    pub fn u32_be(&mut self) -> Res<u32> {
        pdebug!(self, "        parsing u32 (be)");
        self.check(4, err!(|| expected "u32"))?;
        let res = u32::from_be_bytes([
            self.data[self.cursor],
            self.data[self.cursor + 1],
            self.data[self.cursor + 2],
            self.data[self.cursor + 3],
        ]);
        self.cursor += 4;
        Ok(res)
    }

    /// Parses a `u64`.
    pub fn u64(&mut self) -> Res<u64> {
        pdebug!(self, "        parsing u64");
        if self.big_endian {
            self.u64_be()
        } else {
            self.u64_le()
        }
    }

    /// Parses a `u64`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use ctf::RawParser;
    /// let data = 7_501_701_213u64.to_be_bytes();
    /// let mut parser = RawParser::new(&data);
    /// # println!("state: {}", parser.state());
    /// assert_eq!(parser.u64_be(), Ok(7_501_701_213));
    /// assert!(parser.is_eof());
    /// ```
    pub fn u64_be(&mut self) -> Res<u64> {
        pdebug!(self, "        parsing u64 (be)");
        self.check(8, err!(|| expected "u64"))?;
        let res = u64::from_be_bytes([
            self.data[self.cursor],
            self.data[self.cursor + 1],
            self.data[self.cursor + 2],
            self.data[self.cursor + 3],
            self.data[self.cursor + 4],
            self.data[self.cursor + 5],
            self.data[self.cursor + 6],
            self.data[self.cursor + 7],
        ]);
        self.cursor += 8;
        Ok(res)
    }

    /// Parses a `u64`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use ctf::RawParser;
    /// let data = 7_501_701_213u64.to_le_bytes();
    /// let mut parser = RawParser::new(&data);
    /// # println!("state: {}", parser.state());
    /// assert_eq!(parser.u64_le(), Ok(7_501_701_213));
    /// assert!(parser.is_eof());
    /// ```
    pub fn u64_le(&mut self) -> Res<u64> {
        pdebug!(self, "        parsing u64 (le)");
        self.check(8, err!(|| expected "u64"))?;
        let res = u64::from_le_bytes([
            self.data[self.cursor],
            self.data[self.cursor + 1],
            self.data[self.cursor + 2],
            self.data[self.cursor + 3],
            self.data[self.cursor + 4],
            self.data[self.cursor + 5],
            self.data[self.cursor + 6],
            self.data[self.cursor + 7],
        ]);
        self.cursor += 8;
        Ok(res)
    }

    /// Parses a `i8`.
    pub fn i8(&mut self) -> Res<i8> {
        pdebug!(self, "        parsing i8");
        if self.big_endian {
            self.i8_be()
        } else {
            self.i8_le()
        }
    }

    /// Parses a `i8`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use ctf::RawParser;
    /// let data = 7i8.to_le_bytes();
    /// let mut parser = RawParser::new(&data);
    /// # println!("state: {}", parser.state());
    /// assert_eq!(parser.i8_le(), Ok(7));
    /// assert!(parser.is_eof());
    /// ```
    pub fn i8_le(&mut self) -> Res<i8> {
        pdebug!(self, "        parsing i8 (le)");
        self.check(1, err!(|| expected "i8"))?;
        let res = i8::from_le_bytes([self.data[self.cursor]]);
        self.cursor += 1;
        Ok(res)
    }

    /// Parses a `i8`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use ctf::RawParser;
    /// let data = 7i8.to_be_bytes();
    /// let mut parser = RawParser::new(&data);
    /// # println!("state: {}", parser.state());
    /// assert_eq!(parser.i8_be(), Ok(7));
    /// assert!(parser.is_eof());
    /// ```
    pub fn i8_be(&mut self) -> Res<i8> {
        pdebug!(self, "        parsing i8 (be)");
        self.check(1, err!(|| expected "i8"))?;
        let res = i8::from_be_bytes([self.data[self.cursor]]);
        self.cursor += 1;
        Ok(res)
    }

    /// Parses a `i16`.
    pub fn i16(&mut self) -> Res<i16> {
        pdebug!(self, "        parsing i16");
        if self.big_endian {
            self.i16_be()
        } else {
            self.i16_le()
        }
    }

    /// Parses a `i16`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use ctf::RawParser;
    /// let data = 1_213i16.to_le_bytes();
    /// let mut parser = RawParser::new(&data);
    /// # println!("state: {}", parser.state());
    /// assert_eq!(parser.i16_le(), Ok(1_213));
    /// assert!(parser.is_eof());
    /// ```
    pub fn i16_le(&mut self) -> Res<i16> {
        pdebug!(self, "        parsing i16 (le)");
        self.check(2, err!(|| expected "i16"))?;
        let res = i16::from_le_bytes([self.data[self.cursor], self.data[self.cursor + 1]]);
        self.cursor += 2;
        Ok(res)
    }

    /// Parses a `i16`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use ctf::RawParser;
    /// let data = 1_213i16.to_be_bytes();
    /// let mut parser = RawParser::new(&data);
    /// # println!("state: {}", parser.state());
    /// assert_eq!(parser.i16_be(), Ok(1_213));
    /// assert!(parser.is_eof());
    /// ```
    pub fn i16_be(&mut self) -> Res<i16> {
        pdebug!(self, "        parsing i16 (be)");
        self.check(2, err!(|| expected "i16"))?;
        let res = i16::from_be_bytes([self.data[self.cursor], self.data[self.cursor + 1]]);
        self.cursor += 2;
        Ok(res)
    }

    /// Parses a `i32`.
    pub fn i32(&mut self) -> Res<i32> {
        pdebug!(self, "        parsing i32");
        if self.big_endian {
            self.i32_be()
        } else {
            self.i32_le()
        }
    }

    /// Parses a `i32`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use ctf::RawParser;
    /// let data = 1_701_213i32.to_le_bytes();
    /// let mut parser = RawParser::new(&data);
    /// # println!("state: {}", parser.state());
    /// assert_eq!(parser.i32_le(), Ok(1_701_213));
    /// assert!(parser.is_eof());
    /// ```
    pub fn i32_le(&mut self) -> Res<i32> {
        pdebug!(self, "        parsing i32 (le)");
        self.check(4, err!(|| expected "i32"))?;
        let res = i32::from_le_bytes([
            self.data[self.cursor],
            self.data[self.cursor + 1],
            self.data[self.cursor + 2],
            self.data[self.cursor + 3],
        ]);
        self.cursor += 4;
        Ok(res)
    }

    /// Parses a `i32`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use ctf::RawParser;
    /// let data = 1_701_213i32.to_be_bytes();
    /// let mut parser = RawParser::new(&data);
    /// # println!("state: {}", parser.state());
    /// assert_eq!(parser.i32_be(), Ok(1_701_213));
    /// assert!(parser.is_eof());
    /// ```
    pub fn i32_be(&mut self) -> Res<i32> {
        pdebug!(self, "        parsing i32 (be)");
        self.check(4, err!(|| expected "i32"))?;
        let res = i32::from_be_bytes([
            self.data[self.cursor],
            self.data[self.cursor + 1],
            self.data[self.cursor + 2],
            self.data[self.cursor + 3],
        ]);
        self.cursor += 4;
        Ok(res)
    }

    /// Parses a `i64`.
    pub fn i64(&mut self) -> Res<i64> {
        pdebug!(self, "        parsing i64");
        if self.big_endian {
            self.i64_be()
        } else {
            self.i64_le()
        }
    }

    /// Parses a `i64`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use ctf::RawParser;
    /// let data = 7_501_701_213i64.to_le_bytes();
    /// let mut parser = RawParser::new(&data);
    /// # println!("state: {}", parser.state());
    /// assert_eq!(parser.i64_le(), Ok(7_501_701_213));
    /// assert!(parser.is_eof());
    /// ```
    pub fn i64_le(&mut self) -> Res<i64> {
        pdebug!(self, "        parsing i64 (le)");
        self.check(8, err!(|| expected "i64"))?;
        let res = i64::from_le_bytes([
            self.data[self.cursor],
            self.data[self.cursor + 1],
            self.data[self.cursor + 2],
            self.data[self.cursor + 3],
            self.data[self.cursor + 4],
            self.data[self.cursor + 5],
            self.data[self.cursor + 6],
            self.data[self.cursor + 7],
        ]);
        self.cursor += 8;
        Ok(res)
    }

    /// Parses a `i64`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use ctf::RawParser;
    /// let data = 7_501_701_213i64.to_be_bytes();
    /// let mut parser = RawParser::new(&data);
    /// # println!("state: {}", parser.state());
    /// assert_eq!(parser.i64_be(), Ok(7_501_701_213));
    /// assert!(parser.is_eof());
    /// ```
    pub fn i64_be(&mut self) -> Res<i64> {
        pdebug!(self, "        parsing i64 (be)");
        self.check(8, err!(|| expected "i64"))?;
        let res = i64::from_be_bytes([
            self.data[self.cursor],
            self.data[self.cursor + 1],
            self.data[self.cursor + 2],
            self.data[self.cursor + 3],
            self.data[self.cursor + 4],
            self.data[self.cursor + 5],
            self.data[self.cursor + 6],
            self.data[self.cursor + 7],
        ]);
        self.cursor += 8;
        Ok(res)
    }

    /// Parses a clock value.
    pub fn clock(&mut self) -> Res<u64> {
        pdebug!(self, "        parsing clock");
        self.u64().subst_err(err!(|| expected "clock value"))
    }

    pub fn f64(&mut self) -> Res<f64> {
        pdebug!(self, "        parsing f64");
        if self.big_endian {
            self.f64_be()
        } else {
            self.f64_le()
        }
    }
    pub fn f64_be(&mut self) -> Res<f64> {
        pdebug!(self, "        parsing f64 (be)");
        self.check(8, err!(|| expected "f64"))?;
        let res = f64::from_be_bytes([
            self.data[self.cursor],
            self.data[self.cursor + 1],
            self.data[self.cursor + 2],
            self.data[self.cursor + 3],
            self.data[self.cursor + 4],
            self.data[self.cursor + 5],
            self.data[self.cursor + 6],
            self.data[self.cursor + 7],
        ]);
        self.cursor += 8;
        Ok(res)
    }
    pub fn f64_le(&mut self) -> Res<f64> {
        pdebug!(self, "        parsing f64 (le)");
        self.check(8, err!(|| expected "f64"))?;
        let res = f64::from_le_bytes([
            self.data[self.cursor],
            self.data[self.cursor + 1],
            self.data[self.cursor + 2],
            self.data[self.cursor + 3],
            self.data[self.cursor + 4],
            self.data[self.cursor + 5],
            self.data[self.cursor + 6],
            self.data[self.cursor + 7],
        ]);
        self.cursor += 8;
        Ok(res)
    }
}

/// More advanced parsers.
impl<'data> RawParser<'data> {
    /// Parses a `usize` in memtrace's variable-length format.
    pub fn v_usize(&mut self) -> Res<usize> {
        pdebug!(self, "    parsing v_usize");
        let variant: u8 = self
            .u8()
            .chain_err(err!(|| expected "variable-length usize"))?;

        let res = match variant {
            0..=252 => convert(variant, "v_usize: u8"),
            253 => convert(self.u16()?, "v_usize: u16"),
            254 => convert(self.u32()?, "v_usize: u32"),
            255 => convert(self.u64()?, "v_usize: u64"),
        };

        Ok(res)
    }

    pub fn alloc(&mut self, cxt: &mut Cxt<'data>, short: Option<usize>) -> Res<ast::event::Alloc> {
        pinfo!(self, "parsing alloc");
        let alloc_id = cxt.next_alloc_id();
        let (is_short, len, nsamples, is_major) = if let Some(len) = short {
            (true, len, 1, false)
        } else {
            let len = self.v_usize()?;
            let nsample = self.v_usize()?;
            let is_major = match self.u8()? {
                0 => false,
                1 => true,
                n => bail!(err!(expected format!("boolean as a 0- or 1-valued u8, found {}", n))),
            };
            (false, len, nsample, is_major)
        };
        let common_pref_len = self.v_usize()?;
        let nencoded = if is_short {
            self.u8()? as usize
        } else {
            self.u16()? as usize
        };

        pinfo!(
            self,
            "    nencoded: {}, common_pref_len: {}",
            nencoded,
            common_pref_len
        );

        let (backtrace, backtrace_len) =
            cxt.btrace.get_backtrace(self, nencoded, common_pref_len)?;

        Ok(ast::event::Alloc {
            id: alloc_id,
            len,
            nsamples,
            is_major,
            common_pref_len,
            backtrace,
            backtrace_len,
        })
    }

    pub fn id_from_delta(&mut self, cxt: &mut Cxt<'data>) -> Res<u64> {
        let id_delta = self.v_usize()? as u64;
        Ok(cxt.current_alloc_id() - 1 - id_delta)
    }

    pub fn locs(&mut self, cxt: &mut Cxt<'data>) -> Res<ast::Locs<'data>> {
        pinfo!(self, "    parsing locations");
        let id = convert(self.u64()?, "locs: id");
        let len = convert(self.u8()?, "locs: len");
        pinfo!(self, "    -> parsing {} location(s)", len);
        let mut locs = SVec16::with_capacity(len);
        for _ in 0..len {
            let loc = loc::Location::parse(self, &mut cxt.loc)?;
            locs.push(loc)
        }
        Ok(ast::Locs { id, locs })
    }

    pub fn cache_check(&mut self) -> Res<CacheCheck> {
        let ix = self.u16()?;
        let pred = self.u16()?;
        let value = self.u64()?;
        Ok(CacheCheck { ix, pred, value })
    }

    pub fn event_kind(&mut self, header: &header::Header) -> Res<(event::Kind, Clock)> {
        pinfo!(self, "    parsing event kind");
        const EVENT_HEADER_TIME_MASK: u32 = 0x1ffffff;
        const EVENT_HEADER_TIME_MASK_I64: u64 = EVENT_HEADER_TIME_MASK as u64;
        const EVENT_HEADER_TIME_LEN: u32 = 25;

        let code = self.u32()?;
        pinfo!(self, "code: {}", code);
        pinfo!(self, "timestamp begin: {}", header.timestamp.begin);
        // This conversion will pretty much always be OOB.
        let timestamp_begin = header.timestamp.begin as u32;
        let start_low = EVENT_HEADER_TIME_MASK & timestamp_begin;
        let time_low: u64 = convert(
            {
                let time_low = EVENT_HEADER_TIME_MASK & code;
                if time_low < start_low {
                    // Overflow.
                    time_low + (1u32.rotate_left(EVENT_HEADER_TIME_LEN))
                } else {
                    time_low
                }
            },
            "event_kind: time_low",
        );

        let time = (header.timestamp.begin & (!EVENT_HEADER_TIME_MASK_I64)) + time_low;
        // if !header.timestamp.contains(time) {
        //     bail!(
        //         "inconsistent event header time, expected `{} <= {} <= {}`",
        //         header.timestamp.begin,
        //         time,
        //         header.timestamp.end
        //     )
        // }
        let ev_code = code >> EVENT_HEADER_TIME_LEN;
        pinfo!(self, "ev code: {}, time: {}", ev_code, time);
        let ev = event::Kind::from_code(ev_code)?;
        Ok((ev, time))
    }

    /// Parses an event header.
    ///
    /// At binary level, an event header is 32 bits storing two integers: one over 25 bits and one
    /// over 7 bits.
    pub fn event_header(&mut self) -> Res<header::Event> {
        pinfo!(self, "parsing event header");
        self.raw_event_header()
            .chain_err(|| "while parsing event header")
    }
    pub fn raw_event_header(&mut self) -> Res<header::Event> {
        use bitlab::*;

        // Parsing in big endian, we're gonna parse the bits and need them in the same order as they
        // appear in the data.
        let full = self.u32()?;

        let mut timestamp = 0u32;
        let mut id = 0u8;

        // This block goes through `full` in order and puts
        // - the first 25 bits (low endian) in `timestamp` (big endian)
        // - the last 7 bits (low endian) in `id` (big endian)
        {
            macro_rules! err {
                ($variant:tt $value:expr, on $ty:tt) => {
                    || {
                        format!(
                            "while {}ting bit {} on a {}",
                            stringify!($variant),
                            $value,
                            stringify!($ty)
                        )
                    }
                };
            }

            for idx in 0..25 {
                if full
                    .get_bit(idx)
                    .to_res()
                    .chain_err(err!(get idx, on u32))?
                {
                    timestamp = timestamp
                        .set_bit(31 - idx)
                        .to_res()
                        .chain_err(err!(set 31 - idx, on u32))?;
                }
            }
            for idx in 0..7 {
                if full
                    .get_bit(25 + idx)
                    .to_res()
                    .chain_err(err!(get 25 + idx, on u32))?
                {
                    id = id
                        .set_bit(7 - idx)
                        .to_res()
                        .chain_err(err!(set 7 - idx, on u8))?;
                }
            }
        }

        Ok(header::Event { timestamp, id })
    }

    pub fn packet_header(&mut self) -> Res<header::Packet> {
        pinfo!(self, "parsing packet header");
        self.raw_package_header()
            .map(|(header, cache_check)| header::Packet::new(header, cache_check))
            .chain_err(|| "while parsing ctf header")
    }
    pub fn ctf_header(&mut self) -> Res<header::Ctf> {
        pinfo!(self, "parsing ctf header");
        self.raw_package_header()
            .map(|(header, _)| header::Ctf::new(header, self.big_endian))
            .chain_err(|| "while parsing package header")
    }
    fn raw_package_header(&mut self) -> Res<(header::Header, CacheCheck)> {
        let start = self.pos();

        self.magic()?;

        let packet_size_bits = self.u32()?;
        pinfo!(self, "    package size bits {}", packet_size_bits);

        let (begin, end) = (self.clock()?, self.clock()?);
        pinfo!(self, "    begin/end times {}/{}", begin, end);
        let timestamp = Span::new(begin, end).chain_err(|| {
            format!(
                "while parsing timestamp begin/end values ({}/{})",
                begin, end
            )
        })?;

        let _flush_duration = self.u32()?;
        pinfo!(self, "    flush duration {}", _flush_duration);

        let version = self.u16()?;
        pinfo!(self, "    version {}", version);

        let pid = self.u64()?;
        pinfo!(self, "    pid {}", pid);

        let cache_check = self.cache_check()?;

        let (alloc_begin, alloc_end) = (self.u64()?, self.u64()?);
        pinfo!(
            self,
            "    alloc begin/end times {}/{}",
            alloc_begin,
            alloc_end
        );
        let alloc_id = Span::new(alloc_begin, alloc_end).chain_err(|| {
            format!(
                "while parsing allocation id begin/end values ({}/{})",
                alloc_begin, alloc_end
            )
        })?;

        if VERSION == version {
            ()
        } else {
            match (VERSION, version) {
                (2, 1) => (),
                _ => bail!("found trace format v{}, expected v{}", version, VERSION),
            }
        }

        let header_size: u32 = convert(self.pos() - start, "raw_package_header: header_size");

        if packet_size_bits % 8 != 0 {
            bail!("illegal packet size {}, not a legal number of bits")
        }

        let total_content_size = packet_size_bits / 8;
        let content_size = total_content_size - header_size;
        pinfo!(
            self,
            "    content size in bytes {} = ({} / 8) - {}",
            content_size,
            packet_size_bits,
            header_size,
        );

        Ok((
            header::Header {
                content_size,
                total_content_size,
                timestamp,
                alloc_id,
                pid,
                version,
            },
            cache_check,
        ))
    }

    pub fn trace_info(&mut self, header: &header::Ctf) -> Res<event::Info> {
        pinfo!(self, "parsing trace info");
        // let start_time = header.timestamp.begin;
        let sample_rate = self.f64()?;
        pinfo!(self, "    sample rate {}", sample_rate);

        let word_size = self.u8()?;
        pinfo!(self, "    word size {}", word_size);

        let exe_name = self.string()?;
        pinfo!(self, "    exe name {:?}", exe_name);

        let host_name = self.string()?;
        pinfo!(self, "    host name {:?}", host_name);

        let exe_params = self.string()?;
        pinfo!(self, "    exe params {:?}", exe_params);

        let pid = self.u64()?;
        pinfo!(self, "    pid {}", pid);

        let context = if header.has_context() {
            Some(self.string()?.to_string())
        } else {
            None
        };
        pinfo!(self, "    context {:?}", context);

        Ok(event::Info {
            sample_rate,
            word_size,
            exe_name: exe_name.into(),
            host_name: host_name.into(),
            exe_params: exe_params.into(),
            pid,
            context,
        })
    }
}

pub struct Parser<'data> {
    parser: RawParser<'data>,
    header: header::Ctf,
}

impl<'data> Parser<'data> {
    /// Constructor.
    pub fn new(data: &'data [u8]) -> Res<Self> {
        let mut parser = RawParser::new(data);
        let header = parser.ctf_header()?;
        let (event_kind, _event_time) = parser.event_kind(&header)?;
        if !event_kind.is_info() {
            bail!(
                "expected initial event to be an info event, found {:?}",
                event_kind
            )
        }
        Ok(Self { parser, header })
    }

    /// Header accessor.
    pub fn header(&self) -> &header::Ctf {
        &self.header
    }
}

/// Pseudo-parsers: parses a very tiny amout of data to produce a subparser.
impl<'data> Parser<'data> {
    pub fn work_on_packets(
        &mut self,
        mut event_action: impl FnMut(usize, Event<'data>) -> Res<()>,
    ) -> Res<()> {
        let parser = &mut self.parser;
        let mut cxt = Cxt::new();
        let mut package_count = 0;

        while !parser.is_eof() {
            pinfo!(
                parser,
                "currently at {}/{}",
                parser.cursor,
                parser.data.len()
            );
            let my_header = parser.packet_header()?;
            let content_len: usize = convert(my_header.content_size, "packets: content_len");
            let packet_end = parser.cursor + content_len;
            pinfo!(
                parser,
                "next packet: {} bytes -> {}/{}",
                content_len,
                packet_end,
                parser.data.len()
            );
            if packet_end > parser.data.len() {
                bail!(err!(expected format!(
                    "legal packet size: not enough data left ({}/{})",
                    content_len, parser.data.len() - parser.cursor,
                )))
            }
            PacketParser::new(&parser.data[parser.cursor..packet_end], my_header)
                .iter_events(&mut cxt, |event| event_action(package_count, event))?;
            parser.cursor = packet_end;
            package_count += 1;
        }

        pinfo!(self, "parsed {} package(s)", package_count);

        Ok(())
    }

    pub fn work(&mut self, event_action: impl FnMut(usize, Event<'data>) -> Res<()>) -> Res<()> {
        let _trace_info = self.parser.trace_info(&self.header)?;
        pinfo!(self, "done parsing trace info");
        self.work_on_packets(event_action)?;
        Ok(())
    }
}

pub struct Cxt<'data> {
    loc: loc::Cxt<'data>,
    btrace: btrace::Cxt,
    alloc_count: u64,
}
impl<'data> Cxt<'data> {
    pub fn new() -> Self {
        Self {
            loc: loc::Cxt::new(),
            btrace: btrace::Cxt::new(),
            alloc_count: 0u64,
        }
    }
    pub fn next_alloc_id(&mut self) -> u64 {
        let mut next = self.alloc_count + 1;
        std::mem::swap(&mut self.alloc_count, &mut next);
        next
    }
    pub fn current_alloc_id(&self) -> u64 {
        self.alloc_count
    }
}

pub struct PacketParser<'data> {
    parser: RawParser<'data>,
    my_header: header::Packet,
}

impl<'data> PacketParser<'data> {
    pub fn new(input: &'data [u8], my_header: header::Packet) -> Self {
        Self {
            parser: RawParser::new(input),
            my_header,
        }
    }

    fn event(&mut self, cxt: &mut Cxt<'data>) -> Res<Event<'data>> {
        let (event_kind, event_time) = self.parser.event_kind(&self.my_header)?;
        pinfo!(self, "event: {:?} ({})", event_kind, event_time);

        let event = match event_kind {
            event::Kind::Alloc => {
                let alloc = self.alloc(cxt, None)?;
                Event::Alloc(alloc)
            }
            event::Kind::SmallAlloc(n) => {
                let alloc = self.alloc(cxt, Some(convert(n, "event: SmallAlloc(n)")))?;
                Event::Alloc(alloc)
            }
            event::Kind::Promotion => {
                let alloc_id = self.id_from_delta(cxt)?;
                Event::Promotion(alloc_id)
            }
            event::Kind::Collection => {
                let alloc_id = self.id_from_delta(cxt)?;
                Event::Collection(alloc_id)
            }
            event::Kind::Locs => {
                let locs = self.locs(cxt)?;
                Event::Locs(locs)
            }

            // Can't have more than two info events.
            event::Kind::Info => {
                bail!(err!(expected "non-info event: having more than two info events is illegal"))
            }
        };

        pinfo!(self, "    {:?}", event);

        Ok(event)
    }

    pub fn iter_events(
        mut self,
        cxt: &mut Cxt<'data>,
        mut event_do: impl FnMut(Event<'data>) -> Res<()>,
    ) -> Res<()> {
        while !self.is_eof() {
            event_do(self.event(cxt)?)?
        }
        Ok(())
    }
}

mod impls {
    use std::ops::{Deref, DerefMut};

    use super::*;

    impl<'data> Deref for Parser<'data> {
        type Target = RawParser<'data>;
        fn deref(&self) -> &RawParser<'data> {
            &self.parser
        }
    }
    impl<'data> DerefMut for Parser<'data> {
        fn deref_mut(&mut self) -> &mut RawParser<'data> {
            &mut self.parser
        }
    }

    impl<'data> Deref for PacketParser<'data> {
        type Target = RawParser<'data>;
        fn deref(&self) -> &RawParser<'data> {
            &self.parser
        }
    }
    impl<'data> DerefMut for PacketParser<'data> {
        fn deref_mut(&mut self) -> &mut RawParser<'data> {
            &mut self.parser
        }
    }
}
