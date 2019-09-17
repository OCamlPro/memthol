//! Type-safe zero-cost indices.

use std::ops::Deref;

use crate::base::{Deserialize, Serialize};

/// Creates an index type.
macro_rules! new {
    (
        $(
            $(#[$meta:meta])*
            $ident:ident
        ),* $(,)*
    ) => {$(
        $(#[$meta])*
        #[derive(Debug, Clone, Copy, Serialize, Deserialize)]
        pub struct $ident {
            /// Actual index.
            index: usize
        }
        impl $ident {
            /// Constructor.
            pub fn new(index: usize) -> Self {
                Self { index }
            }

            /// Index following this one in a vector.
            pub fn next<T>(&self, vec: &Vec<T>) -> Option<Self> {
                let next = self.index + 1;
                if next < vec.len() {
                    Some(Self::new(next))
                } else {
                    None
                }
            }

            /// Index preceding this one.
            pub fn prev(&self) -> Option<Self> {
                if self.index == 0 {
                    None
                } else {
                    Some( Self::new(self.index - 1) )
                }
            }

            /// Index of the element after the last one in a vector.
            pub fn next_of<T>(vec: Vec<T>) -> Self {
                Self::new(vec.len())
            }
        }
        impl Deref for $ident {
            type Target = usize;
            fn deref(&self) -> &usize {
                &self.index
            }
        }
        impl std::fmt::Display for $ident {
            fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(fmt, "{}", self.index)
            }
        }
    )*};
}

new! {
    /// Filter index.
    ///
    /// Used to refer to [`Filter`]s in a [`Filters`] (filter collection) and/or in a [`Charts`].
    ///
    /// [`Filter`]: ../filter/sub/struct.Filter.html (The Filter struct)
    /// [`Filters`]: ../filter/struct.Filters.html (The Filters struct)
    /// [`Charts`]: ../struct.Charts.html (The Charts struct)
    Filter,
    /// Sub-filter index.
    ///
    /// Used to refer to [`SubFilter`]s in a [`Filter`].
    ///
    /// [`SubFilter`]: ../filter/sub/struct.SubFilter.html (The SubFilter struct)
    /// [`Filter`]: ../filter/struct.Filter.html (The Filter struct)
    SubFilter,
    /// Chart index.
    ///
    /// Used to refer to [`Chart`]s in a [`Charts`].
    ///
    /// [`Chart`]: ../chart/struct.Chart.html (The Chart struct)
    /// [`Charts`]: ../struct.Charts.html (The Charts struct)
    Chart,
}
