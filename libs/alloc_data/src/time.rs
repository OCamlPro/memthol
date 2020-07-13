//! Time-related types.
//!
//! - `SinceStart`: indicates a date since the start of the profiling run. Essentially a wrapper
//!     around a `std::time::Duration`.

use serde_derive::{Deserialize, Serialize};

prelude! {}

pub use std::time::Duration;

/// Re-exports from `chrono`.
pub mod chrono {
    pub use chrono::*;
}

pub type DateTime = time::chrono::DateTime<self::chrono::Utc>;

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
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
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
/// [`SinceStart`]: struct.sincestart.html (The SincStart struct)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Date {
    /// Actual date.
    date: DateTime,
}
impl Default for Date {
    fn default() -> Self {
        Self::of_timestamp(0, 0)
    }
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
        use time::chrono::{offset::TimeZone, Utc};
        let date = Utc.timestamp(secs, nanos);
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

    pub fn copy_add(&self, duration: SinceStart) -> Date {
        let mut date = self.clone();
        date.add(duration);
        date
    }

    // /// JS version of a date.
    // pub fn as_js(&self) -> Value {
    //     use chrono::{Datelike, Timelike};
    //     js!(
    //         return new Date(Date.UTC(
    //             @{self.date.year()},
    //             @{self.date.month0()},
    //             @{self.date.day()},
    //             @{self.date.hour()},
    //             @{self.date.minute()},
    //             @{self.date.second()},
    //             @{self.date.nanosecond() / 1_000_000},
    //         ))
    //     )
    // }

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
        use time::chrono::Timelike;
        (
            self.date.hour(),
            self.date.minute(),
            self.date.second(),
            self.date.nanosecond() / 1_000_000,
        )
    }
}
