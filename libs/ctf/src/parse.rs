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

//! Contains the parser for memtrace's CTF dump format.
//!
//! The parser is designed in such a way that there are no runtime checks related to big-/low-endian
//! encoding. The main building block is [`RawParser`], which features parsing primitives for a wide
//! range of types. In particular, integers and floats have a big-endian version and a low-endian
//! one.
//!
//! The actual parser type is [`Parser`], which takes a type parameter `Endian` corresponding to the
//! endian specification. It can be either [`LowEndian`] or [`BigEndian`]. Both these types are not
//! meant to be constructed in practice, they are just type-level information encoding the endian
//! specification to use. So, in practice, `Parser<LowEndian>` directly parses all integers/floats
//! as assuming a low-endian convention, and similarily for `Parser<BigEndian>`.
//!
//! Deciding which endian convention to use happens when parsing the memtrace CTF magic number. This
//! is done by creating a [`RawParser`], and calling the [`try_magic`] method. Note that when
//! parsing a CTF file, the magic number is the first thing parsed.
//!
//! This gives either a `Parser<LowEndian>` or a `Parser<BigEndian` (or an error). Once parsing
//! starts with either of these types, a change in endian convention is considered an error.
//!
//! [`try_magic`]: RawParser::try_magic (try_magic method on RawParser)

prelude! {}

/// Memtrace CTF magic number.
const MAGIC: u32 = 0xc1fc1fc1;

/// A position in the parser (zero-cost wrapper around a usize).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Pos {
    pos: usize,
}
impl std::ops::Sub for Pos {
    type Output = usize;
    fn sub(self, other: Self) -> usize {
        self.pos - other.pos
    }
}
impl std::ops::Deref for Pos {
    type Target = usize;
    fn deref(&self) -> &usize {
        &self.pos
    }
}
impl std::fmt::Display for Pos {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        self.pos.fmt(fmt)
    }
}

base::new_time_stats! {
    struct Prof {
        basic_parsing => "basic parsing",
        alloc => "alloc",
        collection => "collection",
        locs => "locations",
    }
}

/// Parsing context.
///
/// Stores
///
/// - the location context,
/// - the backtrace context, and
/// - the allocation UID counter.
pub struct Cxt<'data> {
    loc: loc::Cxt<'data>,
    btrace: btrace::Cxt,
    alloc_count: u64,
    prof: Prof,
}
impl<'data> Cxt<'data> {
    /// Constructor.
    pub fn new() -> Self {
        Self {
            loc: loc::Cxt::new(),
            btrace: btrace::Cxt::new(),
            alloc_count: 0u64,
            prof: Prof::new(),
        }
    }

    /// Yields the next available allocation UID and increments its internal counter.
    pub fn next_alloc_id(&mut self) -> u64 {
        // New value of the counter.
        let mut next = self.alloc_count + 1;
        // `next` receives the actual next UID.
        std::mem::swap(&mut self.alloc_count, &mut next);
        next
    }
    /// Same as [`next_alloc_id`][Cxt::next_alloc_id] but does not increment the internal counter.
    pub fn peek_next_alloc_id(&self) -> u64 {
        self.alloc_count
    }
}

/// Raw parser.
///
/// - provides basic and intermediate parsing functions used by [`Parser`] and [`PacketParser`];
/// - works at byte-level.
pub struct RawParser<'data> {
    /// Data to parse.
    data: &'data [u8],
    /// Current position in the text.
    cursor: usize,
    /// True if we're parsing big-endian numbers.
    ///
    /// This is detected/modified by parsing the memtrace CTF [magic number].
    ///
    /// [magic number]: #method.magic
    big_endian: bool,
    /// Offset from the start of the original input.
    ///
    /// Used by [`PacketParser`], which works on a slice of the original input, for consistent
    /// error-reporting.
    offset: usize,
}

impl<'data> std::ops::Index<Pos> for RawParser<'data> {
    type Output = u8;
    fn index(&self, pos: Pos) -> &u8 {
        &self.data[pos.pos]
    }
}

/// Basic functions.
impl<'data> RawParser<'data> {
    /// Constructor.
    ///
    /// - `data`: input bytes to parse;
    /// - `offset` offset from the start of the original input. Used by [`PacketParser`], which
    ///   works on a slice of the original input, for consistent error-reporting.
    pub fn new(data: &'data [u8], offset: usize) -> Self {
        Self {
            data: data.into(),
            cursor: 0,
            big_endian: false,
            offset,
        }
    }

