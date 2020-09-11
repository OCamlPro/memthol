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
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Serialize, Deserialize)]
pub struct Color {
    /// Red component.
    pub r: u8,
    /// Green component.
    pub g: u8,
    /// Blue component.
    pub b: u8,
}

impl plotters_backend::BackendStyle for Color {
    fn color(&self) -> plotters_backend::BackendColor {
        plotters_backend::BackendColor {
            alpha: 1.0,
            rgb: (self.r, self.g, self.b),
        }
    }
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
    pub fn to_plotters(&self) -> palette::rgb::Rgb<palette::encoding::srgb::Srgb, u8> {
        palette::rgb::Rgb::new(self.r, self.g, self.b)
    }

    pub fn from_hue(hue: f32, saturation: f32, lightness: f32) -> Self {
        // Dumb application of https://en.wikipedia.org/wiki/HSL_and_HSV#Color_conversion_formulae.
        let hue = hue % 360.;
        let saturation = if saturation < 0.0 {
            0.0
        } else if 1.0 < saturation {
            1.0
        } else {
            saturation
        };
        let lightness = if lightness < 0.0 {
            0.0
        } else if 1.0 < lightness {
            1.0
        } else {
            lightness
        };

        let first_chroma = (1.0 - (2.0 * lightness - 1.).abs()) * saturation;
        let hue_prime = hue / 60.;
        let x = first_chroma * (1. - ((hue_prime % 2.) - 1.).abs());

        let (r, g, b) = if hue_prime <= 1. {
            (first_chroma, x, 0.)
        } else if hue_prime <= 2. {
            (x, first_chroma, 0.)
        } else if hue_prime <= 3. {
            (0., first_chroma, x)
        } else if hue_prime <= 4. {
            (0., x, first_chroma)
        } else if hue_prime <= 5. {
            (x, 0., first_chroma)
        } else if hue_prime <= 6. {
            (first_chroma, 0., x)
        } else {
            panic!("illegal `hue_prime` value {}", hue_prime)
        };

        let m = lightness - (first_chroma / 2.);

        let (r, g, b) = ((r + m) * 255., (g + m) * 255., (b + m) * 255.);
        let (r, g, b) = (r as u8, g as u8, b as u8);

        Self { r, g, b }
    }

    /// Constructs `n` random colors evenly spread on the color wheel.
    ///
    /// - the starting point on the color wheel is random.
    ///
    /// # Guarantees
    ///
    /// - `Color::randoms(n).len() == n`
    pub fn randoms(n: usize) -> Vec<Self> {
        if n == 0 {
            return vec![];
        }

        let inc = 360. / (n as f32);
        let mut current = rng!().gen::<f32>() * 360f32;

        (0..n)
            .into_iter()
            .map(|_| {
                let color = Self::from_hue(current, 1.0, 0.5);
                current += inc;
                color
            })
            .collect()
    }

    /// Constructs a random color.
    ///
    /// - `dark` indicates whether the random color should be relatively dark.
    pub fn random() -> Self {
        Self::from_hue(rng!().gen::<f32>() * 360f32, 1.0, 0.5)
    }

    /// Keeps on constructing colors until the input predicate is true.
    pub fn random_until(pred: impl Fn(&Color) -> bool) -> Self {
        let mut color = Self::random();
        while !pred(&color) {
            color = Self::random()
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
