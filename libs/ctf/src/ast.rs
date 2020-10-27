//! AST for memtrace's CTF format.

prelude! {}

/// A span for some type.
#[derive(Debug, Clone)]
pub struct Span<T> {
    /// Beginning of the span.
    pub begin: T,
    /// End of the span.
    pub end: T,
}

impl<T> Span<T>
where
    T: PartialOrd + Ord,
{
    /// Constructor.
    ///
    /// Fails if `begin > end`.
    pub fn new(begin: T, end: T) -> Res<Self> {
        if begin > end {
            bail!("non-monotonous values")
        }
        Ok(Self { begin, end })
    }

    /// Checks whether the input value is in the span.
    pub fn contains(&self, value: T) -> bool {
        (self.begin <= value) && (value <= self.end)
    }

    /// Checks whether the input value reference is in the span.
    pub fn contains_ref(&self, value: impl AsRef<T>) -> bool
    where
        for<'a> &'a T: PartialOrd + Ord,
    {
        let value = value.as_ref();
        (&self.begin <= value) && (value <= &self.end)
    }
}

impl<T> Span<T> {
    /// Map over the bounds of the span.
    pub fn map<U>(self, f: impl Fn(T) -> U) -> Span<U> {
        Span {
            begin: f(self.begin),
            end: f(self.end),
        }
    }

    /// Reference version of a span.
    pub fn as_ref(&self) -> Span<&T> {
        Span {
            begin: &self.begin,
            end: &self.end,
        }
    }
}

impl Span<Clock> {
    /// Pretty printer for a span of `u64` durations.
    pub fn pretty_time(&self) -> Span<time::Duration> {
        Span {
            begin: time::Duration::from_millis(self.begin),
            end: time::Duration::from_millis(self.end),
        }
    }
}

impl<T> fmt::Display for Span<T>
where
    T: fmt::Display,
{
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "[{}, {}]", self.begin, self.end)
    }
}

/// Header-related AST types.
pub mod header {
    prelude! {}

    /// Plain header, factors common data between [`Ctf`] and [`Packet`].
    ///
    /// [`Ctf`]: struct.Ctf.html (Ctf struct)
    /// [`Packet`]: struct.Packet.html (Packet struct)
    #[derive(Debug, Clone)]
    pub struct Header {
        /// Size of the content of the packet/stream, **without the header**.
        pub content_size: u32,
        /// Size of the content of the packet/stream, **with the header**.
        pub total_content_size: u32,
        /// Header timestamp.
        pub timestamp: Span<Clock>,
        /// Allocation UID range created in whatever the header represents.
        pub alloc_id: Span<AllocUid>,
        /// Associated PID.
        pub pid: Pid,
        /// Memtrace version in use.
        pub version: u16,
    }
    impl Header {
        /// True if the element this header is for has a context.
        ///
        /// Only true in memtrace `v2.*` and above.
        pub fn has_context(&self) -> bool {
            self.version >= 2
        }
    }

    /// CTF header, top-level header of a memtrace dump.
    #[derive(Debug, Clone)]
    pub struct Ctf {
        /// Actual header.
        header: Header,
        /// True if the dump uses big-endian encoding.
        big_e: bool,
    }
    impl ops::Deref for Ctf {
        type Target = Header;
        fn deref(&self) -> &Header {
            &self.header
        }
    }

    impl Ctf {
        /// Constructor.
        pub fn new(header: Header, big_e: bool) -> Self {
            Self { header, big_e }
        }
        /// Header accessor.
        pub fn header(&self) -> &Header {
            &self.header
        }
        /// True if the dump uses big-endian encoding.
        pub fn is_be(&self) -> bool {
            self.big_e
        }
    }