    /// Data accessor.
    pub fn data(&self) -> &'data [u8] {
        self.data
    }

    /// Consumes some bytes from the input, move the cursor at the end of these bytes.
    pub fn take(&mut self, byte_count: usize) -> &'data [u8] {
        debug_assert!(self.cursor + byte_count <= self.data.len());
        let res = &self.data[self.cursor..self.cursor + byte_count];
        self.cursor += byte_count;
        res
    }
}

/// Position related functions.
impl<'data> RawParser<'data> {
    /// Position accessor.
    pub fn pos(&self) -> Pos {
        Pos { pos: self.cursor }
    }
    /// Retrieves the byte at some position.
    pub fn get(&self, pos: Pos) -> Option<u8> {
        self.data.get(pos.pos).cloned()
    }
    /// Backtracks the parser to a **previous** position.
    ///
    /// # Panics
    ///
    /// - when `pos` is greater than the current position.
    pub fn backtrack(&mut self, pos: Pos) {
        debug_assert!(self.cursor >= pos.pos);
        self.cursor = pos.pos
    }
}

/// RawParser helpers.
impl<'data> RawParser<'data> {
    fn check<E>(&self, can_parse: usize, err: impl FnOnce() -> E) -> Res<()>
    where
        E: Into<err::Error>,
    {
        if self.cursor + can_parse <= self.data.len() {
            Ok(())
        } else {
            Err(err().into().into())
        }
    }

    /// True if the parser is at the end of its input.
    pub fn is_eof(&self) -> bool {
        self.cursor == self.data.len()
    }

    /// Yields the current position and the total length of the input text.
    pub fn real_position(&self) -> (usize, usize) {
        (self.cursor + self.offset, self.data.len())
    }

