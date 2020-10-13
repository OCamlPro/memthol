prelude! {}

const CACHE_SIZE: usize = 1 << 14;

pub struct Cxt {
    cache_loc: Vec<usize>,
    cache_pred: Vec<usize>,
    last_backtrace: Vec<usize>,
}
impl Cxt {
    pub fn new() -> Self {
        Self {
            cache_loc: vec![0; CACHE_SIZE],
            cache_pred: vec![0; CACHE_SIZE],
            last_backtrace: Vec::with_capacity(16),
        }
    }

    pub fn to_ml_string(&self) -> String {
        let mut s = format!("{{\n");
        s.push_str("    cache_loc: [");
        for (idx, val) in self.cache_loc.iter().enumerate() {
            if idx > 0 {
                s.push(',')
            }
            s.push_str(&format!(" {}", val))
        }
        s.push_str(" ],\n    cache_pred: [");
        for (idx, val) in self.cache_pred.iter().enumerate() {
            if idx > 0 {
                s.push(',')
            }
            s.push_str(&format!(" {}", val))
        }
        s.push_str(" ],\n    last_backtrace: [");
        for (idx, val) in self.cache_pred.iter().enumerate() {
            if idx > 0 {
                s.push(',')
            }
            s.push_str(&format!(" {}", val))
        }
        s.push_str(" ],\n}");
        s
    }

    fn realloc(buf: &mut Vec<usize>, pos: usize, val: usize) {
        assert!(pos == buf.len());

        let new_len = if buf.len() < 16 { 32 } else { buf.len() * 2 };
        debug_assert!(new_len > buf.len());

        buf.resize(new_len, val)
    }

    fn put(buf: &mut Vec<usize>, pos: usize, val: usize) {
        if pos < buf.len() {
            let cell = unsafe { buf.get_unchecked_mut(pos) };
            *cell = val
        } else {
            Self::realloc(buf, pos, val)
        }
    }

    pub fn get_backtrace<'data>(
        &mut self,
        parser: &mut impl CanParse<'data>,
        nencoded: usize,
        common_pref_len: usize,
    ) -> Res<(SVec32<usize>, usize)> {
        assert!(common_pref_len <= self.last_backtrace.len());

        let Self {
            cache_loc,
            cache_pred,
            last_backtrace,
        } = self;

        // decode-loop data
        let mut pred = 0;
        let buf = last_backtrace;
        let mut pos = common_pref_len;
        let mut decode_current = nencoded;
        let mut predict_current;

        let res = 'decode: loop {
            if decode_current == 0 {
                break 'decode (buf.iter().cloned().collect(), pos);
            }

            let codeword = parser.u16()?;
            let bucket: usize = convert(codeword >> 2, "ctf backtrace: bucket");
            let tag = codeword & 3;

            cache_pred[pred] = bucket;
            pred = bucket;

            predict_current = match tag {
                // Cache hit, 0, 1 or N prediction(s).
                0..=2 => {
                    Self::put(buf, pos, cache_loc[bucket]);
                    pos += 1;
                    decode_current -= 1;
                    if tag == 2 {
                        let predict = parser.u8()?;
                        predict as u16
                    } else {
                        tag
                    }
                }
                // Cache miss.
                _ => {
                    let lit = convert(parser.u64()?, "get_backtrace: lit");
                    cache_loc[bucket] = lit;
                    Self::put(buf, pos, lit);
                    pos += 1;
                    decode_current -= 1;
                    continue 'decode;
                }
            };

            'predict: loop {
                if predict_current == 0 {
                    continue 'decode;
                } else {
                    pred = convert(cache_pred[pred], "get_backtrace: pred");
                    Self::put(buf, pos, cache_loc[pred]);
                    pos += 1;
                    predict_current -= 1;
                    continue 'predict;
                }
            }
        };

        Ok(res)
    }

    pub fn skip_backtrace<'data>(
        &mut self,
        parser: &mut impl CanParse<'data>,
        nencoded: i8,
        _common_pref_len: i8,
    ) -> Res<()> {
        for _ in 0..nencoded {
            let codeword = parser.u16()?;
            match codeword & 3 {
                2 => destroy(parser.u8()?),
                3 => destroy(parser.u64()?),
                _ => (),
            }
        }
        Ok(())
    }

    pub fn check_cache_verifier<'data>(&self, parser: &mut impl CanParse<'data>) -> Res<()> {
        println!("checking cache");
        let ix: usize = convert(parser.u16()?, "check_cache_verifier: ix");
        let pred = parser.u16()? as usize;
        let value = parser.u64()?;

        macro_rules! error {
            ($($blah:tt)*) => {
                bail!(
                    "ix: {}, pred: {}, value: {}\n\
                    backtrace context {}\n\
                    error during backtrace cache verification\n\
                    {}",
                    ix, pred, value,
                    self.to_ml_string(),
                    format_args!($($blah)*),
                )
            };
        }

        if ix >= self.cache_loc.len() {
            error!(
                "expected ix < cache_loc.len(), got {} >= {}",
                ix,
                self.cache_loc.len(),
            )
        }
        if self.cache_pred[ix] != pred {
            error!(
                "expected cache_pred[ix] == pred, got cache_pred[{}] = {} != {}",
                ix, self.cache_pred[ix], pred,
            )
        }
        if self.cache_loc[ix] != convert(value, "check_cache_verifier: value") {
            error!(
                "expected cache_loc[ix] == value, got cache_loc[{}] = {} != {}",
                ix, self.cache_loc[ix], value,
            )
        }

        Ok(())
    }
    pub fn skip_cache_verifier<'data>(&self, parser: &mut impl CanParse<'data>) -> Res<()> {
        let _ix = parser.u16()?;
        let _pred = parser.u16()?;
        let _value = parser.u64()?;
        Ok(())
    }
}
