//! Generic filter constructions for allocation properties that are lists of string-like elements.
//!
//! Used for labels and locations.

prelude! {}

use filter::FilterExt;

/// A comparison predicate over lists.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Pred {
    /// Contain predicate.
    Contain,
    /// Exclude predicate.
    Exclude,
}
impl fmt::Display for Pred {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Contain => write!(fmt, "contains"),
            Self::Exclude => write!(fmt, "excludes"),
        }
    }
}
impl Pred {
    ///
    pub fn all() -> Vec<Pred> {
        base::debug_do! {
            // If you get an error here, it means the definition of `Pred` changed. You need to
            // update the following `match` statement, as well as the list returned by this function
            // (below).
            match Self::Contain {
                Self::Contain
                | Self::Exclude => (),
            }
        }
        vec![Self::Contain, Self::Exclude]
    }
}

/// Trait that string-like specifications must implement.
pub trait SpecExt: Default + Clone + fmt::Display + Sized {
    /// Type of data the specification is able to check for matches.
    type Data: fmt::Display;

    /// Description of the kind of data this specification works on.
    const DATA_DESC: &'static str;

    /// Constructor from strings.
    fn from_string(s: impl Into<String>) -> Res<Self>;

    /// True if the specification is empty, meaning it is ignored.
    fn is_empty(&self) -> bool;

    /// Extracts data from an allocation.
    fn data_of_alloc(alloc: &Alloc) -> Arc<Vec<Self::Data>>;

    /// True if the input data is a match for this specification.
    fn matches(&self, data: &Self::Data) -> bool;

    /// True if the specification matches a repetition of anything.
    fn matches_anything(&self) -> bool;
}

/// A filter for a lists of string-like elements.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StringLikeFilter<Spec> {
    /// The predicate.
    pred: Pred,
    /// The specifications.
    specs: Vec<Spec>,
}

impl<Spec> fmt::Display for StringLikeFilter<Spec>
where
    Spec: SpecExt,
{
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{} {} [", Spec::DATA_DESC, self.pred)?;
        for spec in &self.specs {
            write!(fmt, " ... {}", spec)?
        }
        write!(fmt, " ... ]")
    }
}

impl<Spec> StringLikeFilter<Spec> {
    /// Constructor.
    pub fn new(pred: Pred, specs: Vec<Spec>) -> Self {
        Self { pred, specs }
    }
    /// "Contain" constructor.
    pub fn contain(specs: Vec<Spec>) -> Self {
        Self {
            pred: Pred::Contain,
            specs,
        }
    }
    /// "Exclude" constructor.
    pub fn exclude(specs: Vec<Spec>) -> Self {
        Self {
            pred: Pred::Exclude,
            specs,
        }
    }

    /// Predicate of a filter.
    pub fn pred(&self) -> Pred {
        self.pred
    }
    /// Specifications of a filter.
    pub fn specs(&self) -> &Vec<Spec> {
        &self.specs
    }
}

impl<Spec> FilterExt<Arc<Vec<Spec::Data>>> for StringLikeFilter<Spec>
where
    Spec: SpecExt,
{
    fn apply(&self, data: &Arc<Vec<Spec::Data>>) -> bool {
        self.matches(data)
    }
}

