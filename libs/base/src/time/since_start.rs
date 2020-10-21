//! A thin wrapper around a duration.

prelude! { time::* }

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

impl SinceStart {
    /// A duration of 0 nanoseconds.
    pub fn zero() -> Self {
        SinceStart {
            duration: std::time::Duration::new(0, 0),
        }
    }
    /// A duration of 1 second.
    pub fn one_sec() -> Self {
        SinceStart {
            duration: std::time::Duration::new(1, 0),
        }
    }

    /// Constructor from a timestamp in nanos seconds.
    pub fn from_nano_timestamp(secs: u64, nanos: u32) -> Self {
        Self {
            duration: Duration::new(secs, nanos),
        }
    }

    /// True if the duration is zero.
    pub fn is_zero(&self) -> bool {
        self.duration.subsec_nanos() == 0 && self.duration.as_secs() == 0
    }

    /// Turns itself in a lifetime.
    pub fn to_lifetime(self) -> Lifetime {
        Lifetime::from(self.duration)
    }
}

impl DurationExt for SinceStart {
    fn as_duration(&self) -> &Duration {
        &self.duration
    }
}

implement! {
    impl SinceStart {
        Display {
            |&self, fmt| {
                self.display_micros().fmt(fmt)
            }
        }

        From {
            from Duration => |duration| Self { duration }
        }
        Into {
            to Duration => |self| self.duration
        }

        Deref {
            to Duration => |&self| &self.duration
        }
    }
}

impl ops::Sub for SinceStart {
    type Output = Self;
    fn sub(mut self, other: Self) -> Self {
        self.duration = self.duration - other.duration;
        self
    }
}
impl<'a> ops::Sub<SinceStart> for &'a SinceStart {
    type Output = SinceStart;
    fn sub(self, mut other: SinceStart) -> SinceStart {
        other.duration = self.duration - other.duration;
        other
    }
}
impl<'a> ops::Sub<&'a SinceStart> for SinceStart {
    type Output = SinceStart;
    fn sub(mut self, other: &'a SinceStart) -> SinceStart {
        self.duration = self.duration - other.duration;
        self
    }
}
impl<'a, 'b> ops::Sub<&'a SinceStart> for &'b SinceStart {
    type Output = SinceStart;
    fn sub(self, other: &'a SinceStart) -> SinceStart {
        SinceStart {
            duration: self.duration - other.duration,
        }
    }
}

impl ops::Add for SinceStart {
    type Output = Self;
    fn add(mut self, other: Self) -> Self {
        self.duration = self.duration + other.duration;
        self
    }
}
impl<'a> ops::Add<SinceStart> for &'a SinceStart {
    type Output = SinceStart;
    fn add(self, mut other: SinceStart) -> SinceStart {
        other.duration = self.duration + other.duration;
        other
    }
}
impl<'a> ops::Add<&'a SinceStart> for SinceStart {
    type Output = SinceStart;
    fn add(mut self, other: &'a SinceStart) -> SinceStart {
        self.duration = self.duration + other.duration;
        self
    }
}
impl<'a, 'b> ops::Add<&'a SinceStart> for &'b SinceStart {
    type Output = SinceStart;
    fn add(self, other: &'a SinceStart) -> SinceStart {
        SinceStart {
            duration: self.duration + other.duration,
        }
    }
}

impl ops::Div<u32> for SinceStart {
    type Output = Self;
    fn div(mut self, ratio: u32) -> Self {
        self.duration = self.duration / ratio;
        self
    }
}
impl<'a> ops::Div<u32> for &'a SinceStart {
    type Output = SinceStart;
    fn div(self, ratio: u32) -> SinceStart {
        let mut slf = *self;
        slf.duration = self.duration / ratio;
        slf
    }
}
