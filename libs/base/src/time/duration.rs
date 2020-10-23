//! Generic stuff over durations.

prelude! { time::* }

/// Adds functionalities to the [`Duration`] type.
///
/// [`Duration`]: https://doc.rust-lang.org/std/time/struct.Duration.html
/// (Duration on Rust std)
pub trait DurationExt: From<Duration> {
    /// Retrieves the duration from `Self`.
    fn as_duration(&self) -> &Duration;

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

        println!();
        println!("ts: `{}`", ts);
        let duration = match (subs.next(), subs.next()) {
            (Some(secs_str), None) => {
                println!("case 1");
                println!("- secs_str: `{}`", secs_str);
                let secs = err! { try u64::from_str(secs_str) };
                println!("- secs: {}", secs);
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

                println!("case 2");
                println!("- secs_str: `{}`", secs_str);
                println!("- subsecs_str: `{}`", subsecs_str);

                let secs = if secs_str.is_empty() {
                    0
                } else {
                    err! { try u64::from_str(secs_str) }
                };

                println!("- secs: {}", secs);

                let nanos = if subsecs_str.is_empty() && !secs_str.is_empty() {
                    0
                } else {
                    let raw = err! { try u32::from_str(subsecs_str) };

                    println!("- raw: {}", raw);
                    println!("- original_subsecs_str_len: {}", original_subsecs_str_len);

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
                println!("- nanos: {}", nanos);
                Duration::new(secs, nanos)
            }
            (None, _) => unreachable!("`str::split` never returns an empty iterator"),
        };

        if subs.next().is_some() {
            err!(bail "found more than one `.` character")
        }

        Ok(duration.into())
    }

    /// Pretty displayable version of a duration, microsecond precision.
    fn display_micros<'me>(&'me self) -> DurationMilliDisplay<'me, Self> {
        self.into()
    }
    /// Pretty displayable version of a duration, nanosecond precision.
    fn display_nanos<'me>(&'me self) -> DurationNanoDisplay<'me, Self> {
        self.into()
    }
}

impl DurationExt for Duration {
    fn as_duration(&self) -> &Self {
        self
    }
}

/// Wraps a duration representation, provides `Display` with nanosecond precision.
pub struct DurationNanoDisplay<'a, T: DurationExt + ?Sized> {
    duration: &'a T,
}
impl<'a, T: DurationExt + ?Sized> From<&'a T> for DurationNanoDisplay<'a, T> {
    fn from(duration: &'a T) -> Self {
        Self { duration }
    }
}
impl<T: DurationExt> fmt::Display for DurationNanoDisplay<'_, T> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        let duration = self.duration.as_duration();
        write!(
            fmt,
            "{}.{:0>9}",
            duration.as_secs(),
            duration.subsec_nanos()
        )
    }
}

/// Wraps a duration representation, provides `Display` with microsecond precision.
pub struct DurationMilliDisplay<'a, T: DurationExt + ?Sized> {
    duration: &'a T,
}
impl<'a, T: DurationExt + ?Sized> From<&'a T> for DurationMilliDisplay<'a, T> {
    fn from(duration: &'a T) -> Self {
        Self { duration }
    }
}
impl<T: DurationExt> fmt::Display for DurationMilliDisplay<'_, T> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        let duration = self.duration.as_duration();
        write!(
            fmt,
            "{}.{:0>6}",
            duration.as_secs(),
            duration.subsec_micros()
        )
    }
}
