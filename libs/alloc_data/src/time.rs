//! Time-related types.
//!
//! - `SinceStart`: indicates a date since the start of the profiling run. Essentially a wrapper
//!     around a `std::time::Duration`.

prelude! {}

pub use std::time::Duration;

/// Re-exports from `chrono`.
pub mod chrono {
    pub use base::chrono::*;
}

pub type DateTime = time::chrono::DateTime<self::chrono::offset::Local>;

pub fn now() -> time::chrono::DateTime<chrono::offset::Local> {
    time::chrono::offset::Local::now()
}

/// Wrapper around a duration representing a lifetime.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Lifetime {
    /// Actual duration.
    duration: Duration,
}
impl Default for Lifetime {
    fn default() -> Self {
        Self {
            duration: Duration::default(),
        }
    }
}
impl std::ops::Deref for Lifetime {
    type Target = Duration;
    fn deref(&self) -> &Duration {
        &self.duration
    }
}
impl From<Duration> for Lifetime {
    fn from(duration: Duration) -> Self {
        Self { duration }
    }
}
impl Into<std::time::Duration> for Lifetime {
    fn into(self) -> std::time::Duration {
        self.duration
    }
}
impl fmt::Display for Lifetime {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        let mut dot_nanos = format!(".{}", self.duration.subsec_nanos());

        // We don't want to remove the `.` or the digit right after it.
        while dot_nanos.len() > 2 {
            match dot_nanos.pop() {
                // Pop zeros.
                Some('0') => continue,
                // Some digit, push it back and break out.
                Some(digit) => {
                    dot_nanos.push(digit);
                    break;
                }

                None => unreachable!("failed to pop `dot_nanos` but its length is > 2"),
            }
        }
        write!(fmt, "{}{}", self.duration.as_secs(), dot_nanos)
    }
}

impl Lifetime {
    /// A duration of 0 nanoseconds.
    pub fn zero() -> Self {
        Lifetime {
            duration: std::time::Duration::new(0, 0),
        }
    }

    /// Duration parser.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use std::time::Duration;
    /// use alloc_data::SinceStart;
    /// let s_list = [
    ///     ("320.74", Duration::new(320, 740_000_000)),
    ///     ("703470.0074", Duration::new(703470, 7_400_000)),
    ///     ("0.2", Duration::new(0, 200_000_000)),
    ///     ("7.0", Duration::new(7, 0)),
    /// ];
    /// for (s, exp) in &s_list {
    ///     let date = SinceStart::from_str(s).unwrap();
    ///     assert_eq! { &*date, exp }
    /// }
    /// ```
    pub fn from_str<Str: AsRef<str>>(s: Str) -> Res<Self> {
        parser::lifetime(s.as_ref()).map_err(|e| e.into())
    }
}

/// Wrapper around a duration.
///
/// This type represents a point in time **relative to** the start time of the run of the program
/// being profiled, which is a [`Date`] (an absolute point in time).
///
/// [`Date`]: struct.date.html (The Date struct)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct SinceStart {
    /// Actual duration.
    duration: Duration,
}
impl Default for SinceStart {
    fn default() -> Self {
        Self {
            duration: Duration::default(),
        }
    }
}
impl std::ops::Deref for SinceStart {
    type Target = Duration;
    fn deref(&self) -> &Duration {
        &self.duration
    }
}
impl From<Duration> for SinceStart {
    fn from(duration: Duration) -> Self {
        Self { duration }
    }
}
impl Into<std::time::Duration> for SinceStart {
    fn into(self) -> std::time::Duration {
        self.duration
    }
}
impl std::ops::Sub for SinceStart {
    type Output = Self;
    fn sub(mut self, other: Self) -> Self {
        self.duration = self.duration - other.duration;
        self
    }
}
impl<'a> std::ops::Sub<SinceStart> for &'a SinceStart {
    type Output = SinceStart;
    fn sub(self, mut other: SinceStart) -> SinceStart {
        other.duration = self.duration - other.duration;
        other
    }
}
impl<'a> std::ops::Sub<&'a SinceStart> for SinceStart {
    type Output = SinceStart;
    fn sub(mut self, other: &'a SinceStart) -> SinceStart {
        self.duration = self.duration - other.duration;
        self
    }
}
impl<'a, 'b> std::ops::Sub<&'a SinceStart> for &'b SinceStart {
    type Output = SinceStart;
    fn sub(self, other: &'a SinceStart) -> SinceStart {
        SinceStart {
            duration: self.duration - other.duration,
        }
    }
}

impl SinceStart {
    /// A duration of 0 nanoseconds.
    pub fn zero() -> Self {
        SinceStart {
            duration: std::time::Duration::new(0, 0),
        }
    }
    pub fn is_zero(&self) -> bool {
        self.duration.subsec_nanos() == 0 && self.duration.as_secs() == 0
    }

    pub fn from_timestamp(secs: u64, nanos: u32) -> Self {
        Self {
            duration: Duration::new(secs, nanos),
        }
    }

