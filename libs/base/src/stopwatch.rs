//! Stopwatch, for time statistics.

use std::fmt;

use std::time::{Duration, Instant};

/// Stopwatch.
#[derive(Debug, Clone)]
pub struct RealStopwatch {
    /// Remember the time counted before the last start, if any.
    elapsed: Duration,
    /// Instant of the last start order not followed by a stop order.
    last_start: Option<Instant>,
}

impl fmt::Display for RealStopwatch {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        let elapsed = self.elapsed();
        let (secs, subsec_nanos) = (elapsed.as_secs(), elapsed.subsec_nanos());
        if subsec_nanos == 0 {
            write!(fmt, "{}s", secs)
        } else {
            write!(fmt, "{}.{:0>9}s", secs, subsec_nanos)
        }
    }
}

/// Stopwatch.
#[derive(Debug, Clone)]
pub struct FakeStopwatch;

impl fmt::Display for FakeStopwatch {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        "???".fmt(fmt)
    }
}

macro_rules! fn_defs {
    ($(
        $(#[$fn_meta:meta])*
        $fn_vis:vis fn $fn_id:ident
            $(<$($t_params:ident),* $(,)?>)?
            ( $($fn_args:tt)* ) $(-> $fn_out:ty)?
        {
            $($profiling_def:tt)*
        } {
            $($not_profiling_def:tt)*
        }
    )*) => {
        impl RealStopwatch {
            /// True if we are profiling.
            pub const TIME_STATS_ACTIVE: bool = true;
            $(
                $(#[$fn_meta])*
                #[inline]
                $fn_vis fn $fn_id $(<$($t_params),*>)? ($($fn_args)*) $(-> $fn_out)? {
                    $($profiling_def)*
                }
            )*
        }

        impl FakeStopwatch {
            /// True if we are profiling.
            pub const TIME_STATS_ACTIVE: bool = false;
            $(
                $(#[$fn_meta])*
                #[inline]
                $fn_vis fn $fn_id $(<$($t_params),*>)? ($($fn_args)*) $(-> $fn_out)? {
                    $($not_profiling_def)*
                }
            )*
        }
    }
}

impl RealStopwatch {
    /// Applies an action to the time counted so far.
    pub fn elapsed(&self) -> Duration {
        let mut duration = self.elapsed.clone();
        if let Some(last_start) = self.last_start {
            duration += Instant::now() - last_start
        }
        duration
    }
}

fn_defs! {
    /// Builds a stopped stopwatch.
    pub fn new() -> Self {
        Self {
            elapsed: Duration::new(0, 0),
            last_start: None,
        }
    } {
        Self
    }

    /// Build a running stopwatch.
    pub fn start_new() -> Self {
        let mut slf = Self::new();
        slf.last_start = Some(Instant::now());
        slf
    } {
        Self
    }

    /// Starts a stopwatch. Does nothing if already running.
    pub fn start(&mut self) {
        if self.last_start.is_none() {
            self.last_start = Some(Instant::now())
        }
        debug_assert!(self.last_start.is_some())
    } {}

    /// Stops a stopwatch. Does nothing if already stopped.
    pub fn stop(&mut self) {
        if let Some(last_start) = std::mem::replace(&mut self.last_start, None) {
            self.elapsed += Instant::now() - last_start
        }
        debug_assert_eq!(self.last_start, None)
    } {}

    /// True if the stopwatch is running.
    pub fn is_running(&self) -> bool {
        self.last_start.is_some()
    } { false }

    /// Resets a stopwatch. Preserves the fact that it is running or not.
    pub fn reset(&mut self) {
        let running = self.is_running();
        *self = Self::new();
        if running {
            self.start()
        }
    } {}

    /// Times some action if not currently running.
    pub fn time<Out>(&mut self, f: impl FnOnce() -> Out) -> Out {
        if !self.is_running() {
            self.start();
            let res = f();
            self.stop();
            res
        } else {
            f()
        }
    } { f() }
}

#[macro_export]
macro_rules! new_time_stats {
    (
        $(#[$ty_meta:meta])*
        $ty_vis:vis struct $ty_name:ident {$(
            $(#[$field_meta:meta])*
            $field_vis:vis $field_name:ident => $field_desc:expr,
        )*}
    ) => {
        $(#[$ty_meta])*
        $ty_vis struct $ty_name {$(
            $(#[$field_meta])*
            #[cfg(any(test, feature = "time_stats"))]
            $field_vis $field_name: $crate::stopwatch::RealStopwatch,

            $(#[$field_meta])*
            #[cfg(not(any(test, feature = "time_stats")))]
            $field_vis $field_name: $crate::stopwatch::FakeStopwatch,
        )*}

        impl $ty_name {
            /// Constructor.
            #[cfg(any(test, feature = "time_stats"))]
            pub fn new() -> Self {
                Self {$(
                    $field_name: $crate::stopwatch::RealStopwatch::new(),
                )*}
            }
            /// Constructor.
            #[cfg(not(any(test, feature = "time_stats")))]
            pub fn new() -> Self {
                Self {$(
                    $field_name: $crate::stopwatch::FakeStopwatch::new(),
                )*}
            }

            /// Resets all the stopwatches.
            pub fn reset(&mut self) {
                $(
                    self.$field_name.reset();
                )*
            }

            /// True if we are profiling.
            pub const TIME_STATS_ACTIVE: bool = cfg!(any(test, feature = "time_stats"));

            /// Iterates over all stopwatches.
            #[cfg(any(test, feature = "time_stats"))]
            pub fn all_do(
                &self,
                first_do: impl FnOnce(),
                mut action: impl FnMut(&'static str, &$crate::stopwatch::RealStopwatch)
            ) {
                first_do();
                $(
                    action($field_desc, &self.$field_name);
                )*
            }
            /// Iterates over all stopwatches.
            #[cfg(not(any(test, feature = "time_stats")))]
            #[inline]
            pub fn all_do(
                &self,
                _: impl FnOnce(),
                _: impl FnMut(&'static str, &$crate::stopwatch::FakeStopwatch)
            ) {
            }
        }
        impl std::fmt::Display for $ty_name {
            fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
                #![allow(unused_assignments)]

                #[allow(unused_mut)]
                let mut pref = "";

                $(
                    write!(
                        fmt, "{}{}: {}",
                        pref,
                        $field_desc,
                        self.$field_name
                    )?;
                    pref = ", ";
                )*
                Ok(())
            }
        }
    };
}

#[cfg(test)]
#[allow(dead_code)]
mod test {
    new_time_stats! {
        /// Profiler.
        pub struct Profiler {
            pub loading => "loading",
            pub parsing => "parsing",
            pub communication => "communication",
        }
    }

    #[test]
    fn basics() {
        let mut profiler = Profiler::new();

        profiler.loading.start();
        profiler.communication.start();
        profiler.loading.reset();
        profiler.communication.stop();
        profiler.loading.stop();

        println!(
            "loading: {}, parsing: {}, communication: {}",
            profiler.loading, profiler.parsing, profiler.communication
        )
    }
}
