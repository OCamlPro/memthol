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

//! Location parsing, using an MTF table.

prelude! {}

/// A string slice and some value.
pub type Data<'data, T> = (&'data str, T);
/// An optional [`Data`] value.
///
/// [`Data`]: type.Data.html (Data type alias)
pub type Entry<'data, T> = Option<Data<'data, T>>;

/// Last legal index in the MTF table.
const LAST_IDX: u8 = 30;
/// First non-legal index in the MTF table.
const MAX_IDX: u8 = LAST_IDX + 1;

/// An index in the MTF table.
///
/// Wrapper around a `u8`.
#[derive(Debug, Clone, Copy)]
pub struct Idx {
    /// Actual index.
    ///
    /// Considered unknown if `idx > LAST_IDX`.
    idx: u8,
}
impl Idx {
    /// Constructor.
    fn new(idx: u8) -> Self {
        Self { idx }
    }

    /// True if the index is outside of the legal index range.
    pub fn is_not_found(self) -> bool {
        self.idx > LAST_IDX
    }
}
impl fmt::Display for Idx {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        self.idx.fmt(fmt)
    }
}

/// MTF (Move-To-Front) map.
#[derive(Debug, Clone)]
pub struct MtfMap<'data, T> {
    /// Actual MTF map.
    ///
    /// **Always has length `MAX_IDX`.**
    vec: Vec<Entry<'data, T>>,
}

impl<'data, T> MtfMap<'data, T> {
    /// Creates an empty MTF map.
    pub fn new() -> Self
    where
        T: Clone,
    {
        Self {
            vec: vec![None; MAX_IDX as usize],
        }
    }

    /// Removes the last entry in the MTF map.
    pub fn remove_last(&mut self) -> Entry<'data, T> {
        if let Some(last) = self.vec.last_mut() {
            std::mem::replace(last, None)
        } else {
            None
        }
    }

    base::cfg_item! {
        pref {
            /// Checks the MTF map's internal invariants.
            #[inline]
            fn check(&self, _from: &'static str) -> Res<()>
        }
        cfg(debug) {{
            let from = _from;
            if self.vec.len() != (LAST_IDX + 1) as usize {
                bail!(
                    "MTF table len is {}, expected {}",
                    self.vec.len(),
                    LAST_IDX + 1
                )
            }
            let mut none_count = 0;
            for (idx, entry) in self.vec.iter().enumerate() {
                if none_count > 0 {
                    if entry.is_some() {
                        bail!(
                            "during {}: mtf map has an entry at {}, \
                            but it is preceeded by {} none-bindings",
                            from,
                            idx,
                            none_count,
                        )
                    } else {
                        none_count += 1
                    }
                } else if entry.is_none() {
                    none_count += 1
                }
            }
            Ok(())
        }} else {{
            Ok(())
        }}
    }

    /// String representation of the MTF table.
    #[cfg(test)]
    pub fn to_string(&self) -> String
    where
        T: fmt::Display,
    {
        let mut res = "{".to_string();
        let mut end = "";
        for (idx, entry) in self.vec.iter().enumerate() {
            let (s, val) = match entry {
                None => continue,
                Some(pair) => pair,
            };
            end = "\n";
            res.push_str(&format!("\n    {: >2} -> {}, `{}`", idx, val, s))
        }
        res.push_str(end);
        res.push('}');
        res
    }

    /// Moves the element at `idx` to the front of the MTF.
    ///
    /// Slides all elements before `idx` to the right.
    fn move_to_front(&mut self, idx: u8) -> Res<()> {
        self.check("before move_to_front")?;
        let idx = idx as usize;
        if self.vec[idx].is_none() {
            bail!(
                "[fatal] trying to swap at index {}, but that index has no entry",
                idx
            )
        }
        for (idx_1, idx_2) in (0..idx).into_iter().map(|idx| (idx, idx + 1)).rev() {
            self.vec.swap(idx_1, idx_2);
        }
        self.check("after move_to_front")
    }

    /// Pushes an element at the front of the MTF map.
    ///
    /// Slides all elements in the map to the right.
    fn push(&mut self, key: &'data str, val: T) -> Res<()> {
        self.check("before pushing")?;
        let mut tmp = Some((key, val));
        for entry in &mut self.vec {
            std::mem::swap(&mut tmp, entry);
            if tmp.is_none() {
                break;
            }
        }
        self.check("after pushing")
    }

    /// Decodes a location at the current position in the input parser.
    pub fn decode<Out, Parser>(
        &mut self,
        parser: &mut Parser,
        idx: Idx,
        if_absent: impl FnOnce(&mut Parser, Entry<'data, T>) -> Res<(&'data str, T)>,
        binding_do: impl FnOnce(&mut Parser, &'data str, &mut T) -> Res<Out>,
    ) -> Res<Out>
    where
        Parser: CanParse<'data>,
    {
        self.check("decode")?;
        if idx.is_not_found() {
            pinfo!(parser, "index {} is not found", idx.idx);
            let last = self.remove_last();
            let (key, mut val) = if_absent(parser, last)?;
            let res = binding_do(parser, key, &mut val);
            self.push(key, val)?;
            res
        } else {
            pinfo!(parser, "index {} is NOT not found", idx.idx);
            let res = match &mut self[idx] {
                Some((key, val)) => binding_do(parser, *key, val),
                None => bail!("[fatal] trying to decode an empty entry at {}", idx),
            };
            self.move_to_front(idx.idx)?;
            res
        }
    }
}

impl<'data, T> std::ops::Index<Idx> for MtfMap<'data, T> {
    type Output = Entry<'data, T>;
    fn index(&self, idx: Idx) -> &Entry<'data, T> {
        &self.vec[idx.idx as usize]
    }
}
impl<'data, T> std::ops::IndexMut<Idx> for MtfMap<'data, T> {
    fn index_mut(&mut self, idx: Idx) -> &mut Entry<'data, T> {
        &mut self.vec[idx.idx as usize]
    }
}