impl<Spec> StringLikeFilter<Spec>
where
    Spec: SpecExt,
{
    /// Removes consecutive wildcards in the specs.
    ///
    /// Ideally, this should run after any manipulation over `self.specs`.
    fn clean_specs(&mut self) {
        // Going old-school to be able to remove elements of `self.specs` as we go through it.
        let mut i = 0;
        let mut prev_is_wildcard = false;
        while i < self.specs.len() {
            // The loop invariant to prove termination is that `self.specs.len() - i` decreases by
            // `1` at each iteration. This is because ¬`increment` ⊨ "an element was removed from
            // `self.specs`". Note also that `increment` ⊨ "no element was removed", which is
            // irrelevant for termination but important for correction.
            let increment = if self.specs[i].matches_anything() {
                // Current element is a wildcard.
                if prev_is_wildcard {
                    // Previous element is a wildcard, remove current element. Preserving order is
                    // mandatory, do **not** evil-optimize it to `swap_remove`.
                    self.specs.remove(i);
                    false
                } else {
                    // Previous element is not a wildcard, remember and keep going.
                    prev_is_wildcard = true;
                    true
                }
            } else {
                // Not a wildcard, remember and keep going.
                prev_is_wildcard = false;
                true
            };
            if increment {
                i += 1
            }
        }
    }

    /// Replaces the specification at some index.
    pub fn replace(&mut self, index: usize, spec: Spec) {
        if spec.is_empty() {
            self.specs.remove(index);
        } else {
            self.specs[index] = spec;
        }
        self.clean_specs()
    }

    /// Inserts a specification at some index.
    pub fn insert(&mut self, index: usize, spec: Spec) {
        if !spec.is_empty() {
            self.specs.insert(index, spec);
            self.clean_specs()
        }
    }

    /// True if the filter input data is a match for the filter.
    pub fn matches(&self, data: &[Spec::Data]) -> bool {
        let res = Self::check_contain(&self.specs, data);
        match self.pred {
            Pred::Contain => res,
            Pred::Exclude => !res,
        }
    }

    /// Helper handling the suffix case.
    ///
    /// Returns nothing if the specs do not fall in the suffix case.
    fn check_suffix(specs: &Vec<Spec>, data: &[Spec::Data]) -> Option<bool> {
        let mut slice = &specs[0..];

        if !slice.is_empty() {
            if !slice[0].matches_anything() {
                // Not starting with a wildcard.
                return None;
            }

            slice = &slice[1..]
        }

        // The specs start with a wildcard. Drain all wildcards.
        while !slice.is_empty() {
            if !slice[0].matches_anything() {
                // Not a wildcard, stop here.
                break;
            } else {
                slice = &slice[1..]
            }
        }

        // Not in a suffix case if there are wildcards left.
        if slice.iter().any(Spec::matches_anything) {
            return None;
        }

        // We're in a suffix case.

        // Reverse iter on the `slice` and `data` to check if we have a match.
        let rev_slice = slice.iter().rev();
        let mut rev_data = data.iter().rev();

        for spec in rev_slice {
            if let Some(data) = rev_data.next() {
                if spec.matches(data) {
                    continue;
                } else {
                    return Some(false);
                }
            } else {
                return Some(false);
            }
        }

        // Only reachable if there's no more specs in the suffix, and all specs matched the data's
        // suffix.
        Some(true)
    }

    /// Helper that returns true if the input data verifies the input specs.
    fn check_contain(specs: &Vec<Spec>, data: &[Spec::Data]) -> bool {
        if let Some(res) = Self::check_suffix(specs, data) {
            return res;
        }

        let mut data = data.iter();
        let mut specs = specs.iter();

        'next_spec: while let Some(spec) = specs.next() {
            // `can_skip` is true if `spec` does not have to match the next label, it can match
            // data appearing later in the sequence.
            let (can_skip, spec) = if spec.matches_anything() {
                // We're matching a sequence of anything. Find the next spec that's not an
                // `Anything`.
                let mut spec_opt = None;
                'drain_match_anything: while let Some(spec) = specs.next() {
                    if spec.matches_anything() {
                        continue 'drain_match_anything;
                    } else {
                        spec_opt = Some(spec);
                        break 'drain_match_anything;
                    }
                }

                if let Some(spec) = spec_opt {
                    (true, spec)
                } else {
                    // We're matching anything, and there is no spec to match after that.
                    return true;
                }
            } else {
                // We're matching an actual spec.
                (false, spec)
            };

            'find_match: while let Some(data) = data.next() {
                if spec.matches(data) {
                    // Found a match.
                    continue 'next_spec;
                } else if can_skip {
                    // `spec` does not have to match right away, keep moving.
                    continue 'find_match;
                } else {
                    return false;
                }
            }

            // Only reachable if there is no more data.
            return false;
        }

        // Only reachable if there are no more specs and all succeeded. Now we just need to check if
        // there are data left.
        data.next().is_none()
    }
}

/// An update for a string-like filter.
pub enum Update {
    /// Change the predicate of the filter.
    Pred(Pred),
    /// Add a new specification at some position.
    Add(usize),
    /// Replace a specification at some position.
    Replace(usize, String),
}
impl fmt::Display for Update {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Pred(pred) => write!(fmt, "pred <- {}", pred),
            Self::Add(index) => write!(fmt, "specs <- add at {}", index),
            Self::Replace(index, spec) => write!(fmt, "specs[{}] <- {}", index, spec),
        }
    }
}

impl<Spec> StringLikeFilter<Spec>
where
    Spec: SpecExt,
{
    /// Updates the filter.
    ///
    /// Returns true iff something changed.
    pub fn update(&mut self, update: Update) -> Res<bool> {
        let has_changed = match update {
            Update::Pred(pred) => {
                if pred != self.pred {
                    self.pred = pred;
                    true
                } else {
                    false
                }
            }
            Update::Add(index) => {
                self.specs.insert(index, Spec::default());
                true
            }
            Update::Replace(index, spec) => {
                let spec = Spec::from_string(spec).chain_err(|| {
                    format!(
                        "while replacing {} spec #{} in filter",
                        Spec::DATA_DESC,
                        index
                    )
                })?;
                self.specs[index] = spec;
                true
            }
        };
        Ok(has_changed)
    }
}
