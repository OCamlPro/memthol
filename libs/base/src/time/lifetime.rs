//! A lifetime is a thin wrapper around a duration.

prelude! { time::* }

/// Wrapper around a duration representing a lifetime.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Lifetime {
    /// Actual duration.
    duration: Duration,
}

impl Lifetime {
    /// A duration of 0 nanoseconds.
    pub fn zero() -> Self {
        Lifetime {
            duration: std::time::Duration::new(0, 0),
        }
    }
}

impl DurationExt for Lifetime {
    fn as_duration(&self) -> &std::time::Duration {
        &self.duration
    }
}

implement! {
    impl Lifetime {
        Display {
            |&self, fmt| {
                self.display_micros().fmt(fmt)
            }
        }

        Default {
            Self { duration: Duration::default() }
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
