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
        }
        impl Deref for $ident {
            type Target = usize;
            fn deref(&self) -> &usize {
                &self.index
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