    pub fn add(&mut self, other: Self) {
        self.duration = self.duration + other.duration
    }

    pub fn div(&mut self, ratio: u32) {
        self.duration = self.duration / ratio
    }

    /// Duration parser.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use std::time::Duration;
    /// use alloc_data::SinceStart;
    /// let s_list = [
    ///     ("320.74", Duration::new(320, 740_000_000)),
    ///     ("703470.0074", Duration::new(703470, 7_400_000)),
    ///     ("0.2", Duration::new(0, 200_000_000)),
    ///     ("7.0", Duration::new(7, 0)),
    /// ];
    /// for (s, exp) in &s_list {
    ///     let date = SinceStart::from_str(s).unwrap();
    ///     assert_eq! { &*date, exp }
    /// }
    /// ```
    pub fn from_str<Str: AsRef<str>>(s: Str) -> Res<Self> {
        parser::since_start(s.as_ref()).map_err(|e| e.into())
    }

    /// Subtraction, yields a lifetime.
    pub fn sub_to_lt(&self, other: &Self) -> Lifetime {
        (self.duration - other.duration).into()
    }
}

/// An actual, absolute date.
///
/// As opposed to [`SinceStart`] which represents a point in time as a duration since the start date
/// of the run of the program being profiled.
///
/// In practice, this type is just a wrapper around a [`chrono`] date.
///
/// [`chrono`]: https://crates.io/crates/chrono (The chrono crate on crates.io)
/// [`SinceStart`]: struct.sincestart.html (The SinceStart struct)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Date {
    /// Actual date.
    date: DateTime,
}
impl Default for Date {
    fn default() -> Self {
        Self::from_timestamp(0, 0)
    }
}
impl Date {
    /// Constructor from a unix timestamp.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use alloc_data::Date;
    /// let (secs, subsec_nanos) = (1566489242, 7000572);
    /// let date = Date::from_timestamp(secs, subsec_nanos);
    /// assert_eq! { date.timestamp(), (secs, subsec_nanos) }
    /// ```
    pub fn from_timestamp(secs: i64, nanos: u32) -> Self {
        use time::chrono::offset::{Local, TimeZone};
        let date = Local.timestamp(secs, nanos);
        Date { date }
    }

    /// Constructor from an ocaml duration.
    pub fn from_microsecs(micros: i64) -> Self {
        use time::chrono::offset::{Local, TimeZone};
        let secs = micros / 1_000_000;
        let subsec_micros: u32 =
            convert((micros - secs * 1_000_000).abs(), "from_microsecs: nanos");
        let date = Local.timestamp(secs, subsec_micros * 1_000);
        Date { date }
    }

    /// Date accessor.
    pub fn date(&self) -> &DateTime {
        &self.date
    }

    /// Timestamp version of a date.
    ///
    /// Returns a pair `(` timestamp's seconds `,` timestamp's subsec nanoseconds `)`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use alloc_data::Date;
    /// let (secs, subsec_nanos) = (1_566_489_242, 7_000_572);
    /// let date = Date::from_timestamp(secs, subsec_nanos);
    /// assert_eq! { date.timestamp(), (secs, subsec_nanos) }
    /// ```
    pub fn timestamp(&self) -> (i64, u32) {
        (self.date.timestamp(), self.date.timestamp_subsec_nanos())
    }

    /// Adds a duration.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use alloc_data::{Date, SinceStart};
    /// let (secs, subsec_nanos) = (1_566_489_242, 7_000_572);
    /// let mut date = Date::from_timestamp(secs, subsec_nanos);
    /// let duration = SinceStart::from_str("7.003000001").unwrap();
    /// date.add(duration);
    /// assert_eq! { date.timestamp(), (secs + 7, subsec_nanos + 3_000_001) }
    /// ```
    pub fn add(&mut self, duration: SinceStart) {
        self.date = self.date + chrono::Duration::from_std(duration.duration).unwrap()
    }

    /// Subtraction.
    pub fn sub(self, rhs: Self) -> Res<SinceStart> {
        let duration = (self.date - rhs.date).to_std().map_err(|e| e.to_string())?;
        Ok(SinceStart { duration })
    }

    pub fn copy_add(&self, duration: SinceStart) -> Date {
        let mut date = self.clone();
        date.add(duration);
        date
    }

    /// The hours/minutes/seconds/millis of a date.
    ///
    /// This is currently used only for debugging purposes.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use alloc_data::{Date, SinceStart};
    /// let mut date = Date::from_timestamp(1566489242, 7000572);
    /// let (_h, m, s, mi) = date.time_info();
    /// // Can't check the hours as this depends on the local system time.
    /// assert_eq! {  m, 54 }
    /// assert_eq! {  s, 2 }
    /// assert_eq! { mi, 7 }
    /// ```
    pub fn time_info(&self) -> (u32, u32, u32, u32) {
        use time::chrono::Timelike;
        (
            self.date.hour(),
            self.date.minute(),
            self.date.second(),
            self.date.nanosecond() / 1_000_000,
        )
    }
}
