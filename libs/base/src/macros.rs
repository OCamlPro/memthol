//! Macros used throughout the whole project.

#[macro_export]
macro_rules! time {
    ($e:expr, |$time:ident| $time_action:expr) => {{
        let start = std::time::Instant::now();
        let res = $e;
        let time = std::time::Instant::now() - start;
        let $time = format!("{}.{:0>9}s", time.as_secs(), time.subsec_nanos());
        $time_action;
        res
    }};
}

#[macro_export]
#[cfg(not(release))]
macro_rules! debug_do {
    ($($stuff:tt)*) => {{
        $($stuff)*
    }};
}
#[macro_export]
#[cfg(release)]
macro_rules! debug_do {
    ($($stuff:tt)*) => {{
        $($stuff)*
    }};
}

#[macro_export]
macro_rules! implement {
    (
        $(
            $trait:ident $( ($($args:tt)*) )? {
                $($def:tt)*
            }
        )*
    ) => {
        $(
            $crate::implement! {
                @ $trait $(( $($args)* ))? { $($def)* }
            }
        )*
    };

    (@Display {
        $( $ty:ty => |&$slf:ident, $fmt:pat| $def:expr ),* $(,)?
    }) => {
        $(
            impl std::fmt::Display for $ty {
                fn fmt(&$slf, $fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
                    $def
                }
            }
        )*
    };

    (@From {
        $( $src:ty, to $tgt:ty => |$param:pat| $def:expr ),* $(,)?
    }) => {
        $(
            impl std::convert::From<$src> for $tgt {
                fn from($param: $src) -> Self {
                    $def
                }
            }
        )*
    };
}

#[macro_export]
macro_rules! impl_display {
    (
        fmt(&$slf:ident, $fmt:ident)

        $ty:ident $([
            $($t_param:tt)*
        ])? = $def:block

        $($tail:tt)*
    ) => {
        impl std::fmt::Display for $ty $(<$($t_param)*>)? {
            fn fmt(&$slf, $fmt: &mut std::fmt::Formatter) -> std::fmt::Result $def
        }
        impl_display! {
            fmt(&$slf, $fmt)
            $($tail)*
        }
    };

    (
        fmt(&$slf:ident, $fmt:ident)

        $ty:ident $([
            $($t_param:tt)*
        ])? = $def:expr ;

        $($tail:tt)*
    ) => {
        impl std::fmt::Display for $ty $(<$($t_param)*>)? {
            fn fmt(&$slf, $fmt: &mut std::fmt::Formatter) -> std::fmt::Result { $def }
        }
        $crate::impl_display! {
            fmt(&$slf, $fmt)
            $($tail)*
        }
    };

    (
        fmt($($stuff:tt)*)
    ) => ();
}
