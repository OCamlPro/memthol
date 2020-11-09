//! Simple chart generators.

prelude! {}

/// Description of a chart.
pub struct ChartDesc {
    /// Title of the chart.
    pub title: Option<String>,
    /// Active filters.
    pub spec: chart::ChartSpec,
}
impl ChartDesc {
    /// Constructor.
    pub fn new_size_over_time(title: Option<String>, spec: BTMap<uid::Line, bool>) -> Self {
        Self {
            title,
            spec: chart::ChartSpec::new(
                chart::axis::XAxis::Time,
                chart::axis::YAxis::TotalSize,
                spec,
            ),
        }
    }

    /// Turns itself in a chart.
    pub fn into_chart(self, filters: &Filters) -> Res<chart::Chart> {
        let Self { title, spec } = self;
        chart::Chart::from_spec(title, filters, spec)
    }
}

/// Default chart generation.
pub fn default(filters: &Filters) -> Res<Vec<chart::Chart>> {
    single(filters)
}

/// Generates a single graph containing everything.
pub fn single(filters: &Filters) -> Res<Vec<chart::Chart>> {
    Ok(vec![ChartDesc::new_size_over_time(
        None,
        filters.uid_map(true),
    )
    .into_chart(filters)?])
}

/// Generates one chart per common allocation-site-file prefix.
pub fn alloc_file_prefix<'a>(
    filters: &Filters,
    file_to_filter: impl IntoIterator<Item = (&'a String, uid::Filter)>,
) -> Res<Vec<chart::Chart>> {
    let mut pref_to_filters = HMap::new();

    for (file, uid) in file_to_filter {
        use std::path::Path;
        let path = Path::new(file);
        let is_new = pref_to_filters
            .entry(path.parent())
            .or_insert_with(HSet::new)
            .insert(uid::Line::Filter(uid));
        debug_assert!(is_new);
    }

    let all_inactive = filters.uid_map(false);

    let mut pref_filters = vec![];
    let mut lonely = ChartDesc::new_size_over_time(Some("others".into()), all_inactive.clone());
    let mut no_pref =
        ChartDesc::new_size_over_time(Some("prefix-less files".into()), all_inactive.clone());

    for (pref, uids) in pref_to_filters {
        debug_assert!(!uids.is_empty());
        if let Some(pref) = pref {
            if uids.len() > 1 {
                let title = pref.to_string_lossy();

                if title.is_empty() {
                    no_pref
                        .spec
                        .active_mut()
                        .extend(uids.into_iter().map(|uid| (uid, true)));
                } else {
                    let mut active = all_inactive.clone();
                    for uid in uids {
                        active.insert(uid, true);
                    }
                    let desc = ChartDesc::new_size_over_time(Some(title.into()), active);
                    let chart = desc.into_chart(filters)?;
                    pref_filters.push(chart)
                }
            } else {
                lonely
                    .spec
                    .active_mut()
                    .extend(uids.into_iter().map(|uid| (uid, true)))
            }
        } else {
            no_pref
                .spec
                .active_mut()
                .extend(uids.into_iter().map(|uid| (uid, true)));
        }
    }

    if !pref_filters.is_empty() {
        let mut active = all_inactive;
        active.insert(uid::Line::Everything, true);
        let everything = ChartDesc::new_size_over_time(None, active);
        let mut res = vec![everything.into_chart(filters)?];
        res.extend(pref_filters);
        if lonely.spec.has_active_filters() {
            res.push(lonely.into_chart(filters)?)
        }
        if no_pref.spec.has_active_filters() {
            res.push(no_pref.into_chart(filters)?)
        }
        Ok(res)
    } else {
        single(filters)
    }
}
