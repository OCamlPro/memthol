//! Automatic filter generation.
//!
//! This module stores a global filter-generation configuration of type [`FilterGen`]. It stores the
//! parameters (if any) for an actual filter-generation strategy. The following functions allow to
//! interact with this global configuration: [`get`], [`set`], and [`set_from_cla`].
//!
//! # Filter Generation Strategies
//!
//! Strategies are defined in sub-modules such as [`inactive`] and [`alloc_site`]. They typically
//! define a unit-struct (*e.g.* [`AllocSite`]) implementing the [`FilterGenExt` trait][ext] which
//! defines a parameter type [`Params`] among other things.
//!
//! [`FilterGen`] features a variant for each strategy that stores the parameters for the strategy,
//! so that it can run it when asked to. Note that strategies also define how they can be activated
//! and controlled by implementing [`FilterGenExt`][ext].
//!
//! > **NB**: when creating a new filter generator, make sure you activate it by updating the
//! > `all_gens` private macro in this module. Overall, you should take inspiration from
//! > [`alloc_site`] when adding a new generator.
//!
//! Filter generation also handles *chart generation*, which is is the [`chart_gen` module].
//!
//! [`FilterGen`]: enum.FilterGen.html (FilterGen enum)
//! [`get`]: fn.get.html (get function)
//! [`set`]: fn.set.html (set function)
//! [`set_from_cla`]: fn.set_from_cla.html (set_from_cla function)
//! [`inactive`]: ./inactive (inactive module)
//! [`alloc_site`]: ./alloc_site (alloc_site module)
//! [`AllocSite`]: ./alloc_site/struct.AllocSite.html (AllocSite struct)
//! [ext]: trait.FilterGenExt.html (FilterGenExt trait)
//! [`Params`]: trait.FilterGenExt.html#associatedtype.Params (FilterGenExt trait)
//! [`chart_gen` module]: ./chart_gen (chart_gen module)

prelude! {}

pub mod parser;

pub mod alloc_site;
pub mod chart_gen;
pub mod inactive;

use self::{
    alloc_site::{AllocSite, AllocSiteParams},
    inactive::Inactive,
    parser::Parser,
};

/// Retrieves the active filter generator.
pub fn get() -> FilterGen {
    ACTIVE_GEN
        .read()
        .expect("global active filter generator was poisoned")
        .clone()
}

/// Sets the active filter generator.
pub fn set(gen: impl Into<FilterGen>) {
    let mut active = ACTIVE_GEN
        .write()
        .expect("global active filter generator was poisoned");
    *active = gen.into();
}

/// Sets the active filter generator from a command-line argument.
///
/// See [`Filtergen::from_cla`][from_cla] for details.
///
/// [from_cla]: enum.FilterGen.html#method.from_cla (from_cla method on FilterGen)
pub fn set_from_cla(args: &str) -> Res<()> {
    let gen = FilterGen::from_cla(args)
        .chain_err(|| format!("while parsing filter-gen argument `{}`", args))?;
    set(gen);
    Ok(())
}

lazy_static! {
    /// Stores the active filter generator.
    ///
    /// This is currently written once during CLAP.
    static ref ACTIVE_GEN: sync::RwLock<FilterGen> =
        sync::RwLock::new(FilterGen::default());
}

/// Enumeration of the filter generation techniques.
///
/// Stores parameters for a given filter generator so that it can run it with [`run`].
///
/// [`run`]: #method.run (run method)
#[derive(Debug, Clone)]
pub enum FilterGen {
    /// Generate one allocation filter per allocation site.
    AllocSite(AllocSiteParams),
    /// No filter generation.
    Inactive,
}
impl From<AllocSiteParams> for FilterGen {
    fn from(params: AllocSiteParams) -> Self {
        Self::AllocSite(params)
    }
}

impl Default for FilterGen {
    fn default() -> Self {
        Self::AllocSite(AllocSiteParams::default())
    }
}

/// Constructs arrays over some treatment of all the filter generators.
///
/// When writing a new filter generator, you need to update this so that `FilterGen` knows about it.
macro_rules! all_gens {
    // Generates an array `[ ... ]` token tree. Its elements are the names of the generators, each
    // with the input prefix/suffix before/after the name.
    (pref($($pref:tt)*) suff($($suff:tt)*)) => {
        [
            $($pref)* Inactive $($suff)*,
            $($pref)* AllocSite $($suff)*,
        ]
    };
    // Generates an array `[ ... ]` token tree. Its elements are the names of the generators, each
    // with the input suffix after the name.
    ($($suff:tt)*) => {
        all_gens!(pref() suff($($suff)*))
    };
}

impl FilterGen {
    /// Runs the filter generator represented by `self` on some data.
    pub fn run(self, data: &data::Data) -> Res<(Filters, Vec<chart::Chart>)> {
        match self {
            Self::AllocSite(params) => AllocSite::work(data, params),
            Self::Inactive => Inactive::work(data, ()),
        }
    }