/// Location parsing context.
///
/// Wrapper around an MTF map.
pub struct Cxt<'data> {
    /// The MTF map.
    map: MtfMap<'data, MtfMap<'data, ()>>,
}
impl<'data> Cxt<'data> {
    /// Constructs an empty context.
    pub fn new() -> Self {
        Self { map: MtfMap::new() }
    }

    /// Multi-line string representation of the context.
    pub fn to_ml_string(&self) -> String {
        let mut s = format!("{{");
        let mut count = 0;

        for (idx, entry) in self.map.vec.iter().enumerate() {
            if let Some((key, val)) = entry {
                s.push_str(&format!("\n    {: >2} -> {} {{", idx, key));
                let mut sub_count = 0;

                for (idx, entry) in val.vec.iter().enumerate() {
                    if let Some((key, ())) = entry {
                        s.push_str(&format!("\n        {: >2} -> {},", idx, key));
                        sub_count += 1;
                    } else {
                        break;
                    }
                }

                if sub_count > 0 {
                    s.push('\n')
                }
                s.push_str("    },");
                count += 1
            } else {
                break;
            }
        }

        if count > 0 {
            s.push('\n')
        }
        s.push('}');
        s
    }
}

/// A list of locations.
pub type Locs<'data> = Vec<Location<'data>>;

/// A location.
#[derive(Debug, Clone)]
pub struct Location<'data> {
    /// Encoded binary version of the location.
    pub encoded: usize,
    /// Path to the allocation-site file.
    pub file_path: &'data str,
    /// Line index (from zero).
    pub line: usize,
    /// Column span (from zero).
    pub col: Range<usize>,
    /// Definition name.
    ///
    /// Currently unused in memthol proper.
    pub def_name: &'data str,
}
impl<'data> Location<'data> {
    /// Parses a location at the current position in the input parser.
    pub fn parse(parser: &mut impl CanParse<'data>, cxt: &mut Cxt<'data>) -> Res<Self> {
        let low: u64 = convert(parser.u32()?, "loc: low");
        let high: u64 = convert(parser.u16()?, "loc: high");
        pinfo!(parser, "    loc {{ low: {}, high: {} }}", low, high);

        // Mimicing the caml code:
        //
        // ```ocaml
        // let encoded = Int64.(
        //     logor (shift_left (of_int high) 32)
        //     (logand (of_int32 low) 0xffffffffL)
        // );
        // ```
        let encoded = (high << 32) | (low & 0xffffffffu64);

        // Now we're doing this:
        //
        // ```ocaml
        // let line, start_char, end_char, filename_code, defname_code = Int64.(
        //     to_int (logand 0xfffffL encoded),
        //     to_int (logand 0xffL (shift_right encoded 20)),
        //     to_int (logand 0x3ffL (shift_right encoded (20 + 8))),
        //     to_int (logand 0x1fL (shift_right encoded (20 + 8 + 10))),
        //     to_int (logand 0x1fL (shift_right encoded (20 + 8 + 10 + 5))))
        // );
        // ```
        let line = convert(0xfffffu64 & encoded, "loc: line");
        let start_char = convert(0xffu64 & (encoded >> 20), "loc: start_char");
        let end_char = convert(0x3ffu64 & (encoded >> 20 + 8), "loc: end_char");
        let col = Range::new(start_char, end_char);
        let file_path_code = convert(0x1fu64 & (encoded >> (20 + 8 + 10)), "loc: file_path_code");
        let def_name_code = convert(
            0x1fu64 & (encoded >> (20 + 8 + 10 + 5)),
            "loc: def_name_code",
        );

        pinfo!(
            parser,
            "    file_path_code: {}, def_name_code: {}",
            file_path_code,
            def_name_code
        );

        let idx = Idx::new(file_path_code);

        let (file_path, def_name) = {
            cxt.map
                .decode(
                    parser,
                    idx,
                    // if absent, parse a string and bind it to a new map
                    |parser, last| {
                        let map = if let Some((_, map)) = last {
                            map
                        } else {
                            MtfMap::new()
                        };
                        pinfo!(parser, "        parsing file path");
                        Ok((parser.string()?, map))
                    },
                    // given the parser and the file path/map binding, do this
                    |parser, file_path, map| {
                        map.decode(
                            parser,
                            Idx::new(def_name_code),
                            // if absent, parse a string and bind it to unit
                            |parser, _| {
                                pinfo!(parser, "        parsing def name");
                                Ok((parser.string()?, ()))
                            },
                            // given the parser and the def name/unit binding, return file path and def
                            // name
                            |_, def_name, _| Ok((file_path, def_name)),
                        )
                        .chain_err(|| {
                            format!(
                                "while working on sub-MTF-map at {} ({})",
                                idx,
                                idx.is_not_found(),
                            )
                        })
                    },
                )
                .chain_err(|| format!("location context {}", cxt.to_ml_string()))?
        };

        Ok(Location {
            encoded: convert(encoded, "ctf location parser: encoded"),
            line,
            col,
            file_path,
            def_name,
        })
    }
}

