//! A window specifying the size of the chart.

pub use stdweb::js;

/// Some margins.
#[derive(Getters, Debug, Clone)]
pub struct Margin {
    /// Top margin.
    #[get = "pub with_prefix"]
    top: usize,
    /// Bottom margin.
    #[get = "pub with_prefix"]
    bot: usize,
    /// Left margin.
    #[get = "pub with_prefix"]
    lft: usize,
    /// Right margin.
    #[get = "pub with_prefix"]
    rgt: usize,
}
impl Default for Margin {
    fn default() -> Self {
        Self {
            top: 30,
            bot: 30,
            lft: 50,
            rgt: 50,
        }
    }
}

impl Margin {
    /// Setter for the top margin.
    pub fn top(mut self, top: usize) -> Self {
        self.top = top;
        self
    }
    /// Setter for the bottom margin.
    pub fn bottom(mut self, bottom: usize) -> Self {
        self.bot = bottom;
        self
    }
    /// Setter for the left margin.
    pub fn left(mut self, left: usize) -> Self {
        self.lft = left;
        self
    }
    /// Setter for the right margin.
    pub fn right(mut self, right: usize) -> Self {
        self.rgt = right;
        self
    }

    /// Transpose of the margin.
    pub fn get_transpose(&self) -> String {
        format!("translate({},{})", self.lft, self.top)
    }
}

/// A window: a width and a height.
#[derive(Getters, Debug, Clone)]
pub struct Window {
    /// Width.
    width: usize,
    /// Height.
    height: usize,
    /// Margin.
    margin: Margin,
}
impl Default for Window {
    fn default() -> Self {
        Self {
            width: 800,
            height: 400,
            margin: Margin::default(),
        }
    }
}

impl Window {
    /// Sets the width.
    pub fn set_width(mut self, width: usize) -> Self {
        self.width = width;
        self
    }
    /// Sets the height.
    pub fn set_height(mut self, height: usize) -> Self {
        self.height = height;
        self
    }

    /// Width accessor.
    pub fn width(&self) -> usize {
        self.width
    }
    /// Height accessor.
    pub fn height(&self) -> usize {
        self.height
    }
    /// Margin accessor.
    pub fn margin(&self) -> &Margin {
        &self.margin
    }

    /// Width including the margins.
    pub fn full_width(&self) -> usize {
        self.margin.lft + self.width + self.margin.rgt
    }
    /// Height including the margins.
    pub fn full_height(&self) -> usize {
        self.margin.top + self.height + self.margin.bot
    }

    /// Forces the window size on an SVG component and its inner component(s).
    pub fn force_on<Str, Comps>(&self, svg: &str, components: Comps)
    where
        Str: AsRef<str>,
        Comps: IntoIterator<Item = Str>,
    {
        js! {
            d3.select("." + @{svg})
                .attr(
                    "width",
                    @{self.full_width().to_string()}
                )
                .attr(
                    "height",
                    @{self.full_height().to_string()}
                )
        };
        for sub in components.into_iter() {
            let sub = sub.as_ref();
            js! {
                d3.select("." + @{sub})
                    .attr(
                        "transform",
                        "translate("
                        + @{self.margin.lft.to_string()}
                        + ","
                        + @{self.margin.top.to_string()}
                        + ")"
                    )
            }
        }
    }
}
