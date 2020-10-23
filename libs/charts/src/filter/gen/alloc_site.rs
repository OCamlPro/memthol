//! Allocation-site-based automatic filter generation.

prelude! {}

use filter::gen::*;

#[derive(Debug, Clone)]
pub struct AllocSiteParams {
    /// Minimum number of allocations needed for a filter to be created for a given file.
    min_count: Option<usize>,
}
impl Default for AllocSiteParams {
    fn default() -> Self {
        Self { min_count: None }
    }
}

impl AllocSiteParams {
    pub fn new(min_count: Option<usize>) -> Self {
        Self { min_count }
    }
}

pub type FileName = String;

pub struct AllocSiteWork {
    map: BTMap<FileName, usize>,
    unk: usize,
}

impl AllocSiteWork {
    pub fn new() -> Self {
        Self {
            map: BTMap::new(),
            unk: 0,
        }
    }

    pub fn inc(&mut self, file: Option<alloc::Str>) {
        if let Some(file) = file {
            file.str_do(|file| {
                if let Some(count) = self.map.get_mut(file) {
                    *count += 1
                } else {
                    let prev = self.map.insert(file.to_string(), 1);
                    debug_assert!(prev.is_none())
                }
            })
        } else {
            self.unk += 1
        }
    }

    pub fn scan(&mut self, data: &data::Data) {
        for alloc in data.iter_allocs() {
            alloc.alloc_site_do(|cloc_opt| self.inc(cloc_opt.map(|cloc| cloc.loc.file)))
        }
    }

    pub fn generate_subfilter(file: &str) -> filter::sub::RawSubFilter {
        let pred = filter::string_like::Pred::Contain;
        let line = filter::loc::LineSpec::any();
        let final_loc_spec = filter::loc::LocSpec::Value {
            value: file.into(),
            line,
        };
        let loc_spec = vec![filter::loc::LocSpec::Anything, final_loc_spec];
        let filter = filter::loc::LocFilter::new(pred, loc_spec);
        filter.into()
    }

    pub fn extract(self, params: AllocSiteParams) -> Res<Vec<Filter>> {
        let mut res = Vec::with_capacity(self.map.len());

        if self.map.is_empty() || (self.map.len() == 1 && self.unk == 0) {
            return Ok(res);
        }

        let min_count = if let Some(min) = params.min_count {
            log::info!("min_count is {}", min);
            min
        } else {
            // let avg = self.map.values().fold(0, |acc, cnt| acc + *cnt) / self.map.len();
            // avg / 20
            0
        };

        let validate = |count: usize| min_count <= count;

        let filter_count = self.map.iter().fold(
            0,
            |acc, (_, count)| if validate(*count) { acc + 1 } else { acc },
        );

        let mut colors = Color::randoms(filter_count).into_iter();

        for (file, count) in &self.map {
            if validate(*count) {
                let sub_filter = Self::generate_subfilter(&file);

                let color = colors.next().expect(
                    "internal error, `filter_count` is not consistant with the actual filter count",
                );
                let mut spec = filter::FilterSpec::new(color);
                spec.set_name(file.clone());

                let mut filter = filter::Filter::new(spec)?;
                filter.insert(sub_filter)?;

                res.push(filter)
            }
        }

        res.shrink_to_fit();

        // Rev-sorting by number of allocations. Note that the order does not matter as the filter
        // exact-match different allocation-site-files.
        res.sort_by(|lft, rgt| {
            let lft = self.map.get(lft.name()).cloned().unwrap_or(0);
            let rgt = self.map.get(rgt.name()).cloned().unwrap_or(0);
            // rev-sorting
            rgt.cmp(&lft)
        });

        Ok(res)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct AllocSite;

const MIN_KEY: &str = "min";

impl FilterGenExt for AllocSite {
    type Params = AllocSiteParams;

    const KEY: &'static str = "alloc_site";
    const FMT: Option<&'static str> = Some("min: <int>");

    fn work(data: &data::Data, params: Self::Params) -> Res<Vec<Filter>> {
        let mut work = AllocSiteWork::new();
        work.scan(data);
        work.extract(params)
    }

    fn parse_args(parser: Option<Parser>) -> Option<FilterGen> {
        let mut parser = if let Some(parser) = parser {
            parser
        } else {
            return Some(Self::Params::default().into());
        };

        if !parser.tag(MIN_KEY) {
            return None;
        }
        parser.ws();
        if !parser.char(':') {
            return None;
        }
        parser.ws();

        let min_count = parser.usize()?;

        parser.ws();
        let _optional = parser.char(',');
        parser.ws();

        if !parser.is_at_eoi() {
            return None;
        }

        Some(AllocSiteParams::new(Some(min_count)).into())
    }

    fn add_help(s: &mut String) {
        s.push_str(&format!(
            "\
- allocation site generator: `{0} {{ {1} }}`
    Generates one filter per allocation site, iff it is responsible for at least `{2}` allocations.
    Defaults: `{2}: 1`.

\
            ",
            Self::KEY,
            Self::FMT.unwrap(),
            MIN_KEY,
        ));
    }
}