    /// Yields a single-line, concise description of the current position.
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
                Err(e) => bail!(parse_error!(expected format!("legal utf8 string: {}", e))),
            }
        } else {
            bail!(parse_error!(expected "string"))
        }
    }

    #[inline(always)]
    unsafe fn get_unchecked(&self, pos: usize) -> u8 {
        *self.data.get_unchecked(pos)
    }

    /// Parses a `u8`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use ctf::parse::RawParser;
    /// let data = 213u8.to_le_bytes();
    /// let mut parser = RawParser::new(&data, 0);
    /// # println!("state: {}", parser.state());
    /// assert_eq!(parser.u8_le().unwrap(), 213);
    /// assert!(parser.is_eof());
    /// ```
    pub fn u8_le(&mut self) -> Res<u8> {
        pdebug!(self, "        parsing u8 (le)");
        self.check(1, parse_error!(|| expected "u8"))?;
        let res = u8::from_le_bytes(unsafe { [self.get_unchecked(self.cursor)] });
        self.cursor += 1;
        Ok(res)
    }

    /// Parses a `u8`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use ctf::parse::RawParser;
    /// let data = 213u8.to_be_bytes();
    /// let mut parser = RawParser::new(&data, 0);
    /// # println!("state: {}", parser.state());
    /// assert_eq!(parser.u8_be().unwrap(), 213);
    /// assert!(parser.is_eof());
    /// ```
    pub fn u8_be(&mut self) -> Res<u8> {
        pdebug!(self, "        parsing u8 (be)");
        self.check(1, parse_error!(|| expected "u8"))?;
        let res = u8::from_be_bytes(unsafe { [self.get_unchecked(self.cursor)] });
        self.cursor += 1;
        Ok(res)
    }

    /// Parses a `u16`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use ctf::parse::RawParser;
    /// let data = 1_213u16.to_le_bytes();
    /// let mut parser = RawParser::new(&data, 0);
    /// # println!("state: {}", parser.state());
    /// assert_eq!(parser.u16_le().unwrap(), 1_213);
    /// assert!(parser.is_eof());
    /// ```
    pub fn u16_le(&mut self) -> Res<u16> {
        pdebug!(self, "        parsing u16 (le)");
        self.check(2, parse_error!(|| expected "u16"))?;
        let res = u16::from_le_bytes(unsafe {
            [
                self.get_unchecked(self.cursor),
                self.get_unchecked(self.cursor + 1),
            ]
        });
        self.cursor += 2;
        Ok(res)
    }

    /// Parses a `u16`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use ctf::parse::RawParser;
    /// let data = 1_213u16.to_be_bytes();
    /// let mut parser = RawParser::new(&data, 0);
    /// # println!("state: {}", parser.state());
    /// assert_eq!(parser.u16_be().unwrap(), 1_213);
    /// assert!(parser.is_eof());
    /// ```
    pub fn u16_be(&mut self) -> Res<u16> {
        pdebug!(self, "        parsing u16 (be)");
        self.check(2, parse_error!(|| expected "u16"))?;
        let res = u16::from_be_bytes(unsafe {
            [
                self.get_unchecked(self.cursor),
                self.get_unchecked(self.cursor + 1),
            ]
        });
        self.cursor += 2;
        Ok(res)
    }

    /// Parses a `u32`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use ctf::parse::RawParser;
    /// let data = 1_701_213u32.to_le_bytes();
    /// let mut parser = RawParser::new(&data, 0);
    /// # println!("state: {}", parser.state());
    /// assert_eq!(parser.u32_le().unwrap(), 1_701_213);
    /// assert!(parser.is_eof());
    /// ```
    pub fn u32_le(&mut self) -> Res<u32> {
        pdebug!(self, "        parsing u32 (le)");
        self.check(4, parse_error!(|| expected "u32"))?;
        let res = u32::from_le_bytes(unsafe {
            [
                self.get_unchecked(self.cursor),
                self.get_unchecked(self.cursor + 1),
                self.get_unchecked(self.cursor + 2),
                self.get_unchecked(self.cursor + 3),
            ]
        });
        self.cursor += 4;
        Ok(res)
    }

    /// Parses a `u32`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use ctf::parse::RawParser;
    /// let data = 1_701_213u32.to_be_bytes();
    /// let mut parser = RawParser::new(&data, 0);
    /// # println!("state: {}", parser.state());
    /// assert_eq!(parser.u32_be().unwrap(), 1_701_213);
    /// assert!(parser.is_eof());
    /// ```
    pub fn u32_be(&mut self) -> Res<u32> {
        pdebug!(self, "        parsing u32 (be)");
        self.check(4, parse_error!(|| expected "u32"))?;
        let res = u32::from_be_bytes(unsafe {
            [
                self.get_unchecked(self.cursor),
                self.get_unchecked(self.cursor + 1),
                self.get_unchecked(self.cursor + 2),
                self.get_unchecked(self.cursor + 3),
            ]
        });
        self.cursor += 4;
        Ok(res)
    }

    /// Parses a `u64`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use ctf::parse::RawParser;
    /// let data = 7_501_701_213u64.to_be_bytes();
    /// let mut parser = RawParser::new(&data, 0);
    /// # println!("state: {}", parser.state());
    /// assert_eq!(parser.u64_be().unwrap(), 7_501_701_213);
    /// assert!(parser.is_eof());
    /// ```
    pub fn u64_be(&mut self) -> Res<u64> {
        pdebug!(self, "        parsing u64 (be)");
        self.check(8, parse_error!(|| expected "u64"))?;
        let res = u64::from_be_bytes(unsafe {
            [
                self.get_unchecked(self.cursor),
                self.get_unchecked(self.cursor + 1),
                self.get_unchecked(self.cursor + 2),
                self.get_unchecked(self.cursor + 3),
                self.get_unchecked(self.cursor + 4),
                self.get_unchecked(self.cursor + 5),
                self.get_unchecked(self.cursor + 6),
                self.get_unchecked(self.cursor + 7),
            ]
        });
        self.cursor += 8;
        Ok(res)
    }

    /// Parses a `u64`, low-endian version.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use ctf::parse::RawParser;
    /// let data = 7_501_701_213u64.to_le_bytes();
    /// let mut parser = RawParser::new(&data, 0);
    /// # println!("state: {}", parser.state());
    /// assert_eq!(parser.u64_le().unwrap(), 7_501_701_213);
    /// assert!(parser.is_eof());
    /// ```
    pub fn u64_le(&mut self) -> Res<u64> {
        pdebug!(self, "        parsing u64 (le)");
        self.check(8, parse_error!(|| expected "u64"))?;
        let res = u64::from_le_bytes(unsafe {
            [
                self.get_unchecked(self.cursor),
                self.get_unchecked(self.cursor + 1),
                self.get_unchecked(self.cursor + 2),
                self.get_unchecked(self.cursor + 3),
                self.get_unchecked(self.cursor + 4),
                self.get_unchecked(self.cursor + 5),
                self.get_unchecked(self.cursor + 6),
                self.get_unchecked(self.cursor + 7),
            ]
        });
        self.cursor += 8;
        Ok(res)
    }

    /// Parses a `u64`, big-endian version.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use ctf::parse::RawParser;
    /// let data = 7_501_701.745f64.to_be_bytes();
    /// let mut parser = RawParser::new(&data, 0);
    /// # println!("state: {}", parser.state());
    /// assert_eq!(parser.f64_be().unwrap(), 7_501_701.745);
    /// assert!(parser.is_eof());
    /// ```
    pub fn f64_be(&mut self) -> Res<f64> {
        pdebug!(self, "        parsing f64 (be)");
        self.check(8, parse_error!(|| expected "f64"))?;
        let res = f64::from_be_bytes(unsafe {
            [
                self.get_unchecked(self.cursor),
                self.get_unchecked(self.cursor + 1),
                self.get_unchecked(self.cursor + 2),
                self.get_unchecked(self.cursor + 3),
                self.get_unchecked(self.cursor + 4),
                self.get_unchecked(self.cursor + 5),
                self.get_unchecked(self.cursor + 6),
                self.get_unchecked(self.cursor + 7),
            ]
        });
        self.cursor += 8;
        Ok(res)
    }

    /// Parses a `u64`, big-endian version.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use ctf::parse::RawParser;
    /// let data = 7_501_701.745f64.to_le_bytes();
    /// let mut parser = RawParser::new(&data, 0);
    /// # println!("state: {}", parser.state());
    /// assert_eq!(parser.f64_le().unwrap(), 7_501_701.745);
    /// assert!(parser.is_eof());
    /// ```
    pub fn f64_le(&mut self) -> Res<f64> {
        pdebug!(self, "        parsing f64 (le)");
        self.check(8, parse_error!(|| expected "f64"))?;
        let res = f64::from_le_bytes(unsafe {
            [
                self.get_unchecked(self.cursor),
                self.get_unchecked(self.cursor + 1),
                self.get_unchecked(self.cursor + 2),
                self.get_unchecked(self.cursor + 3),
                self.get_unchecked(self.cursor + 4),
                self.get_unchecked(self.cursor + 5),
                self.get_unchecked(self.cursor + 6),
                self.get_unchecked(self.cursor + 7),
            ]
        });
        self.cursor += 8;
        Ok(res)
    }

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

    /// Attemps to parse the memtrace CTF magic number.
    ///
    /// Parsing the magic number reveals whether the dump uses a big- or low-endian convention.
    /// Thus, it returns either a big-endian parser or a low-endian one.
    pub fn try_magic(mut self) -> Res<Either<BeParser<'data>, LeParser<'data>>> {
        pinfo!(self, "parsing magic number");
        let start = self.pos();
        let magic = self.u32_be()?;
        if magic == MAGIC {
            Ok(Either::Left(BeParser::from_raw(self)))
        } else {
            self.backtrack(start);
            let magic = self.u32_le()?;
            if magic == MAGIC {
                Ok(Either::Right(LeParser::from_raw(self)))
            } else {
                bail!(parse_error!(expected format!("magic number {}", MAGIC), found magic))
            }
        }
    }
}

