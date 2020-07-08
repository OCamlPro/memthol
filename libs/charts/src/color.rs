//! Color handling.

prelude! {}

use std::sync::RwLock;

pub use base::rand::{
    rngs::SmallRng,
    {Rng, SeedableRng},
};

lazy_static! {
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

/// RGBA color.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct Color {
    /// Red component.
    pub r: u8,
    /// Green component.
    pub g: u8,
    /// Blue component.
    pub b: u8,
}

impl plotters::style::Color for Color {
    fn rgb(&self) -> (u8, u8, u8) {
        (self.r, self.g, self.b)
    }
    fn alpha(&self) -> f64 {
        1.0
    }
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
    /// # use charts::color::Color;
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
    /// # use charts::color::Color;
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

    /// Turns itself in a `plotters`-compliant color.
    pub fn to_plotters(
        &self,
    ) -> plotters::palette::rgb::Rgb<plotters::palette::encoding::srgb::Srgb, u8> {
        plotters::palette::rgb::Rgb::new(self.r, self.g, self.b)
    }

    /// Constructs a random color.
    ///
    /// - `dark` indicates whether the random color should be relatively dark.
    pub fn random(dark: bool) -> Self {
        let mut rng = rng!();
        let mut values = Vec::with_capacity(3);
        // Generate a first hi value.
        values.push(Self::random_hi_u8(&mut rng));
        // Decide wether the next value is high too.
        let v_2_hi = rng.gen::<bool>();
        // Generate a second value.
        values.push(if v_2_hi {
            Self::random_hi_u8(&mut rng)
        } else {
            Self::random_mid_u8(&mut rng)
        });
        // Decide wether the next value is high too.
        let v_3_hi = (!v_2_hi || !dark) && rng.gen::<bool>();
        // Generate a third value.
        values.push(if v_3_hi {
            Self::random_hi_u8(&mut rng)
        } else {
            Self::random_low_u8(&mut rng)
        });

        // Shuffle stuff around.
        use base::rand::seq::SliceRandom;
        values.shuffle(&mut *rng);

        Self {
            r: values[0],
            g: values[1],
            b: values[2],
        }
    }
    /// A random u8 between a lower-bound and an upper-bound.
    ///
    /// Panics if `lb >= ub`.
    fn random_u8(rng: &mut SmallRng, lb: u8, ub: u8) -> u8 {
        if lb >= ub {
            panic!("illegal call `Color::random_u8(_, {}, {})`", lb, ub)
        }
        let mod_val = ub - lb + 1;
        rng.gen::<u8>() % mod_val + lb
    }
    /// A random u8 between `200` and `255`.
    fn random_hi_u8(rng: &mut SmallRng) -> u8 {
        Self::random_u8(rng, 200, 255)
    }
    /// A random u8 between `100` and `200`.
    fn random_mid_u8(rng: &mut SmallRng) -> u8 {
        Self::random_u8(rng, 100, 200)
    }
    /// A random u8 between `50` and `150`.
    fn random_low_u8(rng: &mut SmallRng) -> u8 {
        Self::random_u8(rng, 50, 150)
    }

    /// Keeps on constructing colors until the input predicate is true.
    pub fn random_until(dark: bool, pred: impl Fn(&Color) -> bool) -> Self {
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
