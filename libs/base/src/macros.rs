//! Macros used throughout the whole project.

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
