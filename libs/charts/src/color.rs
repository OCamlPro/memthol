//! Color handling.

use std::sync::RwLock;

use crate::base::*;

pub use rand::{
    rngs::SmallRng,
    {Rng, SeedableRng},
};

lazy_static::lazy_static! {
    /// Color RNG.
    static ref RNG: RwLock<SmallRng> = RwLock::new(
        SmallRng::seed_from_u64(42u64)
    );
}

macro_rules! rng {
    () => {
        RNG.write().expect("failed to retrieve color RNG")
    };
}

/// RGB color.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Color {
    /// Red component.
    pub r: u8,
    /// Green component.
    pub g: u8,
    /// Blue component.
    pub b: u8,
}

impl fmt::Display for Color {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "#{:0>2x}{:0>2x}{:0>2x}", self.r, self.g, self.b)
    }
}

impl Color {
    /// Color constructor.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use charts::color::Color;
    /// let color = Color::new(0xff, 0x00, 0x00);
    /// assert_eq!(&color.to_string(), "#ff0000")
    /// ```
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    /// Constructs a color from a string.
    ///
    /// - only accepts RGB strings starting with `#`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use charts::color::Color;
    /// let color = Color::from_str("#ff0000").unwrap();
    /// assert_eq!(&color.to_string(), "#ff0000")
    /// ```
    pub fn from_str<Str: AsRef<str>>(text: Str) -> Res<Self> {
        let text = text.as_ref();

        macro_rules! fail {
            () => {
                bail!("illegal RGB color string `{}`", text)
            };
            ($e:expr) => {
                if let Ok(res) = $e {
                    res
                } else {
                    fail!()
                }
            };
        }

        if text.len() != 7 || &text[0..1] != "#" {
            fail!()
        }

        let (r, g, b) = (
            fail!(u8::from_str_radix(&text[1..3], 16)),
            fail!(u8::from_str_radix(&text[3..5], 16)),
            fail!(u8::from_str_radix(&text[5..7], 16)),
        );

        Ok(Self::new(r, g, b))
    }

    /// Constructs a random color.
    ///
    /// - `dark` indicates whether the random color should be relatively dark.
    pub fn random(dark: bool) -> Self {
        let mod_val = 156u8;
        let mut rng = rng!();
        let mut get = || rng.gen::<u8>() % mod_val + if dark { 0u8 } else { 100u8 };
        Self {
            r: get(),
            g: get(),
            b: get(),
        }
    }

    /// Keeps on constructing colors until the input predicate is true.
    pub fn random_until<Pred>(dark: bool, pred: Pred) -> Self
    where
        Pred: Fn(&Color) -> bool,
    {
        let mut color = Self::random(dark);
        while !pred(&color) {
            color = Self::random(dark)
        }
        color
    }

    /// Returns true if two colors are very similar.
    ///
    /// "Very similar" here means that all components are less than `15u8` apart.
    pub fn is_similar_to(&self, other: &Self) -> bool {
        macro_rules! check {
            ($lft:expr, $rgt:expr) => {
                15u8 >= if $lft >= $rgt {
                    $lft - $rgt
                } else {
                    $rgt - $lft
                }
            };
        }
        check!(self.r, other.r) && check!(self.g, other.g) && check!(self.b, other.b)
    }
}
