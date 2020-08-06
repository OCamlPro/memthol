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

pub type FileName = String;

pub struct AllocSite {
    map: Map<FileName, usize>,
    unk: usize,
}

impl AllocSite {
    fn new() -> Self {
        Self {
            map: Map::new(),
            unk: 0,
        }
    }

    fn inc(&mut self, file: Option<alloc::Str>) {
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

    fn scan(&mut self, data: &data::Data) {
        for alloc in data.iter_all() {
            alloc.alloc_site_do(|cloc_opt| self.inc(cloc_opt.map(|cloc| cloc.loc.file)))
        }
    }

    fn generate_subfilter(file: &str) -> filter::sub::RawSubFilter {
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

    fn extract(self, params: AllocSiteParams) -> Res<Vec<Filter>> {
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

impl FilterGenExt for AllocSite {
    type Params = AllocSiteParams;

    fn work(data: &data::Data, params: Self::Params) -> Res<Vec<Filter>> {
        let mut slf = Self::new();
        slf.scan(data);
        slf.extract(params)
    }
}