    /// Packet header, contains information about a sequence of events.
    #[derive(Debug, Clone)]
    pub struct Packet {
        /// Actual header.
        header: Header,
        /// Cache-check structure.
        cache_check: ast::CacheCheck,
        /// Packet ID.
        id: usize,
    }
    impl ops::Deref for Packet {
        type Target = Header;
        fn deref(&self) -> &Header {
            &self.header
        }
    }

    impl Packet {
        /// Constructor.
        pub fn new(id: usize, header: Header, cache_check: ast::CacheCheck) -> Self {
            Self {
                id,
                header,
                cache_check,
            }
        }
        /// Header accessor.
        pub fn header(&self) -> &Header {
            &self.header
        }
        /// Cache-checker.
        pub fn cache_check(&self) -> &ast::CacheCheck {
            &self.cache_check
        }
        /// Packet id.
        pub fn id(&self) -> usize {
            self.id
        }
    }

    /// An event header.
    #[derive(Debug, Clone)]
    pub struct Event {
        /// Timestamp of the event.
        timestamp: u32,
        /// Event code.
        code: u8,
    }
    impl Event {
        /// Constructor.
        pub fn new(timestamp: u32, code: u8) -> Self {
            Self { timestamp, code }
        }
        /// Timestamp accessor.
        pub fn timestamp(&self) -> u32 {
            self.timestamp
        }
        /// Event code accessor.
        pub fn code(&self) -> u8 {
            self.code
        }
    }
}

/// Event-related types.
pub mod event {
    use super::*;
    // prelude! {}

    /// Code for info events.
    const INFO_CODE: u32 = 0;
    /// Code for location events.
    const LOCS_CODE: u32 = 1;
    /// Code for allocation events.
    const ALLOC_CODE: u32 = 2;
    /// Code for promotion events.
    const PROMOTION_CODE: u32 = 3;
    /// Code for collection events.
    const COLLECTION_CODE: u32 = 4;

    /// Relative codes encoding small allocations.
    const SMALL_ALLOC_REDUCED_CODES: Span<u32> = Span { begin: 1, end: 16 };
    /// Offset for small allocation codes.
    const SMALL_ALLOC_OFFSET: u32 = 100;

    /// Absolute codes encoding small allocations.
    const SMALL_ALLOC_CODES: Span<u32> = Span {
        begin: SMALL_ALLOC_REDUCED_CODES.begin + SMALL_ALLOC_OFFSET,
        end: SMALL_ALLOC_REDUCED_CODES.end + SMALL_ALLOC_OFFSET,
    };

