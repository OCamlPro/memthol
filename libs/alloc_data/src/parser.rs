//! Allocation data parsers.

prelude! {}

pub use data_parser::*;

peg::parser! {
    grammar data_parser() for str {
        /// Whitespaces.
        rule ws() = quiet! {
            [' ' | '\n' | '\t']+
        }
        /// Whitespaces and comments.
        rule _() = quiet! {
            ws()?
        }

        /// Integer, as a string slice.
        rule integer() -> &'input str
        = $(
            "0"
            / ['1'..='9'] ['0'..='9']*
        )

        /// Integer, big uint.
        pub rule big_uint() -> BigUint
        = quiet! {
            n: integer() {?
                BigUint::parse_bytes(n.as_bytes(), 10).ok_or("illegal integer (big uint)")
            }
        }
        / expected!("integer (big uint)")

        /// Integer, u32.
        pub rule u32() -> u32
        = quiet! {
            n: integer() {?
                n.parse().map_err(|e| "illegal integer (u32)")
            }
        }
        / expected!("integer (u32)")

        /// Integer, u64.
        pub rule u64() -> u64
        = quiet! {
            n: integer() {?
                n.parse().map_err(|e| "illegal integer (u64)")
            }
        }
        / expected!("integer (u64)")

        /// Integer, usize.
        pub rule usize() -> usize
        = quiet! {
            n: integer() {?
                n.parse().map_err(|e| "illegal integer (usize)")
            }
        }
        / expected!("integer (usize)")

        /// A backquote-delimited string.
        pub rule string() -> &'input str
        = "`" s: $( (!['`'] [_])* ) "`" { s }
        / expected!("string (delimited by backquotes)")

        /// A whitespace-separated list of strings.
        pub rule string_list() -> Vec<String>
        = "[" list: ( (_ s: string() { s.to_string() })* ) _ "]" { list }
        / expected!("list of whitespace-separated, backquote-delimited strings")

        /// Parses a location.
        pub rule loc() -> Loc
        = file: string()
            _ ":"
            _ line: usize()
            _ ":"
            _ col_start: usize()
            _ "-"
            _ col_end: usize()
        {
            Loc::new(file, line, (col_start, col_end))
        }
        / expected!("file location")

        /// Parses a location followed a hashtag `#` and a count (integer, usize).
        pub rule counted_loc() -> CLoc
        = loc: loc() "#" count: usize() { CLoc::new(loc, count) }

        /// A whitespace-separated list of locations.
        pub rule loc_list() -> Vec<CLoc>
        = "[" list: ( (_ loc: counted_loc() { loc })* ) _ "]" { list }
        / expected!("list of whitespace-separated locations")


        /// Parses an amount of seconds as a float with nanosecond precision.
        pub rule secs() -> (u64, u32)
        = secs: u64() sub_sec: (
            sub_sec: (
                "."
                heading_zeros_str: $(['0']*)
                sub_sec_opt: u32()? {
                    (heading_zeros_str.len(), sub_sec_opt)
                }
            )? {
                sub_sec.and_then(|(heading_zeros_count, sub_sec_opt)|
                    sub_sec_opt.map(
                        |sub_sec| (heading_zeros_count, sub_sec)
                    )
                )
            }
        ) {?
            let sub_sec_res = sub_sec.map(
                |(heading_zeros_count, sub_sec)| {
                    // Maximal supported precision is nanoseconds (10^-9).

                    // Number of digits in the sub-sec value, including heading zeros.
                    let digit_count = {
                        let (mut acc, mut len) = (sub_sec, 0);
                        while acc > 0 {
                            acc = acc / 10;
                            len += 1
                        }
                        len + heading_zeros_count
                    };

                    // Make sure we have nanoseconds at most.
                    if digit_count > 9 {
                        Err("\
                            illegal date (seconds), \
                            precision below nanoseconds is not supported\
                        ")
                    } else {
                        // Correct `sub_sec` so that it represents nanoseconds.
                        let sub_sec_offset = 9 - digit_count;
                        let mut sub_sec = sub_sec;
                        for _ in 0..sub_sec_offset {
                            sub_sec = sub_sec * 10
                        }
                        Ok(sub_sec)
                    }
                }
            ).unwrap_or_else(
                || Ok(0u32)
            );

            sub_sec_res.map(|sub_sec|(secs, sub_sec))
        }
        / expected!("seconds (float with at most nanosecond sub-second precision)")

        /// Parses an amount of seconds representing a lifetime.
        pub rule lifetime() -> Lifetime
        = secs: secs() {
            Duration::new(secs.0, secs.1).into()
        }
        / expected!("an amount of seconds (float, lifetime)")

        /// Parses an amount of seconds since the start of the run.
        pub rule since_start() -> SinceStart
        = secs: secs() {
            Duration::new(secs.0, secs.1).into()
        }
        / expected!("an amount of seconds (float) since the start of the run")

        /// Parses an optional amount of seconds since the start of the run.
        pub rule since_start_opt() -> Option<SinceStart>
        = "_" { None }
        / time: since_start() { Some(time) }
        / expected!("an optional amount of seconds (float) since the start of the run, `_` if none")

        /// Parses a date.
        pub rule date() -> Date
        = secs: secs() {?
            let (secs, sub_secs) = secs;
            i64::try_from(secs).map(
                |secs| Date::of_timestamp(secs, sub_secs)
            ).map_err(|_| "illegal amount of seconds for a date")
        }

        /// Parses a uid.
        pub rule uid() -> Uid
        = quiet! {
            uid: big_uint() { uid.into() }
        }
        / expected!("UID (big uint)")

        /// Parses an allocation kind.
        pub rule alloc_kind() -> AllocKind
        = "Minor" { AllocKind::Minor }
        / "MajorPostponed" { AllocKind::MajorPostponed }
        / "Major" { AllocKind::Major }
        / "Serialized" { AllocKind::Serialized }
        / "_" { AllocKind::Unknown }
        / expected!("allocation kind")

        /// Parses an allocation.
        pub rule new_alloc() -> Alloc
        = uid: uid()
            _ ":"
            _ kind: alloc_kind()
            _ size: u32()

            // Callstack.
            _ callstack: loc_list()
            // User-provided labels.
            _ labels: string_list()
            // Time Of Creation.
            _ toc: since_start()
            // Time Of Death.
            _ tod: since_start_opt()
        {
            Alloc::new(uid, kind, size, callstack, labels, toc, tod)
        }
        / expected!("allocation data")

        /// Parses the new allocations of a diff.
        pub rule diff_new_allocs() -> Vec<Alloc>
        = "new" _ "{"
            new_allocs: ( _ new_alloc: new_alloc() { new_alloc })*
        _ "}" {
            new_allocs
        }
        / expected!("list of new allocations")

        /// Parses the death of an allocation.
        pub rule dead_alloc() -> (Uid, SinceStart)
        = uid: uid() _ ":" _ secs: since_start() {  (uid, secs) }

        /// Parses the dead allocations of a diff.
        pub rule diff_dead_allocs() -> Vec<(Uid, SinceStart)>
        = "dead" _ "{"
            dead_allocs: (_ dead_alloc: dead_alloc() { dead_alloc })*
        _ "}" {
            dead_allocs
        }
        / expected!("list of dead allocations")

        /// Parses a dump diff, consumes heading/trailing whitespaces.
        pub rule diff() -> Diff =
            _ date: since_start()
            _ new: diff_new_allocs()
            _ dead: diff_dead_allocs()
            _
        {
            Diff::new(date, new, dead)
        }

        /// Parses a dump init, consumes heading/trailing whitespaces.
        pub rule init() -> Init =
            _ "start" _ ":" _ start_time: date()
            _ "word_size" _ ":" _ word_size: usize()
            _
        {
            Init::new(start_time, word_size)
        }
    }
}

