#[macro_export]
macro_rules! prelude {
    () => {
        use $crate::prelude::*;
    };
}

/// Applies some action to a `CtfParser` applied to some bytes.
#[macro_export]
macro_rules! parse {
    (
        $bytes:expr => |$parser_pat:pat| $action:expr
    ) => {{
        match $crate::parse::CtfParser::new($bytes)? {
            $crate::prelude::Either::Left($parser_pat) => $action,
            $crate::prelude::Either::Right($parser_pat) => $action,
        }
    }};
}

macro_rules! parser_do {
    (
        $parser_disj:expr => join |$parser_pat:pat| {
            $($stuff:tt)*
        }
    ) => {
        match $parser_disj {
            $crate::prelude::Either::Left($parser_pat) => {
                $($stuff)*
            }
            $crate::prelude::Either::Right($parser_pat) => {
                $($stuff)*
            }
        }
    };
    (
        $parser_disj:expr => map |mut $parser_id:ident| {
            $($stuff:tt)*
        }
    ) => {
        match $parser_disj {
            $crate::prelude::Either::Left(mut $parser_id) => $crate::prelude::Either::Left({
                $($stuff)*
            }),
            $crate::prelude::Either::Right(mut $parser_id) => $crate::prelude::Either::Right({
                $($stuff)*
            }),
        }
    };
}

macro_rules! parse_error {
    (|| $($tail:tt)*) => {
        || { parse_error!($($tail)*) }
    };

    (expected $exp:expr) => {
        parse_error!(expected $exp, found "EOF")
    };
    (expected $exp:expr, found $found:expr) => {
        format!("expected `{}`, found `{}`", $exp, $found)
    };
}

#[cfg(not(debug_assertions))]
macro_rules! pinfo {
    ($($stuff:tt)*) => {
        ()
    };
}
#[cfg(debug_assertions)]
macro_rules! pinfo {
    ($parser:expr, $($blah:tt)*) => {if $crate::VERB {
        let (pos, max) = $parser.real_position();
        $crate::prelude::log::info!("[{}/{}] {}", pos, max, format_args!($($blah)*))
    }};
}
#[cfg(not(debug_assertions))]
macro_rules! pdebug {
    ($($stuff:tt)*) => {
        ()
    };
}
#[cfg(debug_assertions)]
macro_rules! pdebug {
    ($parser:expr, $($blah:tt)*) => {if $crate::DEBUG_VERB {
        let (pos, max) = $parser.real_position();
        $crate::prelude::log::debug!("[{}/{}] {}", pos, max, format_args!($($blah)*))
    }};
}
