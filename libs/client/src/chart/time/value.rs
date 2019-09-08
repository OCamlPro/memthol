//! Values that can appear in a time chart.

use crate::base::*;

/// A type of value.
pub enum Value {
    /// Total (live) size.
    TotalSize { current: usize },
    /// Highest (live) lifetime.
    HighestLifetime { live: Set<AllocUid> },
}
impl Value {
    /// Constructor for total size y-axis.
    pub fn total_size() -> Self {
        Self::TotalSize { current: 0 }
    }

    /// Constructor for the highest lifetime.
    pub fn highest_lifetime() -> Self {
        Self::HighestLifetime { live: Set::new() }
    }

    /// Y-axis description.
    pub fn desc(&self) -> &'static str {
        match self {
            Self::TotalSize { .. } => "total size (in bytes)",
            Self::HighestLifetime { .. } => "highest lifetime (in seconds)",
        }
    }

    /// The points corresponding to a diff.
    ///
    /// Needs the whole `Storage` so that it can retrieve the size of allocations that died.
    pub fn points_of_diff(&mut self, data: &Storage, diff: &AllocDiff) -> JsVal {
        match self {
            Self::TotalSize { ref mut current } => diff::total_size(current, data, diff),
            Self::HighestLifetime { ref mut live } => {
                let (_uid, point) = diff::highest_lifetime(live, data, diff);
                point
            }
        }
    }

    /// Origin point, if any.
    pub fn origin(&mut self, data: &Storage) -> Option<JsVal> {
        match self {
            Self::TotalSize { ref mut current } => origin::total_size(current, data),
            Self::HighestLifetime { ref mut live } => origin::highest_lifetime(live, data),
        }
    }
}

/// Helpers generating origin points for values.
pub mod origin {
    use super::*;
    use stdweb::js;

    pub fn total_size(current: &mut usize, data: &Storage) -> Option<JsVal> {
        *current = 0;
        Some(js!(return { x: @{data.start_time().as_js()}, y: 0 }))
    }

    pub fn highest_lifetime(live: &mut Set<AllocUid>, data: &Storage) -> Option<JsVal> {
        live.clear();
        Some(js!(return { x: @{data.start_time().as_js()}, y: 0 }))
    }
}

/// Helpers to retrieve all the points from a diff.
pub mod diff {
    use super::*;
    use stdweb::js;

    pub fn total_size(current: &mut usize, data: &Storage, diff: &AllocDiff) -> JsVal {
        let start_date = data.start_time();
        let date_of = |duration| {
            let mut res = start_date.clone();
            res.add(duration);
            res
        };

        // Result list.
        let res = js!(return []);

        // Sorted map used to construct the points.
        let mut map: Map<AllocDate, (usize, usize)> = Map::new();
        macro_rules! map {
            (add $date:expr => $size:expr) => {
                map.entry($date).or_insert((0, 0)).0 += $size
            };
            (sub $date:expr => $size:expr) => {
                map.entry($date).or_insert((0, 0)).1 += $size
            };
        }
        // Add the current time.
        map!(add data.current_time() => 0);

        for alloc in &diff.new {
            let toc = date_of(alloc.toc());
            let size = alloc.size();
            map!(add toc => size);
            if let Some(tod) = alloc.tod() {
                let tod = date_of(tod);
                map!(sub tod => size)
            }
        }

        for (uid, tod) in &diff.dead {
            let tod = date_of(*tod);
            let size = data.get_alloc(uid).size();
            map!(sub tod => size)
        }

        for (time, (to_add, to_sub)) in map {
            *current = *current + to_add - to_sub;
            let js_size = js!(return @{current.to_string()});
            js!(@(no_return)
                @{&res}.push({x: @{time.as_js()}, y: @{js_size}})
            )
        }

        res
    }

    pub fn highest_lifetime(
        live: &mut Set<AllocUid>,
        data: &Storage,
        diff: &AllocDiff,
    ) -> (Option<AllocUid>, JsVal) {
        // Updates the set of live allocs.
        for (uid, _) in &diff.dead {
            let was_there = live.remove(uid);
            assert! { was_there }
        }

        for alloc in &diff.new {
            if alloc.tod().is_none() {
                let is_new = live.insert(alloc.uid().clone());
                assert! { is_new }
            }
        }

        let curr_time = diff.time.clone();
        let get_lifetime = |uid| {
            let toc = data.get_alloc(uid).toc();
            if curr_time < toc {
                SinceStart::zero()
            } else {
                curr_time - data.get_alloc(uid).toc()
            }
        };

        let mut highest = None;
        macro_rules! update {
            ($uid:expr, $lifetime:expr) => {
                if let Some((uid, lifetime)) = highest.as_mut() {
                    if *lifetime < $lifetime {
                        *uid = $uid;
                        *lifetime = $lifetime
                    }
                } else {
                    highest = Some(($uid, $lifetime))
                }
            };
        }

        for uid in live.iter() {
            let lifetime = get_lifetime(uid);
            update!(uid.clone(), lifetime)
        }

        let (uid, js_value) = if let Some((uid, lifetime)) = highest {
            let lifetime = format!("{}.{}", lifetime.as_secs(), lifetime.subsec_millis());
            (Some(uid), js!(return @{lifetime}))
        } else {
            (None, js!(return "0"))
        };

        let mut current_time = data.start_time().clone();
        current_time.add(diff.time);

        let point = js!(return { x: @{current_time.as_js()}, y: @{js_value}});

        (uid, point)
    }
}
