//! Allocation-site-file-based automatic filter generation.
//!
//! Parameterized with an optional `min_count: usize`. This generator generates one filter per
//! allocation-site-file with at least `min_count` allocations in it. Note that the behavior is the
//! same when `min_count` is `0` or when it is `1`.
//!
//! When no `min_count` parameter is present, the current behavior is the same as `min_count == 1`.

prelude! {}

use filter::gen::*;

/// Parameters for the alloc-site generator.
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
    /// Constructor.
    pub fn new(min_count: Option<usize>) -> Self {
        Self { min_count }
    }
}

type FileName = String;

/// Actual alloc-site generator worker.
pub struct AllocSiteWork {
    /// Maps file names to the number of allocations in them.
    map: BTMap<FileName, (usize, Option<uid::Filter>)>,
    /// Number of allocations in unknown files.
    unk: usize,
}

impl AllocSiteWork {
    /// Constructor.
    pub fn new() -> Self {
        Self {
            map: BTMap::new(),
            unk: 0,
        }
    }

    /// Increments by one the number of allocations in `file`.
    ///
    /// If `None`, `file` is treated as the unknown file.
    pub fn inc(&mut self, file: Option<alloc::Str>) {
        if let Some(file) = file {
            file.str_do(|file| {
                if let Some((count, _)) = self.map.get_mut(file) {
                    *count += 1
                } else {
                    let prev = self.map.insert(file.to_string(), (1, None));
                    debug_assert!(prev.is_none())
                }
            })
        } else {
            self.unk += 1
        }
    }

    /// Scans the input data to populate the map from file names to allocation count.
    pub fn scan(&mut self, data: &data::Data) {
        for alloc in data.iter_allocs() {
            alloc.alloc_site_do(|cloc_opt| self.inc(cloc_opt.map(|cloc| cloc.loc.file)))
        }
    }

    /// Generates a subfilter for a specific file name.
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

    /// Extracts allocation-site-file filters.
    pub fn extract(&mut self, params: AllocSiteParams) -> Res<Vec<Filter>> {
        let mut res = Vec::with_capacity(self.map.len());

        if self.map.is_empty() || (self.map.len() == 1 && self.unk == 0) {
            return Ok(res);
        }

        let min_count = if let Some(min) = params.min_count {
            min
        } else {
            // let avg = self.map.values().fold(0, |acc, cnt| acc + *cnt) / self.map.len();
            // avg / 20
            0
        };

        let validate = |count: usize| min_count <= count;

        let filter_count = self.map.iter().fold(
            0,
            |acc, (_, (count, _))| if validate(*count) { acc + 1 } else { acc },
        );

        let mut colors = Color::randoms(filter_count).into_iter();

        for (file, (count, uid_opt)) in &mut self.map {
            if validate(*count) {
                let sub_filter = Self::generate_subfilter(&file);

                let color = colors.next().expect(
                    "internal error, `filter_count` is not consistant with the actual filter count",
                );
                let mut spec = filter::FilterSpec::new(color);
                spec.set_name(file.clone());

                let mut filter = filter::Filter::new(spec)?;
                filter.insert(sub_filter)?;

                debug_assert_eq!(*uid_opt, None);
                *uid_opt = Some(filter.uid());

                res.push(filter)
            }
        }

        res.shrink_to_fit();

        // Rev-sorting by number of allocations. Note that the order does not matter as the filter
        // exact-match different allocation-site-files.
        res.sort_by(|lft, rgt| {
            let lft = self.map.get(lft.name()).cloned().unwrap_or((0, None));
            let rgt = self.map.get(rgt.name()).cloned().unwrap_or((0, None));
            // rev-sorting
            rgt.cmp(&lft)
        });

        // log::info!("allocation sites:");
        // for (file, (count, uid)) in &self.map {
        //     log::info!("    {:>30}: {}, captured by {:?}", file, count, uid)
        // }

        Ok(res)
    }

    /// Runs chart generation.
    pub fn chart_gen(self, filters: &Filters) -> Res<Vec<chart::Chart>> {
        chart_gen::alloc_file_prefix(
            filters,
            self.map
                .iter()
                .filter_map(|(file, (_count, uid))| uid.map(|uid| (file, uid))),
        )
    }
}

/// Unit-struct handling CLAP and creating/running the actual generator.
#[derive(Debug, Clone, Copy)]
pub struct AllocSite;

/// Name of the `min` key.
const MIN_KEY: &str = "min";

impl FilterGenExt for AllocSite {
    type Params = AllocSiteParams;

    const KEY: &'static str = "alloc_site";
    const FMT: Option<&'static str> = Some("min: <int>");

    fn work(data: &data::Data, params: Self::Params) -> Res<(Filters, Vec<chart::Chart>)> {
        let mut work = AllocSiteWork::new();
        work.scan(data);
        let filters = work.extract(params).map(Filters::new_with)?;
        let charts = work.chart_gen(&filters)?;
        Ok((filters, charts))
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
