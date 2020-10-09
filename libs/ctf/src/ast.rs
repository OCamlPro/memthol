prelude! {}

#[derive(Debug, Clone)]
pub struct Span<T> {
    pub begin: T,
    pub end: T,
}
impl<T> Span<T>
where
    T: PartialOrd + Ord,
{
    pub fn new(begin: T, end: T) -> Res<Self> {
        if begin > end {
            bail!("non-monotonous values")
        }
        Ok(Self { begin, end })
    }
    pub fn contains(&self, value: T) -> bool {
        (self.begin <= value) && (value <= self.end)
    }
    pub fn contains_ref(&self, value: impl AsRef<T>) -> bool
    where
        for<'a> &'a T: PartialOrd + Ord,
    {
        let value = value.as_ref();
        (&self.begin <= value) && (value <= &self.end)
    }
}

pub mod header {
    prelude! {}

    use std::ops::Deref;

    // #[derive(Debug, Clone)]
    // pub struct Packet {
    //     pub timestamp_beg: Clock,
    //     pub timestamp_end: Clock,

    //     pub flush_duration: u32,

    //     pub version: u16,

    //     pub pid: u64,

    //     pub cache_verify_ix: u16,
    //     pub cache_verify_pred: u16,
    //     pub cache_verify_val: u16,

    //     pub alloc_id_beg: u64,
    //     pub alloc_id_end: u64,
    // }

    // #[derive(Debug, Clone)]
    // pub struct Ctf {
    //     packet: Packet,
    // }
    // impl std::ops::Deref for Ctf {
    //     type Target = Packet;
    //     fn deref(&self) -> &Packet {
    //         &self.packet
    //     }
    // }
    // impl From<Packet> for Ctf {
    //     fn from(packet: Packet) -> Self {
    //         Self { packet }
    //     }
    // }

    #[derive(Debug, Clone)]
    pub struct Header {
        /// Size of the content of the packet/stream, **without the header**.
        pub content_size: u32,
        /// Size of the content of the packet/stream, **with the header**.
        pub total_content_size: u32,
        pub timestamp: Span<Clock>,
        pub alloc_id: Span<AllocId>,
        pub pid: Pid,
        pub version: u16,
    }
    impl Header {
        pub fn has_context(&self) -> bool {
            self.version >= 2
        }
    }

    #[derive(Debug, Clone)]
    pub struct Ctf {
        header: Header,
        big_e: bool,
    }
    impl Deref for Ctf {
        type Target = Header;
        fn deref(&self) -> &Header {
            &self.header
        }
    }
    impl Ctf {
        pub fn new(header: Header, big_e: bool) -> Self {
            Self { header, big_e }
        }
        pub fn is_be(&self) -> bool {
            self.big_e
        }
    }

    #[derive(Debug, Clone)]
    pub struct Packet {
        header: Header,
        cache_check: ast::CacheCheck,
    }
    impl Deref for Packet {
        type Target = Header;
        fn deref(&self) -> &Header {
            &self.header
        }
    }
    impl Packet {
        pub fn new(header: Header, cache_check: ast::CacheCheck) -> Self {
            Self {
                header,
                cache_check,
            }
        }
        pub fn header(&self) -> &Header {
            &self.header
        }
        pub fn cache_check(&self) -> &ast::CacheCheck {
            &self.cache_check
        }
    }

    #[derive(Debug, Clone)]
    pub struct Event {
        pub timestamp: u32,
        pub id: u8,
    }
}

pub mod event {
    use super::*;
    // prelude! {}

    const INFO_CODE: u32 = 0;
    const LOCS_CODE: u32 = 1;
    const ALLOC_CODE: u32 = 2;
    const PROMOTION_CODE: u32 = 3;
    const COLLECTION_CODE: u32 = 4;

    const SMALL_ALLOC_REDUCED_CODES: Span<u32> = Span { begin: 1, end: 16 };
    const SMALL_ALLOC_OFFSET: u32 = 100;

    const SMALL_ALLOC_CODES: Span<u32> = Span {
        begin: SMALL_ALLOC_REDUCED_CODES.begin + SMALL_ALLOC_OFFSET,
        end: SMALL_ALLOC_REDUCED_CODES.end + SMALL_ALLOC_OFFSET,
    };

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum Kind {
        Info,
        Locs,
        Alloc,
        Promotion,
        Collection,
        /// Stores a value between `1` and `16`.
        SmallAlloc(u32),
    }
    impl Kind {
        fn small_alloc_invariant(code: u32) {
            if !SMALL_ALLOC_REDUCED_CODES.contains(code) {
                panic!(
                    "illegal small allocation reduced code, expected {} <= {} <= {}",
                    SMALL_ALLOC_REDUCED_CODES.begin, code, SMALL_ALLOC_REDUCED_CODES.end
                )
            }
        }