/// Type representing big-endian parsing.
#[derive(Debug, Clone, Copy)]
pub struct BigEndian;
/// Type representing low-endian parsing.
#[derive(Debug, Clone, Copy)]
pub struct LowEndian;

/// Big-endian parser.
pub type BeParser<'data> = Parser<'data, BigEndian>;
/// Low-endian parser.
pub type LeParser<'data> = Parser<'data, LowEndian>;

/// A parser, parameterized by an endian specification (big or low).
pub struct Parser<'data, Endian> {
    parser: RawParser<'data>,
    _phantom: std::marker::PhantomData<Endian>,
}

impl<'data, Endian> std::ops::Deref for Parser<'data, Endian> {
    type Target = RawParser<'data>;
    fn deref(&self) -> &Self::Target {
        &self.parser
    }
}
impl<'data, Endian> std::ops::DerefMut for Parser<'data, Endian> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.parser
    }
}

impl<'data, Endian> Parser<'data, Endian> {
    /// Constructor.
    pub fn new(input: &'data [u8], offset: usize) -> Self {
        Self {
            parser: RawParser::new(input, offset),
            _phantom: std::marker::PhantomData,
        }
    }
    /// Constructor for a raw parser.
    fn from_raw(parser: RawParser<'data>) -> Self {
        Self {
            parser,
            _phantom: std::marker::PhantomData,
        }
    }
}

