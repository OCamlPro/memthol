//! Automatic filter generation.

prelude! {}

pub mod alloc_site;

/// Enumeration of the filter generation techniques.
#[derive(Debug, Clone)]
pub enum FilterGen {
    /// Generate one allocation filter per allocation site.
    AllocSite(alloc_site::AllocSiteParams),
}

impl Default for FilterGen {
    fn default() -> Self {
        Self::AllocSite(alloc_site::AllocSiteParams::default())
    }
}

impl FilterGen {
    pub fn run(self, data: &data::Data) -> Res<Vec<Filter>> {
        match self {
            Self::AllocSite(params) => alloc_site::AllocSite::work(data, params),
        }
    }
}

impl From<alloc_site::AllocSiteParams> for FilterGen {
    fn from(params: alloc_site::AllocSiteParams) -> Self {
        Self::AllocSite(params)
    }
}

/// Trait implemented by filter generation techniques.
pub trait FilterGenExt {
    type Params: Default;

    fn work(data: &data::Data, params: Self::Params) -> Res<Vec<Filter>>;
}
