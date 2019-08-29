//! Time-related types.
//!
//! - `SinceStart`: indicates a date since the start of the profiling run. Essentially a wrapper
//!     around a `std::time::Duration`.

use stdweb::{js, Value};

use crate::{Parser, Res};

pub use std::time::Duration;

pub type DateTime = chrono::DateTime<chrono::Utc>;

/// Wrapper around a duration.
///
/// This type represents a point in time **relative to** the start time of the run of the program
/// being profiled, which is a [`Date`] (an absolute point in time).
///
/// [`Date`]: struct.date.html (The Date struct)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct SinceStart {
    /// Actual duration.
    duration: Duration,
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
    fn sub(self, other: Self) -> Self {
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
        Parser::parse_all(s.as_ref(), Parser::date, "date")
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
/// [`SinceStart`]: struct.sincestart.html (The SincStart struct)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Date {
    /// Actual date.
    date: DateTime,
}
impl Date {
    /// Constructor from a unix UTC timestamp.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use alloc_data::Date;
    /// let date = Date::of_timestamp(1566489242, 7000572);
    /// assert_eq! { date.to_string(), "2019-08-22 15:54:02.007000572 UTC" }
    /// ```
    pub fn of_timestamp(secs: i64, nanos: u32) -> Self {
        use chrono::{offset::TimeZone, Utc};
        let date = Utc.timestamp(secs, nanos);
        Date { date }
    }

    /// Timestamp version of a date.
    ///
    /// Returns a pair `(` timestamp's seconds `,` timestamp's subsec nanoseconds `)`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use alloc_data::Date;
    /// let (secs, subsec_nanos): (i64, u32) = (1566489242, 7000572);
    /// let date = Date::of_timestamp(secs, subsec_nanos);
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
    /// let mut date = Date::of_timestamp(1566489242, 7000572);
    /// assert_eq! { date.to_string(), "2019-08-22 15:54:02.007000572 UTC" }
    /// let duration = SinceStart::from_str("7.003000001").unwrap();
    /// date.add(duration);
    /// assert_eq! { date.to_string(), "2019-08-22 15:54:09.010000573 UTC" }
    /// ```
    pub fn add(&mut self, duration: SinceStart) {
        self.date = self.date + chrono::Duration::from_std(duration.duration).unwrap()
    }

    /// JS version of a date.
    pub fn as_js(&self) -> Value {
        use chrono::{Datelike, Timelike};
        js!(
            return new Date(Date.UTC(
                @{self.date.year()},
                @{self.date.month0()},
                @{self.date.day()},
                @{self.date.hour()},
                @{self.date.minute()},
                @{self.date.second()},
                @{self.date.nanosecond() / 1_000_000},
            ))
        )
    }

    /// The hours/minutes/seconds/millis of a date.
    ///
    /// This is currently used only for debugging purposes.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use alloc_data::{Date, SinceStart};
    /// let mut date = Date::of_timestamp(1566489242, 7000572);
    /// assert_eq! { date.to_string(), "2019-08-22 15:54:02.007000572 UTC" }
    /// let (h, m, s, mi) = date.time_info();
    /// assert_eq! {  h, 15 }
    /// assert_eq! {  m, 54 }
    /// assert_eq! {  s, 2 }
    /// assert_eq! { mi, 7 }
    /// ```
    pub fn time_info(&self) -> (u32, u32, u32, u32) {
        use chrono::Timelike;
        (
            self.date.hour(),
            self.date.minute(),
            self.date.second(),
            self.date.nanosecond() / 1_000_000,
        )
    }
}

swarkn::display! {
    impl for SinceStart {
        self, fmt => {
            let mut nanos = format!(".{:>09}", self.duration.subsec_nanos());
            // Remove trailing zeros.
            loop {
                match nanos.pop() {
                    // Remove zeros.
                    Some('0') => (),
                    // There was nothing but zeros, remove dot as well (last character).
                    Some('.') => break,
                    // Otherwise it's a number, we must keep it and stop removing stuff.
                    Some(c) => {
                        nanos.push(c);
                        break;
                    }
                    None => unreachable!(),
                }
            }
            write!(fmt, "{}{}", self.duration.as_secs(), nanos)
        }
    }
}
swarkn::display! {
    impl for Date {
        self, fmt => write!(fmt, "{}", self.date)
    }
}