/// Trait for types that can be parsed.
pub trait Parseable: Sized {
    /// Parses something.
    fn parse(text: impl AsRef<str>) -> Res<Self>;
}

macro_rules! implement {
    (parseable($txt:ident) for {
        $($inner:tt)*
    }) => {
        implement! {
            @($txt) $($inner)*
        }
    };

    (@($txt:ident)
        $ty:ty => +TryFrom $def:expr $( , $($tail:tt)*)?
    ) => {
        implement! {
            @($txt) $ty => $def
        }
        impl<'s> std::convert::TryFrom<&'s str> for $ty {
            type Error = err::Err;
            fn try_from($txt: &'s str) -> Res<Self> {
                Self::parse($txt)
            }
        }

        $(
            implement! {
                @($txt) $($tail)*
            }
        )?
    };

    (@($txt:ident)
        $ty:ty => $def:expr $( , $($tail:tt)*)?
    ) => {
        impl Parseable for $ty {
            fn parse(text: impl AsRef<str>) -> Res<Self> {
                let $txt = text.as_ref();
                let res = $def?;
                Ok(res)
            }
        }

        $(
            implement! {
                @($txt) $($tail)*
            }
        )?
    };

    (@($txt:ident)) => {};
}

implement! {
    parseable(text) for {
        usize => usize(text),
        u32 => u32(text),
        u64 => u64(text),

        SinceStart => +TryFrom since_start(text),
        Lifetime => +TryFrom lifetime(text),
        Date => +TryFrom date(text),

        Uid => +TryFrom uid(text),
        AllocKind => +TryFrom alloc_kind(text),
        Loc => +TryFrom loc(text),
        CLoc => +TryFrom counted_loc(text),

        Alloc => +TryFrom new_alloc(text),
        Diff => +TryFrom diff(text),
        Init => +TryFrom init(text),
    }
}
