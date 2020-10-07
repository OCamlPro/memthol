//! Location parsing, using an MTF table.

prelude! {}

pub type Data<'data, T> = (&'data str, T);
pub type Entry<'data, T> = Option<Data<'data, T>>;

const FIRST_IDX: Idx = Idx { idx: 0 };
const LAST_IDX: u8 = 30;
const MAX_IDX: u8 = LAST_IDX + 1;

#[derive(Debug, Clone, Copy)]
pub struct Idx {
    idx: u8,
}
impl Idx {
    fn new(idx: u8) -> Self {
        let res = Self { idx };
        res.check("creation");
        res
    }
    fn not_found() -> Self {
        Self { idx: MAX_IDX }
    }

    pub fn is_not_found(self) -> bool {
        true
    }

    fn check(self, from: &'static str) {
        if self.idx >= MAX_IDX {
            panic!(
                "[fatal] during {}: illegal MFT map index {} (should be < {})",
                from, self.idx, MAX_IDX,
            )
        }
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

    pub fn check(&self, from: &'static str) {
        let mut none_count = 0;
        for (idx, entry) in self.vec.iter().enumerate() {
            if none_count > 0 {
                if entry.is_some() {
                    panic!(
                        "[fatal] during {}: mtf map has an entry at {}, \
                        but it is preceeded by {} none-bindings",
                        from, idx, none_count,
                    )
                } else {
                    none_count += 1
                }
            } else if entry.is_none() {
                none_count += 1
            }
        }
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
    fn move_to_front(&mut self, idx: u8) {
        self.check("before move_to_front");
        Idx::new(idx).check("swap");
        let idx = idx as usize;
        if self.vec[idx].is_none() {
            panic!(
                "[fatal] trying to swap at index {}, but that index has no entry",
                idx
            )
        }
        for (idx_1, idx_2) in (0..idx).into_iter().map(|idx| (idx, idx + 1)).rev() {
            self.vec.swap(idx_1, idx_2);
        }
        self.check("after move_to_front");
    }

    fn push(&mut self, key: &'data str, val: T) {
        self.check("before pushing");
        let mut tmp = Some((key, val));
        for entry in &mut self.vec {
            std::mem::swap(&mut tmp, entry);
            if tmp.is_none() {
                break;
            }
        }
        self.check("after pushing");
    }

    pub fn encode(&mut self, key: &'data str, if_absent: impl FnOnce() -> T) -> Idx {
        let mut key_idx = MAX_IDX;
        for (idx, entry) in self.vec.iter().enumerate() {
            match *entry {
                None => break,
                Some((k, _)) => {
                    if k == key {
                        key_idx = idx as u8;
                        break;
                    }
                }
            }
        }

        if key_idx < MAX_IDX {
            self.move_to_front(key_idx);
            Idx::new(key_idx)
        } else {
            let val = if_absent();
            self.push(key, val);
            Idx::not_found()
        }
    }

    pub fn decode(&mut self, idx: Idx, if_absent: impl FnOnce() -> (&'data str, T)) -> Idx {
        if idx.is_not_found() {
            let (key, val) = if_absent();
            self.push(key, val);
            FIRST_IDX
        } else if self[idx].is_some() {
            self.move_to_front(idx.idx);
            FIRST_IDX
        } else {
            panic!("[fatal] trying to decode an empty entry")
        }
    }

    pub fn last(&self) -> Option<&(&'data str, T)> {
        self.vec[LAST_IDX as usize].as_ref()
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

pub struct LocationCxt<'data> {
    map: MtfMap<'data, MtfMap<'data, ()>>,
}
impl<'data> LocationCxt<'data> {
    pub fn new() -> Self {
        Self { map: MtfMap::new() }
    }
}

#[derive(Debug, Clone)]
pub struct Location<'data> {
    pub file_path: &'data str,
    /// Line index (from zero).
    pub line: usize,
    /// Column span (from zero).
    pub col: Span<usize>,
    pub def_name: &'data str,
}
impl<'data> Location<'data> {
    pub fn unknown() -> Self {
        Self {
            file_path: "unknown",
            line: 0,
            col: Span { begin: 0, end: 0 },
            def_name: "??",
        }
    }

    /// Parses a location by overwriting itself.
    fn parse(&mut self, cxt: &mut LocationCxt, parser: &mut RawParser<'data>) -> Res<()> {
        let low = parser.i32()?;
        let high = parser.i16()?;
        // Mimicing the caml code:
        //
        // ```ocaml
        // let encoded = Int64.(
        //     logor (shift_left (of_int high) 32)
        //     (logand (of_int32 low) 0xffffffffL)
        // );
        // ```
        let encoded = ((high as i64) << 32) | ((low as i64) & 0xffffffffi64);
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
        let line = 0xfffffi64 & encoded;
        let start_char = 0xffi64 & (encoded >> 20);
        let end_char = 0x3ffi64 & (encoded >> 20 + 8);
        let file_path_code = 0x1fi64 & (encoded >> (20 + 8 + 10));
        let def_name_code = 0x1fi64 & (encoded >> (20 + 8 + 10 + 5));
        todo!("parse location")
    }

    pub fn parse_list(
        cxt: &mut LocationCxt,
        parser: &mut RawParser<'data>,
    ) -> Res<Vec<Location<'data>>> {
        let id = parser.i64()?;
        let nlocs = parser.i8()?;
        let mut res = vec![Self::unknown(); nlocs as usize];
        for slf in &mut res {
            slf.parse(cxt, parser)?
        }
        Ok(res)
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

        mtf.move_to_front(0);
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

        mtf.move_to_front(1);
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

        mtf.move_to_front(2);
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

        mtf.move_to_front(3);
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

        mtf.move_to_front(4);
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

        mtf.push(data[0], '🙀');
        check(
            &mtf,
            "\
{
     0 -> 🙀, `0`
}\
            ",
        );

        mtf.push(data[1], '🙀');
        check(
            &mtf,
            "\
{
     0 -> 🙀, `1`
     1 -> 🙀, `0`
}\
            ",
        );

        mtf.push(data[2], '🙀');
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

        mtf.push(data[3], '🙀');
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

        mtf.push(data[4], '🙀');
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
