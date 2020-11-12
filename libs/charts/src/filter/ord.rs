//! Filter over ordered (number-like) quantities.

prelude! {}

/// Comparison predicate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Pred {
    /// Equals.
    Eq,
    /// Greater or equal.
    Ge,
    /// Less or equal.
    Le,
    /// Inside a range.
    In,
}
impl fmt::Display for Pred {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Eq => write!(fmt, "="),
            Self::Ge => write!(fmt, "≥"),
            Self::Le => write!(fmt, "≤"),
            Self::In => write!(fmt, "⋲"),
        }
    }
}
impl Pred {
    /// A list of all the predicates variants.
    pub fn all() -> Vec<Pred> {
        base::debug_do! {
            // If you get an error here, it means the definition of `Pred` changed. You need to
            // update the following `match` statement, as well as the list returned by this function
            // (below).
            match Self::Eq {
                Self::Eq
                | Self::Ge
                | Self::Le
                | Self::In => ()
            }
        }
        vec![Self::Eq, Self::Ge, Self::Le, Self::In]
    }
}

/// Comparison predicates.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Cmp {
    /// Equal.
    Eq,
    /// Greater or equal.
    Ge,
    /// Less or equal.
    Le,
}
impl Cmp {
    /// Applies the comparison predicate to some quantities.
    pub fn apply<Num: PartialEq + PartialOrd>(&self, lhs: Num, rhs: Num) -> bool {
        match self {
            Self::Eq => lhs == rhs,
            Self::Ge => lhs >= rhs,
            Self::Le => lhs <= rhs,
        }
    }

    /// String representation of a predicate.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Eq => "=",
            Self::Le => "≤",
            Self::Ge => "≥",
        }
    }
}
impl fmt::Display for Cmp {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{}", self.as_str())
    }
}

/// A filter for ordered quantities.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum OrdFilter<Num> {
    /// Comparison with a constant.
    Cmp {
        /// Operator.
        cmp: Cmp,
        /// Constant value.
        val: Num,
    },
    /// Interval.
    In {
        /// Lower-bound.
        lb: Num,
        /// Upper-bound.
        ub: Num,
    },
}

impl<Num> OrdFilter<Num>
where
    Num: PartialEq + PartialOrd + fmt::Debug,
{
    /// Creates an interval filter.
    pub fn between(lb: Num, ub: Num) -> Res<Self> {
        if lb <= ub {
            Ok(Self::In { lb, ub })
        } else {
            bail!("illegal interval [{:?}, {:?}]", lb, ub)
        }
    }

    /// Creates a filter for a comparison with a constant.
    pub fn cmp(cmp: Cmp, val: Num) -> Self {
        Self::Cmp { cmp, val }
    }

    /// Generates a default filter for an ordered filter predicate.
    pub fn default_of_cmp(cmp_kind: Pred) -> Self
    where
        Num: Default,
    {
        match cmp_kind {
            Pred::Eq => Self::cmp(Cmp::Eq, Num::default()),
            Pred::Ge => Self::cmp(Cmp::Ge, Num::default()),
            Pred::Le => Self::cmp(Cmp::Le, Num::default()),
            Pred::In => Self::between(Num::default(), Num::default()).unwrap(),
        }
    }

    /// Changes the predicate of a an ordered filter.
    ///
    /// This method tries to make the change natural from a user's perspective. For instance, it
    /// keeps the value for comparison predicates.
    pub fn change_cmp_kind(self, kind: Pred) -> Self
    where
        Num: Default + Clone,
    {
        match self {
            Self::Cmp { val, .. } => match kind {
                Pred::Eq => Self::cmp(Cmp::Eq, val),
                Pred::Ge => Self::cmp(Cmp::Ge, val),
                Pred::Le => Self::cmp(Cmp::Le, val),
                Pred::In => Self::In {
                    lb: val.clone(),
                    ub: val,
                },
            },
            Self::In { .. } => Self::default_of_cmp(kind),
        }
    }

    /// Accessor for the ordered predicate.
    pub fn cmp_kind(&self) -> Pred {
        match self {
            Self::Cmp { cmp: Cmp::Eq, .. } => Pred::Eq,
            Self::Cmp { cmp: Cmp::Ge, .. } => Pred::Ge,
            Self::Cmp { cmp: Cmp::Le, .. } => Pred::Le,
            Self::In { .. } => Pred::In,
        }
    }
}

impl<Num> Default for OrdFilter<Num>
where
    Num: Default + PartialEq + PartialOrd + fmt::Debug,
{
    fn default() -> Self {
        Self::cmp(Cmp::Eq, Num::default())
    }
}

impl<Num: fmt::Display> fmt::Display for OrdFilter<Num> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Cmp { cmp, val } => write!(fmt, "{} {}", cmp, val),
            Self::In { lb, ub } => write!(fmt, "⋲ [{}, {}]", lb, ub),
        }
    }
}

impl<Num> crate::filter::FilterExt<Num> for OrdFilter<Num>
where
    Num: PartialOrd + PartialEq + fmt::Display + Clone + 'static,
{
    fn apply(&self, data: &Num) -> bool {
        match self {
            Self::Cmp { cmp, val } => cmp.apply(data, val),
            Self::In { lb, ub } => Cmp::Ge.apply(data, lb) && Cmp::Le.apply(data, ub),
        }
    }
}

/// An update for a size filter.
pub type SizeUpdate = Update<u32>;

/// An update for a lifetime filter.
pub type LifetimeUpdate = Update<time::Lifetime>;

/// An update for an ordered filter.
pub enum Update<Val> {
    /// Change the comparator of a `Cmp` filter.
    Cmp(Cmp),
    /// Change the value of a `Cmp` filter.
    Value(Val),
    /// Change the lower-bound of an interval filter.
    InLb(Val),
    /// Change the upper-bound of an interval filter.
    InUb(Val),
}
impl<Val: fmt::Display> fmt::Display for Update<Val> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Cmp(cmp) => write!(fmt, "comparator <- {}", cmp),
            Self::Value(val) => write!(fmt, "comparator value <- {}", val),
            Self::InLb(val) => write!(fmt, "interval lbound <- {}", val),
            Self::InUb(val) => write!(fmt, "interval ubound <- {}", val),
        }
    }
}

impl<Num: fmt::Display + PartialEq + Eq> OrdFilter<Num> {
    /// Updates the filter.
    pub fn update(&mut self, update: Update<Num>) -> Res<bool> {
        let has_changed = match self {
            Self::Cmp { cmp, val } => match update {
                Update::Cmp(nu_cmp) => {
                    if nu_cmp != *cmp {
                        *cmp = nu_cmp;
                        true
                    } else {
                        false
                    }
                }
                Update::Value(nu_val) => {
                    if nu_val != *val {
                        *val = nu_val;
                        true
                    } else {
                        false
                    }
                }

                update => bail!("cannot update filter `_ {}` with `{}`", self, update),
            },

            Self::In { lb, ub } => match update {
                Update::InLb(val) => {
                    if val != *lb {
                        *lb = val;
                        true
                    } else {
                        false
                    }
                }
                Update::InUb(val) => {
                    if val != *ub {
                        *ub = val;
                        true
                    } else {
                        false
                    }
                }

                update => bail!("cannot update filter `_ {}` with `{}`", self, update),
            },
        };

        Ok(has_changed)
    }
}
