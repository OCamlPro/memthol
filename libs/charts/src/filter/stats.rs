/*<LICENSE>
    This file is part of Memthol.

    Copyright (C) 2020 OCamlPro.

    Memthol is free software: you can redistribute it and/or modify
    it under the terms of the GNU General Public License as published by
    the Free Software Foundation, either version 3 of the License, or
    (at your option) any later version.

    Memthol is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
    GNU General Public License for more details.

    You should have received a copy of the GNU General Public License
    along with Memthol.  If not, see <https://www.gnu.org/licenses/>.
*/

//! Statistics about filters.
//!
//! Used by the client to present stats to users.

prelude! {}

/// Filter statistics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilterStats {
    /// Number of allocation caught by the filter.
    pub alloc_count: usize,
}
impl FilterStats {
    /// Constructor.
    pub fn new() -> Self {
        Self { alloc_count: 0 }
    }

    /// Increments the number of allocations.
    pub fn inc(&mut self) {
        self.alloc_count += 1
    }
}

/// Contains statistics for all filters.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllFilterStats {
    /// Map from filters to their statistics.
    pub stats: BTMap<uid::Line, FilterStats>,
}
impl AllFilterStats {
    /// Constructor.
    pub fn new() -> Self {
        Self {
            stats: BTMap::new(),
        }
    }

    /// Number of filters mentioned in the stats.
    pub fn len(&self) -> usize {
        self.stats.len()
    }

    /// Mutable accessor for a specific filter, inserts new stats if not there.
    pub fn stats_mut(&mut self, filter: uid::Line) -> &mut FilterStats {
        self.stats.entry(filter).or_insert_with(FilterStats::new)
    }

    /// Applies an action to a specific filter, inserts new stats if not there.
    pub fn stats_do<T>(
        &mut self,
        filter: uid::Line,
        action: impl FnOnce(&mut FilterStats) -> T,
    ) -> T {
        action(self.stats_mut(filter))
    }

    /// Removes the stats for a filter.
    pub fn remove(&mut self, filter: uid::Line) -> Option<FilterStats> {
        self.stats.remove(&filter)
    }

    /// Retrieves the stats for a filter.
    pub fn get(&self, filter: uid::Line) -> Option<&FilterStats> {
        self.stats.get(&filter)
    }
}
