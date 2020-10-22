//! Macros used throughout the whole project.

/// Fails if a result expression is an error, after printing the error.
#[macro_export]
macro_rules! unwrap_or {
    ($e:expr, exit) => {
        $crate::unwrap_or!($e, std::process::exit(2))
    };
    ($e:expr, $action:expr) => {
        match $e {
            Ok(res) => res,
            Err(e) => {
                $crate::prelude::log::error!("|===| Error ({}:{})", file!(), line!());
                for e in e.iter() {
                    for line in format!("{}", e).lines() {
                        $crate::prelude::log::error!("| {}", line)
                    }
                }
                $crate::prelude::log::error!("|===|");
                $action
            }
        }
    };
}

/// Imports this crate's prelude.
#[macro_export]
macro_rules! prelude {
    () => {
        use $crate::prelude::*;
    };
    ($($imports:tt)*) => {
        use $crate::prelude::{$($imports)*};
    };
}

/// Sub-macro for `cfg_item`.
#[doc(hidden)]
#[macro_export]
macro_rules! __cfg_cond {
    (@cfg(time_stats) $($stuff:tt)*) => {
        #[cfg(any(test, feature = "time_stats"))]
        $($stuff)*
    };
    (@cfg(not time_stats) $($stuff:tt)*) => {
        #[cfg(not(any(test, feature = "time_stats")))]
        $($stuff)*
    };

    (@cfg(server) $($stuff:tt)*) => {
        #[cfg(any(test, feature = "server"))]
        $($stuff)*
    };
    (@cfg(not server) $($stuff:tt)*) => {
        #[cfg(not(any(test, feature = "server")))]
        $($stuff)*
    };

    (@cfg(debug) $($stuff:tt)*) => {
        #[cfg(any(test, debug_assertions))]
        $($stuff)*
    };
    (@cfg(not debug) $($stuff:tt)*) => {
        #[cfg(not(any(test, debug_assertions)))]
        $($stuff)*
    };

    (@cfg(release) $($stuff:tt)*) => {
        $crate::cfg_item! {
            @cfg(not debug)
            $($stuff)*
        }
    };
    (@cfg(not release) $($stuff:tt)*) => {
        $crate::cfg_item! {
            @cfg(debug)
            $($stuff)*
        }
    };
}

/// `cfg`-level if-then-else for items.
#[macro_export]
macro_rules! cfg_item {
    (
        pref {
            $($pref_stuff:tt)*
        }
        cfg ($($cfg_conf:tt)*) {
            $($cfg_stuff:tt)*
        }
        else {
            $($els_stuff:tt)*
        }
    ) => {
        $crate::cfg_item! {
            pref {
                $($pref_stuff)*
            }
            cfg($($cfg_conf)*) {
                $($cfg_stuff)*
            }
            cfg(not $($cfg_conf)*) {
                $($els_stuff)*
            }
        }
    };

    (
        pref {
            $($pref_stuff:tt)*
        }
        cfg ($($cfg_conf:tt)*) {
            $($cfg_stuff:tt)*
        }
        $($tail:tt)*
    ) => {
        $crate::__cfg_cond! {
            @cfg
            ($($cfg_conf)*)
            $($pref_stuff)* $($cfg_stuff)*
        }

        $crate::cfg_item! {
            pref { $($pref_stuff)* }
            $($tail)*
        }
    };

    (
        pref {
            $($pref_stuff:tt)*
        }
    ) => {};

    (
        cfg $($tail:tt)*
    ) => {
        $crate::cfg_item! {
            pref {}
            cfg $($tail)*
        }
    };
}

cfg_item! {
    pref {
        /// Counts the time it takes to evaluate an expression.
        ///
        /// Expects `<stopwatches> => <expr>` where `<stopwatches>` is a comma-separated list of
        /// `Stopwatch`.
        #[macro_export]
        macro_rules! time
    }

    cfg(time_stats) {
        {
            ( $(> $stopwatches:expr, )+ $e:expr) => {{
                $(
                    $stopwatches.start();
                )*
                let res = $e;
                $(
                    $stopwatches.stop();
                )*
                res
            }};
            ($e:expr, |$time:pat| $time_action:expr) => {{
                let mut ____sw = $crate::time_stats::RealStopwatch::new();
                ____sw.start();
                let res = $e;
                ____sw.stop();
                {
                    let $time = ____sw;
                    $time_action
                }
                res
            }};
        }
    } else {
        {
            ( $(> $stopwatches:expr, )+ $e:expr) => {
                $e
            };
            ($e:expr, |$time:pat| $time_action:expr) => {
                $e
            };
        }
    }
}

