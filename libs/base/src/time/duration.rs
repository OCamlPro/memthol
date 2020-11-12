//! Generic stuff over durations.

prelude! { time::* }

/// Adds functionalities to the [`Duration`] type.
///
/// [`Duration`]: https://doc.rust-lang.org/std/time/struct.Duration.html
/// (Duration on Rust std)
pub trait DurationExt: From<Duration> {
    /// Retrieves the duration from `Self`.
    fn as_duration(&self) -> &Duration;

    /// Retrieves the chrono duration from `Self`.
    fn to_chrono_duration(&self) -> chrono::Duration {
        let duration = *self.as_duration();
        chrono::Duration::from_std(duration)
            .expect("error while converting duration from std to a chrono duration")
    }

    /// Creates a duration from a timestamp in microseconds.
    fn from_micros(ts: u64) -> Self {
        let secs = ts / 1_000_000;
        let micros = ts - (secs * 1_000_000);
        std::time::Duration::new(secs, convert(micros, "duration_from_micros: micros")).into()
    }

    /// Duration parser from an amount of seconds, seen as a float.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use base::prelude::time::{Duration, DurationExt};
    /// let s_list = vec![
    ///     ("320.74", Duration::new(320, 740_000_000)),
    ///     ("703470.0074", Duration::new(703470, 7_400_000)),
    ///     ("0.2", Duration::new(0, 200_000_000)),
    ///     ("7.0", Duration::new(7, 0)),
    ///     (".003", Duration::new(0, 3_000_000)),
    ///     ("42", Duration::new(42, 0)),
    /// ];
    /// for (s, exp) in s_list {
    ///     let duration = Duration::parse_secs(s).unwrap();
    ///     assert_eq! { duration, exp }
    /// }
    /// ```
    fn parse_secs<Str>(ts: &Str) -> Res<Self>
    where
        Str: ?Sized + AsRef<str>,
    {
        let ts = ts.as_ref();
        let mut subs = ts.split('.');

        macro_rules! err {
            (bail $($stuff:tt)*) => {
                return Err(err!(chain crate::err::Error::from(
                    format!($($stuff)*)
                )));
            };
            (try $e:expr) => {
                err!(chain $e)?
            };
            (chain $e:expr) => {
                $e.chain_err(|| format!("while parsing `{}` as an amount of seconds (float)", ts))
            }
        }

        let duration = match (subs.next(), subs.next()) {
            (Some(secs_str), None) => {
                let secs = err! { try u64::from_str(secs_str) };
                Duration::new(secs, 0)
            }

            (Some(secs_str), Some(mut subsecs_str)) => {
                let original_subsecs_str_len = subsecs_str.len();
                while subsecs_str.len() > 1 {
                    if &subsecs_str[0..1] == "0" {
                        subsecs_str = &subsecs_str[1..]
                    } else {
                        break;
                    }
                }

                let secs = if secs_str.is_empty() {
                    0
                } else {
                    err! { try u64::from_str(secs_str) }
                };

                let nanos = if subsecs_str.is_empty() && !secs_str.is_empty() {
                    0
                } else {
                    let raw = err! { try u32::from_str(subsecs_str) };

                    if original_subsecs_str_len < 9 {
                        raw * 10u32.pow(9 - (original_subsecs_str_len as u32))
                    } else if original_subsecs_str_len == 9 {
                        raw
                    } else {
                        err!(bail
                            "illegal sub-second decimal: \
                            precision above nanoseconds is not supported"
                        )
                    }
                };
                Duration::new(secs, nanos)
            }
            (None, _) => unreachable!("`str::split` never returns an empty iterator"),
        };

        if subs.next().is_some() {
            err!(bail "found more than one `.` character")
        }

        Ok(duration.into())
    }

    /// Pretty displayable version of a duration, millisecond precision.
    fn display_millis<'me>(&'me self) -> DurationDisplay<'me, Self, Millis> {
        self.into()
    }
    /// Pretty displayable version of a duration, microsecond precision.
    fn display_micros<'me>(&'me self) -> DurationDisplay<'me, Self, Micros> {
        self.into()
    }
    /// Pretty displayable version of a duration, nanosecond precision.
    fn display_nanos<'me>(&'me self) -> DurationDisplay<'me, Self, Nanos> {
        self.into()
    }
}

impl DurationExt for Duration {
    fn as_duration(&self) -> &Self {
        self
    }
}

/// Trait implemented by unit-structs representing time precision.
pub trait TimePrecision {
    /// Formats a duration with a given precision.
    fn duration_fmt(duration: &Duration, fmt: &mut fmt::Formatter) -> fmt::Result;
}

/// Nanosecond precision.
pub struct Nanos;
impl TimePrecision for Nanos {
    fn duration_fmt(duration: &Duration, fmt: &mut fmt::Formatter) -> fmt::Result {
        let duration = duration.as_duration();
        write!(
            fmt,
            "{}.{:0>9}",
            duration.as_secs(),
            duration.subsec_nanos()
        )
    }
}

/// Microsecond precision.
pub struct Micros;
impl TimePrecision for Micros {
    fn duration_fmt(duration: &Duration, fmt: &mut fmt::Formatter) -> fmt::Result {
        let duration = duration.as_duration();
        write!(
            fmt,
            "{}.{:0>6}",
            duration.as_secs(),
            duration.subsec_micros()
        )
    }
}

/// Millisecond precision
pub struct Millis;
impl TimePrecision for Millis {
    fn duration_fmt(duration: &Duration, fmt: &mut fmt::Formatter) -> fmt::Result {
        let duration = duration.as_duration();
        write!(
            fmt,
            "{}.{:0>3}",
            duration.as_secs(),
            duration.subsec_millis()
        )
    }
}

/// Thin wrapper around a reference to a duration.
pub struct DurationDisplay<'a, T: DurationExt + ?Sized, Precision: TimePrecision> {
    /// The actual duration.
    duration: &'a T,
    /// Phantom data for the precision.
    _phantom: std::marker::PhantomData<Precision>,
}

impl<'a, T: DurationExt + ?Sized> From<&'a T> for DurationDisplay<'a, T, Nanos> {
    fn from(duration: &'a T) -> Self {
        Self {
            duration,
            _phantom: std::marker::PhantomData,
        }
    }
}
impl<'a, T: DurationExt + ?Sized> From<&'a T> for DurationDisplay<'a, T, Micros> {
    fn from(duration: &'a T) -> Self {
        Self {
            duration,
            _phantom: std::marker::PhantomData,
        }
    }
}
impl<'a, T: DurationExt + ?Sized> From<&'a T> for DurationDisplay<'a, T, Millis> {
    fn from(duration: &'a T) -> Self {
        Self {
            duration,
            _phantom: std::marker::PhantomData,
        }
    }
}
impl<T: DurationExt, Precision: TimePrecision> fmt::Display for DurationDisplay<'_, T, Precision> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        Precision::duration_fmt(self.duration.as_duration(), fmt)
    }
}