        pub fn is_info(self) -> bool {
            self == Self::Info
        }

        pub fn from_code(code: u32) -> Res<Self> {
            let res = if code == INFO_CODE {
                Self::Info
            } else if code == LOCS_CODE {
                Self::Locs
            } else if code == ALLOC_CODE {
                Self::Alloc
            } else if code == PROMOTION_CODE {
                Self::Promotion
            } else if code == COLLECTION_CODE {
                Self::Collection
            } else if SMALL_ALLOC_CODES.contains(code) {
                let reduced_code = code - SMALL_ALLOC_OFFSET;
                Self::small_alloc_invariant(reduced_code);
                Self::SmallAlloc(reduced_code)
            } else {
                bail!("unexpected event code `{}`", code)
            };
            Ok(res)
        }

        pub fn code(self) -> u32 {
            match self {
                Self::Info => INFO_CODE,
                Self::Locs => LOCS_CODE,
                Self::Alloc => ALLOC_CODE,
                Self::Promotion => PROMOTION_CODE,
                Self::Collection => COLLECTION_CODE,
                Self::SmallAlloc(n) => {
                    Self::small_alloc_invariant(n);
                    n + 100
                }
            }
        }
    }

    #[derive(Debug, Clone)]
    pub enum Event<'data> {
        // Info(Info),
        Locs(Locs<'data>),
        Alloc(Alloc),
        Promotion(u64),
        Collection(u64),
    }
    impl<'data> Event<'data> {
        pub fn name(&self) -> &'static str {
            match self {
                Self::Locs(_) => "locations",
                Self::Alloc(_) => "allocation",
                Self::Promotion(_) => "promotion",
                Self::Collection(_) => "collection",
            }
        }
    }

    #[derive(Debug, Clone)]
    pub struct Info {
        pub sample_rate: f64,
        pub word_size: u8,
        pub exe_name: String,
        pub host_name: String,
        pub exe_params: String,
        pub pid: u64,
        pub context: Option<String>,
    }
    impl Info {
        pub const fn event_id() -> u32 {
            INFO_CODE
        }
        pub const fn name() -> &'static str {
            "trace_info"
        }
    }

    #[derive(Debug, Clone)]
    pub struct Alloc {
        pub id: u64,
        pub len: usize,
        pub nsamples: usize,
        pub is_major: bool,
        pub backtrace: SVec16<usize>,
        pub backtrace_len: usize,
        pub common_pref_len: usize,
    }
    impl Alloc {
        pub const fn event_id() -> u32 {
            ALLOC_CODE
        }
        pub const fn name() -> &'static str {
            "alloc"
        }
    }

    #[derive(Debug, Clone)]
    pub struct Promotion {
        pub id_delta: u64,
    }
    impl Promotion {
        pub const fn event_id() -> u32 {
            PROMOTION_CODE
        }
        pub const fn name() -> &'static str {
            "promote"
        }
    }

    #[derive(Debug, Clone)]
    pub struct Collection {
        pub id_delta: u64,
    }
    impl Collection {
        pub const fn event_id() -> u32 {
            COLLECTION_CODE
        }
        pub const fn name() -> &'static str {
            "collect"
        }
    }
}

#[derive(Debug, Clone)]
pub enum CachedVal<T> {
    Cached(u8),
    New(T),
}

#[derive(Debug, Clone)]
pub struct Loc {
    pub line: usize,
    pub start_char: usize,
    pub end_char: usize,
    pub file_path_code: String,
    pub def_name_code: String,
}

#[derive(Debug, Clone)]
pub struct Locs<'data> {
    pub id: u64,
    pub locs: SVec16<loc::Location<'data>>,
}

#[derive(Debug, Clone)]
pub struct BacktraceCode {
    pub tag: Tag,
    pub cache_bucket: u16,
}

#[derive(Debug, Clone)]
pub enum Tag {
    Hit0,
    Hit1,
    HitN(u8),
    Miss(u64),
}

#[derive(Debug, Clone)]
pub struct ShortAlloc {
    pub common_prefix: u64,
    pub new_suffix: Vec<BacktraceCode>,
}

#[derive(Debug, Clone)]
pub struct CacheCheck {
    pub ix: u16,
    pub pred: u16,
    pub value: u64,
}
