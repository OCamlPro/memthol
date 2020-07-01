//! Filter over ordered (number-like) quantities.

use crate::common::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Kind {
    Eq,
    Ge,
    Le,
    In,
}
impl fmt::Display for Kind {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Eq => write!(fmt, "="),
            Self::Ge => write!(fmt, "≥"),
            Self::Le => write!(fmt, "≤"),
            Self::In => write!(fmt, "⋲"),
        }
    }
}
impl Kind {
    pub fn all() -> Vec<Kind> {
        vec![Self::Eq, Self::Ge, Self::Le, Self::In]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Cmp {
    Eq,
    Ge,
    Le,
}
impl Cmp {
    pub fn apply<Num: PartialEq + PartialOrd>(&self, lhs: Num, rhs: Num) -> bool {
        match self {
            Self::Eq => lhs == rhs,
            Self::Ge => lhs >= rhs,
            Self::Le => lhs <= rhs,
        }
    }

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OrdFilter<Num> {
    Cmp { cmp: Cmp, val: Num },
    In { lb: Num, ub: Num },
}

impl<Num> OrdFilter<Num>
where
    Num: PartialEq + PartialOrd + fmt::Display,
{
    pub fn between(lb: Num, ub: Num) -> Res<Self> {
        if lb <= ub {
            Ok(Self::In { lb, ub })
        } else {
            bail!("illegal interval [{}, {}]", lb, ub)
        }
    }
    pub fn cmp(cmp: Cmp, val: Num) -> Self {
        Self::Cmp { cmp, val }
    }
    pub fn default_of_cmp(cmp_kind: Kind) -> Self
    where
        Num: Default,
    {
        match cmp_kind {
            Kind::Eq => Self::cmp(Cmp::Eq, Num::default()),
            Kind::Ge => Self::cmp(Cmp::Ge, Num::default()),
            Kind::Le => Self::cmp(Cmp::Le, Num::default()),
            Kind::In => Self::between(Num::default(), Num::default()).unwrap(),
        }
    }

    pub fn change_cmp_kind(self, kind: Kind) -> Self
    where
        Num: Default + Clone,
    {
        match self {
            Self::Cmp { val, .. } => match kind {
                Kind::Eq => Self::cmp(Cmp::Eq, val),
                Kind::Ge => Self::cmp(Cmp::Ge, val),
                Kind::Le => Self::cmp(Cmp::Le, val),
                Kind::In => Self::In {
                    lb: val.clone(),
                    ub: val,
                },
            },
            Self::In { .. } => Self::default_of_cmp(kind),
        }
    }

    pub fn cmp_kind(&self) -> Kind {
        match self {
            Self::Cmp { cmp: Cmp::Eq, .. } => Kind::Eq,
            Self::Cmp { cmp: Cmp::Ge, .. } => Kind::Ge,
            Self::Cmp { cmp: Cmp::Le, .. } => Kind::Le,
            Self::In { .. } => Kind::In,
        }
    }
}

impl<Num> Default for OrdFilter<Num>
where
    Num: Default + PartialEq + PartialOrd + fmt::Display,
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
    Num: PartialOrd + PartialEq + fmt::Display + Default + alloc_data::Parseable + Clone + 'static,
    Self: Into<filter::SubFilter>,
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