impl fmt::Display for Location<'_> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(
            fmt,
            "{}@{}:{}:{}-{}",
            self.def_name, self.file_path, self.line, self.col.lbound, self.col.ubound
        )
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn check<T>(mtf: &MtfMap<T>, expected: &'static str)
    where
        T: fmt::Display,
    {
        let s = mtf.to_string();
        println!("mtf map: {}", s);
        if s != expected {
            println!("\n\n|===| Error:");
            println!("got mtf map {}", s);
            println!("expected {}", expected);
            panic!("mtf map check failed")
        }
    }

    #[test]
    fn move_to_front() {
        let data = vec!["0", "1", "2", "3", "4"];
        let mut mtf = MtfMap::new();
        for idx in 0..data.len() {
            mtf.vec[idx] = Some((data[idx], '🙀'));
        }
        check(
            &mtf,
            "\
{
     0 -> 🙀, `0`
     1 -> 🙀, `1`
     2 -> 🙀, `2`
     3 -> 🙀, `3`
     4 -> 🙀, `4`
}\
            ",
        );

        mtf.move_to_front(0).unwrap();
        check(
            &mtf,
            "\
{
     0 -> 🙀, `0`
     1 -> 🙀, `1`
     2 -> 🙀, `2`
     3 -> 🙀, `3`
     4 -> 🙀, `4`
}\
            ",
        );

        mtf.move_to_front(1).unwrap();
        check(
            &mtf,
            "\
{
     0 -> 🙀, `1`
     1 -> 🙀, `0`
     2 -> 🙀, `2`
     3 -> 🙀, `3`
     4 -> 🙀, `4`
}\
            ",
        );

        mtf.move_to_front(2).unwrap();
        check(
            &mtf,
            "\
{
     0 -> 🙀, `2`
     1 -> 🙀, `1`
     2 -> 🙀, `0`
     3 -> 🙀, `3`
     4 -> 🙀, `4`
}\
            ",
        );

        mtf.move_to_front(3).unwrap();
        check(
            &mtf,
            "\
{
     0 -> 🙀, `3`
     1 -> 🙀, `2`
     2 -> 🙀, `1`
     3 -> 🙀, `0`
     4 -> 🙀, `4`
}\
            ",
        );

        mtf.move_to_front(4).unwrap();
        check(
            &mtf,
            "\
{
     0 -> 🙀, `4`
     1 -> 🙀, `3`
     2 -> 🙀, `2`
     3 -> 🙀, `1`
     4 -> 🙀, `0`
}\
            ",
        );
    }

    #[test]
    fn push() {
        let data = vec!["0", "1", "2", "3", "4"];
        let mut mtf = MtfMap::new();

        check(&mtf, "{}");

        mtf.push(data[0], '🙀').unwrap();
        check(
            &mtf,
            "\
{
     0 -> 🙀, `0`
}\
            ",
        );

        mtf.push(data[1], '🙀').unwrap();
        check(
            &mtf,
            "\
{
     0 -> 🙀, `1`
     1 -> 🙀, `0`
}\
            ",
        );

        mtf.push(data[2], '🙀').unwrap();
        check(
            &mtf,
            "\
{
     0 -> 🙀, `2`
     1 -> 🙀, `1`
     2 -> 🙀, `0`
}\
            ",
        );

        mtf.push(data[3], '🙀').unwrap();
        check(
            &mtf,
            "\
{
     0 -> 🙀, `3`
     1 -> 🙀, `2`
     2 -> 🙀, `1`
     3 -> 🙀, `0`
}\
            ",
        );

        mtf.push(data[4], '🙀').unwrap();
        check(
            &mtf,
            "\
{
     0 -> 🙀, `4`
     1 -> 🙀, `3`
     2 -> 🙀, `2`
     3 -> 🙀, `1`
     4 -> 🙀, `0`
}\
            ",
        );
    }
}