macro_rules! decl_impl_trait {
    (
        $(#[$trait_meta:meta])*
        pub trait $trait_id:ident<$data_lt:lifetime> {
            impl {
                $(
                    $(#[$fn_meta:meta])*
                    fn $fn_id:ident( $($fn_args:tt)* ) $(-> $fn_ty:ty)? {
                        be: $fn_be_def:expr,
                        le: $fn_le_def:expr $(,)?
                    }
                )*
            }

            $($concrete_funs:tt)*
        }
    ) => {
        $(#[$trait_meta])*
        pub trait $trait_id<$data_lt>
            : Sized
            + std::ops::Deref<Target = RawParser<'data>>
            + std::ops::DerefMut
        {
            /// Type describing the endian convention: big or low.
            type Endian;

            $(
                $(#[$fn_meta])*
                fn $fn_id( $($fn_args)* ) $(-> $fn_ty)?;
            )*

            $($concrete_funs)*
        }

        impl<$data_lt> $trait_id<$data_lt> for BeParser<$data_lt> {
            type Endian = BigEndian;
            $(
                fn $fn_id( $($fn_args)* ) $(-> $fn_ty)? {
                    $fn_be_def
                }
            )*
        }
        impl<$data_lt> $trait_id<$data_lt> for LeParser<$data_lt> {
            type Endian = LowEndian;
            $(
                fn $fn_id( $($fn_args)* ) $(-> $fn_ty)? {
                    $fn_le_def
                }
            )*
        }
    }
}

decl_impl_trait! {
    /// Trait implemented by the big-endian parser and the low-endian parser.
    pub trait CanParse<'data> {
        impl {
            /// True if the parser is big-endian.
            fn is_big_endian(&self) -> bool {
                be: true,
                le: false,
            }

            /// Parses a `u8`.
            fn u8(&mut self) -> Res<u8> {
                be: self.parser.u8_be(),
                le: self.parser.u8_le(),
            }

            /// Parses a `u16`.
            fn u16(&mut self) -> Res<u16> {
                be: self.parser.u16_be(),
                le: self.parser.u16_le(),
            }

            /// Parses a `u32`.
            fn u32(&mut self) -> Res<u32> {
                be: self.parser.u32_be(),
                le: self.parser.u32_le(),
            }

            /// Parses a `u64`.
            fn u64(&mut self) -> Res<u64> {
                be: self.parser.u64_be(),
                le: self.parser.u64_le(),
            }

            /// Parses an `f64`.
            fn f64(&mut self) -> Res<f64> {
                be: self.f64_be(),
                le: self.f64_le(),
            }
        }

        /// Parses a clock value.
        fn clock(&mut self) -> Res<u64> {
            pdebug!(self, "        parsing clock");
            self.u64()
                .map_err(|_| parse_error!(expected "clock value").into())
        }

        /// Parses a `usize` in memtrace's variable-length format.
        fn v_usize(&mut self) -> Res<usize> {
            pdebug!(self, "    parsing v_usize");
            let variant: u8 = self
                .u8()
                .chain_err(parse_error!(|| expected "variable-length usize"))?;

            let res = match variant {
                0..=252 => convert(variant, "v_usize: u8"),
                253 => convert(self.u16()?, "v_usize: u16"),
                254 => convert(self.u32()?, "v_usize: u32"),
                255 => convert(self.u64()?, "v_usize: u64"),
            };

            Ok(res)
        }

        /// Parses an allocation.
        ///
        /// Context-sensitive.
        fn alloc(
            &mut self, timestamp: u64, cxt: &mut Cxt<'data>, short: Option<usize>
        ) -> Res<ast::event::Alloc> {
            use ast::event::AllocSource;

            pinfo!(self, "parsing alloc");
            let alloc_id = cxt.next_alloc_id();
            let (is_short, len, nsamples, source) = if let Some(len) = short {
                (true, len, 1, AllocSource::Minor)
            } else {
                let len = self.v_usize()?;
                let nsample = self.v_usize()?;
                let source = match self.u8()? {
                    0 => AllocSource::Minor,
                    1 => AllocSource::Major,
                    2 => AllocSource::External,
                    n => bail!(parse_error!(
                        expected format!("boolean as a 0- or 1-valued u8, found {}", n)
                    )),
                };
                (false, len, nsample, source)
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

            let backtrace =
                cxt.btrace
                    .get_backtrace(self, nencoded, common_pref_len)?;

            let alloc_time = time::Duration::from_micros(timestamp);

            Ok(ast::event::Alloc {
                id: alloc_id,
                alloc_time,
                len,
                nsamples,
                source,
                common_pref_len,
                backtrace,
            })
        }

        /// Parses an allocation UID from a delta *w.r.t.* the most recent UID generated.
        ///
        /// Context-sensitive.
        ///
        /// Used when retrieving the UID of a promotion/collection.
        ///
        /// # Panics
        ///
        /// Calling this function before any allocation UID is generated is a logical error. As long
        /// as this function is used only for promotion/collection, it will not panic on coherent
        /// CTF files. This is because promoting/collecting necessarily talks about an allocation
        /// that was created previously, meaning at least one allocation UID was generated.
        ///
        /// > In debug, the code actually `debug_assert`s this. In release, the panic will be an
        /// > arithmetic underflow.
        fn alloc_uid_from_delta(&mut self, cxt: &Cxt<'data>) -> Res<u64> {
            let next_alloc_id = cxt.peek_next_alloc_id();
            debug_assert!(next_alloc_id > 0);
            let id_delta = self.v_usize()? as u64;
            Ok(next_alloc_id - 1 - id_delta)
        }

        /// Parses some new locations.
        ///
        /// Context-sensitive.
        fn locs(&mut self, cxt: &mut Cxt<'data>) -> Res<ast::Locs<'data>> {
            pinfo!(self, "    parsing locations");
            let id = convert(self.u64()?, "locs: id");
            let len = convert(self.u8()?, "locs: len");
            pinfo!(self, "    -> parsing {} location(s)", len);
            let mut locs = Vec::with_capacity(len);
            for _ in 0..len {
                let loc = loc::Location::parse(self, &mut cxt.loc)?;
                locs.push(loc)
            }
            Ok(ast::Locs { id, locs })
        }

        /// Parses cache-checking information.
        #[inline]
        fn cache_check(&mut self) -> Res<CacheCheck> {
            let ix = self.u16()?;
            let pred = self.u16()?;
            let value = self.u64()?;
            Ok(CacheCheck { ix, pred, value })
        }

        /// Parses an event header.
        #[inline]
        fn event_kind(&mut self, header: &header::Header) -> Res<(event::Kind, DeltaClock)> {
            pinfo!(self, "    parsing event kind");
            const EVENT_HEADER_TIME_MASK: u32 = 0x1ffffff;
            const EVENT_HEADER_TIME_MASK_U64: u64 = EVENT_HEADER_TIME_MASK as u64;
            const EVENT_HEADER_TIME_LEN: u32 = 25;

            let code = self.u32()?;
            pinfo!(self, "code: {}", code);
            pinfo!(self, "timestamp lbound: {}", header.timestamp.lbound);
            // This conversion will pretty much always be OOB.
            let timestamp_begin = header.timestamp.lbound as u32;
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

            let time = (header.timestamp.lbound & (!EVENT_HEADER_TIME_MASK_U64)) + time_low;
            // if !header.timestamp.contains(time) {
            //     bail!(
            //         "inconsistent event header time, expected `{} <= {} <= {}`",
            //         header.timestamp.lbound,
            //         time,
            //         header.timestamp.end
            //     )
            // }
            let ev_code = code >> EVENT_HEADER_TIME_LEN;
            pinfo!(self, "ev code: {}, time: {}", ev_code, time);
            let ev = event::Kind::from_code(ev_code)?;
            Ok((ev, time))
        }

        /// Parses a packet header.
        ///
        /// A packet is a sequence of events.
        fn packet_header(&mut self, id: usize) -> Res<header::Packet> {
            pinfo!(self, "parsing packet header");
            self.raw_package_header(true)
                .map(|(header, cache_check)| header::Packet::new(id, header, cache_check))
                .chain_err(|| "while parsing packet header")
        }

        /// Parses the top-level CTF header.
        ///
        /// The CTF header is the first element of a CTF file, followed by the trace info, and then
        /// the sequence of packets.
        fn ctf_header(&mut self) -> Res<header::Ctf> {
            pinfo!(self, "parsing ctf header");
            self.raw_package_header(false)
                .map(|(header, _)| header::Ctf::new(header, self.big_endian))
                .chain_err(|| "while parsing ctf header")
        }

        /// Raw package header parser.
        ///
        /// Returns internal errors as they are, should only be called by [`ctf_header`] and
        /// [`packet_header`] which enrich these errors.
        ///
        /// [`ctf_header`]: CanParse::ctf_header
        /// [`packet_header`]: CanParse::packet_header
        fn raw_package_header(&mut self, parse_magic: bool) -> Res<(header::Header, CacheCheck)> {
            let start = self.pos();

            if parse_magic {
                self.magic()?;
            }

            let packet_size_bits = self.u32()?;
            pinfo!(self, "    package size bits {}", packet_size_bits);

            let (begin, end) = (self.clock()?, self.clock()?);
            pinfo!(self, "    begin/end times {}/{}", begin, end);
            let timestamp = Range::new(begin, end);

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
            let alloc_id = Range::new(alloc_begin, alloc_end);

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

        /// Parses a trace info.
        ///
        /// Technically, a trace info is a normal event, meaning it could appear in a normal packet.
        /// However, **currently** the trace info needs to be unique and appear between the CTF
        /// (top-level) header and the first package of the trace.
        fn trace_info(&mut self, header: &header::Ctf) -> Res<event::Info<'data>> {
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
                Some(self.string()?)
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

        /// Parses the memtrace CTF magic number.
        fn magic(&mut self) -> Res<()> {
            let magic = self.u32()?;
            if magic == MAGIC {
                Ok(())
            } else {
                bail!(parse_error!(expected format!("magic number {}", MAGIC), found magic))
            }
        }
    }
}

/// Top-level parser.
pub struct CtfParser<'data, Endian> {
    parser: Parser<'data, Endian>,
    header: header::Ctf,
    trace_info: ast::event::Info<'data>,
    cxt: Cxt<'data>,
    packet_count: usize,
}
impl<'data> CtfParser<'data, ()> {
    /// Constructor.
    ///
    /// Yields either a big-endian or a low-endian parser, based on the magic-number starting the
    /// sequence of bytes. This function is not meant to be used directly, use the [`parse` macro]
    /// instead, which hides the details of handling the `Either` part.
    ///
    /// [`parse` macro]: parse! (parse macro)
    pub fn new(bytes: &'data [u8]) -> Res<Either<BeCtfParser<'data>, LeCtfParser<'data>>> {
        let parser = RawParser::new(bytes, 0);
        let parser_disj = parser.try_magic()?;

        let res = parser_do! {
            parser_disj => map |mut parser| {
                let header = parser.ctf_header()?;
                let (event_kind, _event_time) = parser.event_kind(&header)?;
                if !event_kind.is_info() {
                    bail!(
                        "expected initial event to be an info event, found {:?}",
                        event_kind
                    )
                }
                let trace_info = parser.trace_info(&header)?;
                CtfParser {
                    parser, header, trace_info, cxt: Cxt::new(), packet_count: 0,
                }
            }
        };

        Ok(res)
    }
}

impl<'data, Endian> std::ops::Deref for CtfParser<'data, Endian> {
    type Target = Parser<'data, Endian>;
    fn deref(&self) -> &Self::Target {
        &self.parser
    }
}
impl<'data, Endian> std::ops::DerefMut for CtfParser<'data, Endian> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.parser
    }
}

/// Low-endian CTF parser.
pub type LeCtfParser<'data> = CtfParser<'data, LowEndian>;
/// Big-endian CTF parser.
pub type BeCtfParser<'data> = CtfParser<'data, BigEndian>;

impl<'data, Endian> CtfParser<'data, Endian> {
    /// Header accessor.
    pub fn header(&self) -> &header::Ctf {
        &self.header
    }
    /// Trace info accessor.
    pub fn trace_info(&self) -> &ast::event::Info<'data> {
        &self.trace_info
    }
}

/// Pseudo-parsers: parses a very tiny amout of data to produce a subparser.
impl<'data, Endian> CtfParser<'data, Endian>
where
    Parser<'data, Endian>: CanParse<'data>,
{
    /// Yields a [`PacketParser`] for the next packet, if any.
    pub fn next_packet<'me>(&'me mut self) -> Res<Option<PacketParser<'me, 'data, Endian>>> {
        let parser = &mut self.parser;
        let cxt = &mut self.cxt;
        let packet_count = &mut self.packet_count;

        if parser.is_eof() {
            cxt.prof.all_do(
                || log::info!("done parsing packets"),
                |desc, sw| log::info!("| {:>13}: {}", desc, sw),
            );
            return Ok(None);
        }
        pinfo!(parser, "parsing packet header");

        let packet_header = parser.packet_header(*packet_count)?;
        let content_len: usize = convert(packet_header.content_size, "next_packet: content_len");
        pinfo!(
            parser,
            "next packet: {} bytes -> {}/{}",
            content_len,
            *parser.pos() + content_len,
            parser.data().len()
        );
        if *parser.pos() + content_len > parser.data().len() {
            bail!(parse_error!(expected format!(
                "legal packet size: not enough data left ({}/{})",
                content_len, parser.data().len() - *parser.pos(),
            )))
        }

        let event_bytes = parser.take(content_len);
        let next = PacketParser::<Endian>::new(event_bytes, *parser.pos(), packet_header, cxt);
        *packet_count += 1;

        Ok(Some(next))
    }
}

/// Packet parser.
///
/// Thin wrapper around a [`RawParser`] over the bytes of the events of the packet. Also stores the
/// packet header. Note that the bytes for the header are not included in the parser's data. It has
/// already been parsed.
pub struct PacketParser<'cxt, 'data, Endian> {
    /// Internal parser over the bytes of the events of the packet.
    ///
    /// Does **not** contain the packet header's bytes.
    parser: Parser<'data, Endian>,
    /// Packet header.
    header: header::Packet,
    /// Event counter.
    event_cnt: usize,
    /// Parsing context.
    cxt: &'cxt mut Cxt<'data>,
}

impl<'cxt, 'data, Endian> std::ops::Deref for PacketParser<'cxt, 'data, Endian> {
    type Target = Parser<'data, Endian>;
    fn deref(&self) -> &Parser<'data, Endian> {
        &self.parser
    }
}
impl<'cxt, 'data, Endian> std::ops::DerefMut for PacketParser<'cxt, 'data, Endian> {
    fn deref_mut(&mut self) -> &mut Parser<'data, Endian> {
        &mut self.parser
    }
}

