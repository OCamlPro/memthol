//! Structures representing time in different ways.

prelude! {
    *, serde::*,
}

use crate::err::Res;

pub use std::time::{Duration, Instant};

/// Re-exports from `chrono`.
pub mod chrono {
    pub use chrono::*;
}

pub mod duration;
mod lifetime;
mod since_start;

// Re-exporting sub-module stuff.
pub use self::{duration::DurationExt, lifetime::Lifetime, since_start::SinceStart};

/// Type alias for a `chrono` local date/time.
pub type DateTime = chrono::DateTime<self::chrono::offset::Local>;

/// An actual, absolute date.
///
/// As opposed to [`SinceStart`] which represents a point in time as a duration since the start date
/// of the run of the program being profiled.
///
/// In practice, this type is just a wrapper around a [`chrono`] date.
///
/// [`chrono`]: https://crates.io/crates/chrono (The chrono crate on crates.io)
/// [`SinceStart`]: struct.SinceStart.html (The SinceStart struct)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Date {
    /// Actual date.
    date: DateTime,
}

impl Date {
    /// Constructor from a unix timestamp.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use base::prelude::time::*;
    /// let (secs, subsec_nanos) = (1566489242, 7000572);
    /// let date = Date::from_timestamp(secs, subsec_nanos);
    /// assert_eq! { date.timestamp(), (secs, subsec_nanos) }
    /// ```
    pub fn from_timestamp(secs: i64, nanos: u32) -> Self {
        use self::chrono::offset::{Local, TimeZone};
        let date = Local.timestamp(secs, nanos);
        Date { date }
    }

    /// The current date.
    pub fn now() -> Self {
        chrono::offset::Local::now().into()
    }

    /// Constructor from an ocaml duration.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use base::prelude::time::*;
    /// let secs = 156648924_u64;
    /// let subsec_micros = 270005_u32;
    /// let micros = secs * 1_000_000 + (subsec_micros as u64);
    /// let date = Date::from_micros(micros);
    /// assert_eq! { date.timestamp(), (secs as i64, subsec_micros * 1_000) }
    /// ```
    pub fn from_micros(micros: u64) -> Self {
        use self::chrono::offset::{Local, TimeZone};
        let secs = micros / 1_000_000;
        let subsec_micros: u32 = convert(micros % 1_000_000, "from_microsecs: nanos");
        let date = Local.timestamp(convert(secs, "from_microsecs: secs"), subsec_micros * 1_000);
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
    /// use base::prelude::time::*;
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
    /// use base::prelude::time::*;
    /// let (secs, subsec_nanos) = (1_566_489_242, 7_000_572);
    /// let mut date = Date::from_timestamp(secs, subsec_nanos);
    /// let duration = SinceStart::parse_secs("7.003000001").unwrap();
    /// date.add(duration);
    /// assert_eq! { date.timestamp(), (secs + 7, subsec_nanos + 3_000_001) }
    /// ```
    pub fn add(&mut self, duration: SinceStart) {
        self.date = self.date + chrono::Duration::from_std(duration.into()).unwrap()
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
    /// use base::prelude::time::*;
    /// let mut date = Date::from_timestamp(1566489242, 7000572);
    /// let (_h, m, s, mi) = date.time_info();
    /// // Can't check the hours as this depends on the local system time.
    /// assert_eq! {  m, 54 }
    /// assert_eq! {  s, 2 }
    /// assert_eq! { mi, 7 }
    /// ```
    pub fn time_info(&self) -> (u32, u32, u32, u32) {
        use self::chrono::Timelike;
        (
            self.date.hour(),
            self.date.minute(),
            self.date.second(),
            self.date.nanosecond() / 1_000_000,
        )
    }
}

implement! {
    impl Date {
        Display {
            |&self, fmt| self.date.fmt(fmt)
        }

        From {
            from DateTime => |date| Self { date }
        }
    }
}

impl ops::Sub<Self> for Date {
    type Output = SinceStart;
    fn sub(self, other: Self) -> Self::Output {
        let duration = (self.date - other.date)
            .to_std()
            .expect("fatal error while subtracting two dates");
        duration.into()
    }
}
impl ops::Add<SinceStart> for Date {
    type Output = Self;
    fn add(mut self, duration: SinceStart) -> Self::Output {
        self.date = self.date
            + chrono::Duration::from_std(duration.into())
                .expect("fatal error while adding a duration to a date");
        self
    }
}