cfg_item! {
    pref {
        /// Ignores the input tokens in `release`.
        #[macro_export]
        macro_rules! debug_do
    }

    cfg(debug) {
        {
            ($($stuff:tt)*) => {{
                $($stuff)*
            }};
        }
    } else {
        {
            ($($stuff:tt)*) => {};
        }
    }
}

/// Sub-macro for `implement`.
#[macro_export]
#[doc(hidden)]
macro_rules! __implement_trait {
    (@Display ($ty:ty) ($($args:tt)*) {
        |&$slf:ident, $fmt:pat| $def:expr $(,)?
    }) => {
        impl < $($args)* > std::fmt::Display for $ty {
            fn fmt(&$slf, $fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
                $def
            }
        }
    };

    (@From ($ty:ty) ($($args:tt)*) {
        from $src:ty => |$param:pat| $def:expr $(,)?
    }) => {
        impl < $($args)* > std::convert::From<$src> for $ty {
            fn from($param: $src) -> Self {
                $def
            }
        }
    };
    (@From ($ty:ty) ($($args:tt)*) {
        from $src:ty => |$param:pat| $def:expr,
        $($tail:tt)*
    }) => {
        impl < $($args)* > std::convert::From<$src> for $ty {
            fn from($param: $src) -> Self {
                $def
            }
        }
        $crate::__implement_trait! {
            @From ($ty) ($($args)*) { $($tail)* }
        }
    };

    (@Into ($ty:ty) ($($args:tt)*) {
        to $tgt:ty => |$slf:ident| $def:expr $(,)?
    }) => {
        impl < $($args)* > std::convert::Into<$tgt> for $ty {
            fn into($slf) -> $tgt {
                $def
            }
        }
    };
    (@Into ($ty:ty) ($($args:tt)*) {
        to $tgt:ty => |$slf:ident| $def:expr,
        $($tail:tt)*
    }) => {
        impl < $($args)* > std::convert::Into<$tgt> for $ty {
            fn into($slf) -> $tgt {
                $def
            }
        }
        $crate::__implement_trait! {
            @Into ($ty) ($($args)*) { $($tail)* }
        }
    };

    (@Default ($ty:ty) ($($args:tt)*) {
        $def:expr $(,)?
    }) => {
        impl < $($args)* > std::default::Default for $ty {
            fn default() -> Self {
                $def
            }
        }
    };

    (@Deref ($ty:ty) ($($args:tt)*) {
        to $tgt:ty => |&$slf:ident| $def:expr $(,)?
    }) => {
        impl $(< $($args)* >)* std::ops::Deref for $ty {
            type Target = $tgt;
            fn deref(&$slf) -> &$tgt {
                $def
            }
        }
    };

    (@DerefMut ($ty:ty) ($($args:tt)*) {
        |&mut $slf:ident| $def:expr $(,)?
    }) => {
        impl $(< $($args)* >)* std::ops::DerefMut for $ty {
            fn deref_mut(&mut $slf) -> &mut Self::Target {
                $def
            }
        }
    };
}

/// Convenience macro for implementing basic traits.
///
/// Supports `Display`, `From`, `Into`, `Deref`, `DerefMut`.
#[macro_export]
macro_rules! implement {
    (
        impl $ty:ty {
            $(
                $trait:ident {
                    $($trait_def:tt)*
                }
            )+
        }

        $($tail:tt)*
    ) => {
        $(
            $crate::__implement_trait! {
                @ $trait ($ty) () { $($trait_def)* }
            }
        )+

        $crate::implement! { $($tail)* }
    };

    (
        impl $trait:ident for $ty:ty {
            $($trait_def:tt)*
        }

        $($tail:tt)*
    ) => {
        $crate::__implement_trait! {
            @ $trait ($ty) () { $($trait_def)* }
        }

        $crate::implement! { $($tail)* }
    };

    (
        impl $ty_args:tt $ty:ty {
            $(
                $trait:ident {
                    $($trait_def:tt)*
                }
            )+
        }

        $($tail:tt)*
    ) => {
        $(
            $crate::__implement_trait! {
                @ $trait ($ty) $ty_args { $($trait_def)* }
            }
        )+

        $crate::implement! { $($tail)* }
    };

    (
        impl $ty_args:tt $trait:ident for $ty:ty {
            $($trait_def:tt)*
        }

        $($tail:tt)*
    ) => {
        $crate::__implement_trait! {
            @ $trait ($ty) $ty_args { $($trait_def)* }
        }

        $crate::implement! { $($tail)* }
    };

    () => {};
}
