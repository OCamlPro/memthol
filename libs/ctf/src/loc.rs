//! Location parsing, using an MTF table.

prelude! {}

pub type Data<'data, T> = (&'data str, T);
pub type Entry<'data, T> = Option<Data<'data, T>>;

// const FIRST_IDX: Idx = Idx { idx: 0 };
const LAST_IDX: u8 = 30;
const MAX_IDX: u8 = LAST_IDX + 1;

#[derive(Debug, Clone, Copy)]
pub struct Idx {
    idx: u8,
}
impl Idx {
    fn new(idx: u8) -> Self {
        Self { idx }
    }
    // fn not_found() -> Self {
    //     Self { idx: MAX_IDX }
    // }

    pub fn is_not_found(self) -> bool {
        self.idx > LAST_IDX
    }
}
impl fmt::Display for Idx {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        self.idx.fmt(fmt)
    }
}

#[derive(Debug, Clone)]
struct MtfMap<'data, T> {
    vec: Vec<Entry<'data, T>>,
}
impl<'data, T> MtfMap<'data, T> {
    pub fn new() -> Self
    where
        T: Clone,
    {
        Self {
            vec: vec![None; MAX_IDX as usize],
        }
    }

    pub fn remove_last(&mut self) -> Entry<'data, T> {
        if let Some(last) = self.vec.last_mut() {
            std::mem::replace(last, None)
        } else {
            None
        }
    }

    #[cfg(not(debug_assertions))]
    #[inline]
    fn check(&self, _: &'static str) -> Res<()> {
        Ok(())
    }

    #[cfg(debug_assertions)]
    fn check(&self, from: &'static str) -> Res<()> {
        let mut none_count = 0;
        for (idx, entry) in self.vec.iter().enumerate() {
            if none_count > 0 {
                if entry.is_some() {
                    bail!(
                        "[fatal] during {}: mtf map has an entry at {}, \
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
    }

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

    // pub fn last(&self) -> Option<&(&'data str, T)> {
    //     self.vec[LAST_IDX as usize].as_ref()
    // }
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

pub struct Cxt<'data> {
    map: MtfMap<'data, MtfMap<'data, ()>>,
}
impl<'data> Cxt<'data> {
    pub fn new() -> Self {
        Self { map: MtfMap::new() }
    }

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

pub type Locs<'data> = Vec<Location<'data>>;

#[derive(Debug, Clone)]
pub struct Location<'data> {
    pub encoded: usize,
    pub file_path: &'data str,
    /// Line index (from zero).
    pub line: usize,
    /// Column span (from zero).
    pub col: Span<usize>,
    pub def_name: &'data str,
}
impl<'data> Location<'data> {
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
        let col = Span::new(start_char, end_char)?;
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
            self.def_name, self.file_path, self.line, self.col.begin, self.col.end
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