/// Low-endian packet parser.
pub type LePacketParser<'cxt, 'data> = PacketParser<'cxt, 'data, LowEndian>;
/// Big-endian packet parser.
pub type BePacketParser<'cxt, 'data> = PacketParser<'cxt, 'data, BigEndian>;

impl<'cxt, 'data, Endian> PacketParser<'cxt, 'data, Endian>
where
    Parser<'data, Endian>: CanParse<'data>,
{
    /// Constructor.
    ///
    /// - `input`: should contain the bytes of all the events in the packet, and nothing more (in
    ///   particular *not* the packet's header);
    /// - `offset`: offset from the start of the original input, for error-reporting;
    /// - `header`: packet header, must be parsed beforehand,
    /// - `cxt`: parsing context, borrowed from the [`CtfParser`].
    fn new(
        input: &'data [u8],
        offset: usize,
        header: header::Packet,
        cxt: &'cxt mut Cxt<'data>,
    ) -> Self {
        Self {
            parser: Parser::new(input, offset),
            header,
            event_cnt: 0,
            cxt,
        }
    }

    /// Header accessor.
    pub fn header(&self) -> &header::Packet {
        &self.header
    }

    /// Returns the next event of the packet, if any.
    pub fn next_event(&mut self) -> Res<Option<(Clock, Event<'data>)>> {
        if self.is_eof() {
            return Ok(None);
        }

        self.cxt.prof.basic_parsing.start();
        let (event_kind, event_timestamp) = self.parser.event_kind(&self.header)?;
        self.cxt.prof.basic_parsing.stop();

        pinfo!(self, "event: {:?} ({})", event_kind, event_timestamp);

        let parser = &mut self.parser;
        let cxt = &mut self.cxt;

        let event = match event_kind {
            event::Kind::Alloc => {
                cxt.prof.alloc.start();
                let alloc = parser.alloc(event_timestamp, cxt, None)?;
                cxt.prof.alloc.stop();
                Event::Alloc(alloc)
            }
            event::Kind::SmallAlloc(n) => {
                cxt.prof.alloc.start();
                let alloc = parser.alloc(
                    event_timestamp,
                    cxt,
                    Some(convert(n, "event: SmallAlloc(n)")),
                )?;
                cxt.prof.alloc.stop();
                Event::Alloc(alloc)
            }
            event::Kind::Promotion => {
                let alloc_id = parser.alloc_uid_from_delta(cxt)?;
                Event::Promotion(alloc_id)
            }
            event::Kind::Collection => {
                cxt.prof.collection.start();
                let alloc_id = parser.alloc_uid_from_delta(cxt)?;
                cxt.prof.collection.stop();
                Event::Collection(alloc_id)
            }
            event::Kind::Locs => {
                cxt.prof.locs.start();
                let locs = parser.locs(cxt)?;
                cxt.prof.locs.stop();
                Event::Locs(locs)
            }

            // Can't have more than two info events.
            event::Kind::Info => bail!(
                parse_error!(expected "non-info event: having more than two info events is illegal")
            ),
        };

        pinfo!(parser, "    {:?}", event);

        self.event_cnt += 1;

        Ok(Some((event_timestamp, event)))
    }
}