    /// List of all the filter-generator keys.
    const KEYS: &'static [&'static str] = &all_gens!(::KEY);

    /// Generates a filter-generator key error.
    ///
    /// Used on unknown filter-generator keys.
    fn key_err() -> String {
        let mut keys = "argument must start with a legal key among ".to_string();
        for (idx, key) in Self::KEYS.iter().enumerate() {
            if idx > 0 {
                keys.push_str(", ")
            }
            keys.push('`');
            keys.push_str(key);
            keys.push('`');
        }
        keys
    }

    /// Generates the help message for the `--filter_gen` flag.
    pub fn help() -> String {
        let mut s = format!(
            "\
When memthol launches, it runs a *filter generator* that looks at all the data and generates
filters that (hopefully) help you better visualize your data. Use the `--filter_gen` flag to choose
among several generators and parameterize them.
(Show this help message at any time with `--filter_gen help`.)

This flag takes a string argument, which has shapes
- `<gen> {{ <params> }}`: use generator `<gen>` with parameters `<params>`, or
- `<gen>`: use generator `<gen>` in its default mode.

The different generators are

\
            ",
        );

        for add_help in all_gens!(::add_help).iter() {
            add_help(&mut s)
        }

        s
    }

    /// Generates itself from arguments for the `--filter_gen` flag.
    ///
    /// The expected format of the arguments is `<gen_key> { <gen_params> }` or just `<gen_key>`,
    /// where
    ///
    /// - `<gen_key>` must be a [`KEY`] identifier corresponding to one of the generators, and
    /// - `<gen_params>`, if any, is a generator-specific parameter specification; this
    ///   specification generally looks like a comma-separated sequence of bindings of the form
    ///   `<id>: <value>` described by the generator's [`FMT`] format.
    ///
    /// [`KEY`]: trait.FilterGenExt.html#associatedconstant.KEY
    /// (KEY constant on the FilterGenExt trait)
    /// [`FMT`]: trait.FilterGenExt.html#associatedconstant.FMT
    /// (FMT constant on the FilterGenExt trait)
    pub fn from_cla(args: &str) -> Res<Self> {
        let mut parser = Parser::new(args.trim());

        let key = parser.ident().ok_or_else(Self::key_err)?;

        macro_rules! err {
            (key: $key:expr, fmt: $fmt:expr) => {
                if let Some(fmt) = $fmt {
                    format!(
                        "`{}` filter expects its arguments to have form `{}`",
                        $key, fmt
                    )
                } else {
                    format!("`{}` filter expects no arguments", $key)
                }
            };
        }

        parser.ws();
        let inner_parser: Option<Parser> = parser.block()?;
        parser.ws();

        if !parser.is_at_eoi() {
            if inner_parser.is_some() {
                bail!(
                    "unexpected trailing characters after block of parameters for `{}`",
                    key
                )
            } else {
                bail!(
                    "expected block `{{ ... }}` of parameters or nothing after key `{}`",
                    key
                )
            }
        }

        for ((gen_key, parse), gen_fmt) in all_gens!(::KEY)
            .iter()
            .zip(all_gens!(::parse_args).iter())
            .zip(all_gens!(::FMT).iter())
        {
            if key == *gen_key {
                return parse(inner_parser).ok_or_else(|| err!(key: gen_key, fmt: gen_fmt).into());
            }
        }

        bail!("unexpected key `{}`, {}", key, Self::key_err())
    }
}

/// Trait implemented by filter generation techniques.
///
/// This trait represents the info needed by [`FilterGen`] to
///
/// - [`parse_args`], [`KEY`]: parse the arguments of the `--filter_gen` flag,
/// - [`add_help`]: generate the part of the help message for `--filter_gen` specific to this filter
///   generator,
/// - [`KEY`], [`FMT`]: generate relevant error messages during CLAP, and
/// - [`work`] actually run the generator.
///
/// [`FilterGen`]: enum.FilterGen.html (FilterGen enum)
/// [`parse_args`]: #tymethod.parse_args (parse_args abstract method)
/// [`KEY`]: #associatedconstant.KEY (KEY abstract constant)
/// [`add_help`]: #tymethod.add_help (add_help abstract method)
/// [`FMT`]: #associatedconstant.FMT (FMT abstract constant)
/// [`work`]: #tymethod.work (work abstract method)
pub trait FilterGenExt {
    /// Type of the parameters of the filter generator.
    type Params: Default;

    /// CLAP key activating this generator.
    const KEY: &'static str;
    /// Optional parameter format, should be a list of comma-separated `<id>: <value>` bindings.
    const FMT: Option<&'static str>;

    /// Parses the (potentialy optional) parameters for the generator.
    ///
    /// Simply returns `None` if parameters are ill-formed. Caller's responsible for error
    /// reporting.
    fn parse_args(parser: Option<Parser>) -> Option<FilterGen>;

    /// Adds help about itself to a `String`.
    ///
    /// See [`AllocSite`]'s implementation of this function to get an idea of the format.
    ///
    /// [`AllocSite`]: ./alloc_site/struct.AllocSite.html (AllocSite struct)
    fn add_help(s: &mut String);

    /// Runs the generator on some data given some parameters.
    fn work(data: &data::Data, params: Self::Params) -> Res<(Filters, Vec<chart::Chart>)>;
}