    /// Event kind.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum Kind {
        /// Info event.
        Info,
        /// Locations event.
        Locs,
        /// Allocation event.
        Alloc,
        /// Promotion event.
        Promotion,
        /// Collection event.
        Collection,
        /// Stores a value between `1` and `16`.
        SmallAlloc(u32),
    }
    impl Kind {
        /// Checks the invariant for the `SmallAlloc` variant.
        fn small_alloc_invariant(code: u32) {
            if !SMALL_ALLOC_REDUCED_CODES.contains(code) {
                panic!(
                    "illegal small allocation reduced code, expected {} <= {} <= {}",
                    SMALL_ALLOC_REDUCED_CODES.begin, code, SMALL_ALLOC_REDUCED_CODES.end
                )
            }
        }

        /// True if the event is an info event.
        pub fn is_info(self) -> bool {
            self == Self::Info
        }

        /// Constructor from an event code.
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

        /// Event code of an event kind.
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

    /// An event, decoded version.
    #[derive(Debug, Clone)]
    pub enum Event<'data> {
        /// Location event.
        Locs(Locs<'data>),
        /// Allocation event.
        Alloc(Alloc),
        /// Promotion event.
        Promotion(u64),
        /// Collection event.
        Collection(u64),
    }
    impl<'data> Event<'data> {
        /// One-word description of the event.
        pub fn name(&self) -> &'static str {
            match self {
                Self::Locs(_) => "locations",
                Self::Alloc(_) => "allocation",
                Self::Promotion(_) => "promotion",
                Self::Collection(_) => "collection",
            }
        }

        /// Verbose description of the event.
        pub fn desc(&self) -> String {
            let name = self.name();
            match self {
                Self::Alloc(alloc) => format!(
                    "{}({} @ {})",
                    name,
                    alloc.id,
                    alloc.alloc_time.display_micros(),
                ),
                Self::Collection(id) => format!("{}({})", name, id),
                Self::Promotion(id) => format!("{}({})", name, id),
                _ => name.into(),
            }
        }
    }

    /// Information event.
    ///
    /// This kind of event is expected to appear exactly once at the beginning, right after the CTF
    /// (top-level) header.
    #[derive(Debug, Clone)]
    pub struct Info<'data> {
        /// Sample rate.
        pub sample_rate: f64,
        /// Word size.
        pub word_size: u8,
        /// Executable name.
        pub exe_name: String,
        /// Name of the host system.
        pub host_name: String,
        /// Parameters for the executable.
        pub exe_params: String,
        /// Process PID.
        pub pid: u64,
        /// Context.
        pub context: Option<&'data str>,
    }
    impl<'data> Info<'data> {
        /// Code for this event.
        pub const fn event_code() -> u32 {
            INFO_CODE
        }
        /// Name of this event.
        pub const fn name() -> &'static str {
            "trace_info"
        }

        /// Turns itself into an `Init`.
        pub fn to_init(&self, start_time: time::Date) -> alloc_data::Init {
            alloc_data::Init::new(
                start_time,
                None,
                convert(self.word_size, "ctf parser: word_size"),
                false,
            )
            .sampling_rate(self.sample_rate)
        }
    }

    /// Allocation event.
    #[derive(Debug, Clone)]
    pub struct Alloc {
        /// Event UID.
        pub id: u64,
        /// Size of the allocation.
        pub len: usize,
        /// Timestamp at which the allocation occured.
        pub alloc_time: time::Duration,
        /// Sample count.
        pub nsamples: usize,
        /// True if major.
        pub is_major: bool,
        /// Backtrace of the allocation.
        pub backtrace: Vec<usize>,
        /// Backtrace common prefix *w.r.t.* the previous backtrace.
        pub common_pref_len: usize,
    }
    impl Alloc {
        /// Code of this event.
        pub const fn event_code() -> u32 {
            ALLOC_CODE
        }
        /// Name of this event.
        pub const fn name() -> &'static str {
            "alloc"
        }
    }

    /// Promotion event.
    #[derive(Debug, Clone)]
    pub struct Promotion {
        /// Allocation UID delta.
        ///
        /// Specifies the difference between the UID of the last allocation and the UID that is
        /// being promoted.
        pub id_delta: u64,
    }
    impl Promotion {
        /// Code of this event.
        pub const fn event_code() -> u32 {
            PROMOTION_CODE
        }
        /// Name of this event.
        pub const fn name() -> &'static str {
            "promote"
        }
    }

    /// Collection event.
    #[derive(Debug, Clone)]
    pub struct Collection {
        /// Allocation UID delta.
        ///
        /// Specifies the difference between the UID of the last allocation and the UID that is
        /// being collected.
        pub id_delta: u64,
    }
    impl Collection {
        /// Code of this event.
        pub const fn event_code() -> u32 {
            COLLECTION_CODE
        }
        /// Name of this event.
        pub const fn name() -> &'static str {
            "collect"
        }
    }
}

/// A collection of locations.
#[derive(Debug, Clone)]
pub struct Locs<'data> {
    /// ID of the locations.
    pub id: u64,
    /// Locations.
    pub locs: Vec<loc::Location<'data>>,
}

/// Cache-check data.
#[derive(Debug, Clone)]
pub struct CacheCheck {
    /// Currently unused.
    pub ix: u16,
    /// Currently unused.
    pub pred: u16,
    /// Currently unused.
    pub value: u64,
}
