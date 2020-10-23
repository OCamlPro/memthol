//! Automatic filter generation.
//!
//!

prelude! {}

pub mod parser;

pub mod alloc_site;
pub mod none;

use self::{
    alloc_site::{AllocSite, AllocSiteParams},
    none::Inactive,
    parser::Parser,
};

macro_rules! all_gens {
    (pref($($pref:tt)*) suff($($suff:tt)*)) => {
        [
            $($pref)* Inactive $($suff)*,
            $($pref)* AllocSite $($suff)*,
        ]
    };
    ($($suff:tt)*) => {
        all_gens!(pref() suff($($suff)*))
    };
}

/// Retrieve the active filter generator.
pub fn get() -> FilterGen {
    ACTIVE_GEN
        .read()
        .expect("global active filter generator was poisoned")
        .clone()
}

/// Sets the active filter generator from a command-line argument.
pub fn set_from_cla(args: &str) -> Res<()> {
    let gen = FilterGen::from_args(args)
        .chain_err(|| format!("while parsing filter-gen argument `{}`", args))?;

    let mut active = ACTIVE_GEN
        .write()
        .map_err(|_| "global active filter generator was poisoned")?;
    *active = gen;

    Ok(())
}

lazy_static! {
    /// Stores the active filter generator.
    ///
    /// Written once during CLAP.
    static ref ACTIVE_GEN: sync::RwLock<FilterGen> =
        sync::RwLock::new(FilterGen::default());
}

/// Enumeration of the filter generation techniques.
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

impl FilterGen {
    pub fn run(self, data: &data::Data) -> Res<Vec<Filter>> {
        match self {
            Self::AllocSite(params) => AllocSite::work(data, params),
            Self::Inactive => Inactive::work(data, ()),
        }
    }

    const KEYS: &'static [&'static str] = &all_gens!(::KEY);

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

    pub fn help() -> String {
        let mut s = format!(
            "\
When memthol launches, it runs a *filter generator* that looks at all the data and generates
filters that (hopefully) help you better visualize your data. Use the `--filter_gen` flag to choose
among several generators and parameterize them.

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

    pub fn from_args(args: &str) -> Res<Self> {
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
pub trait FilterGenExt: Sized {
    type Params: Default;

    const KEY: &'static str;
    const FMT: Option<&'static str>;

    fn work(data: &data::Data, params: Self::Params) -> Res<Vec<Filter>>;

    fn parse_args(parser: Option<Parser>) -> Option<FilterGen>;

    fn add_help(s: &mut String);
}
