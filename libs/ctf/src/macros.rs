#[macro_export]
macro_rules! prelude {
    () => {
        use $crate::prelude::*;
    };
}

#[macro_export]
macro_rules! parse {
    (
        $parser_disj:expr => join |mut $parser_id:ident| {
            $($stuff:tt)*
        }
    ) => {
        match $parser_disj {
            $crate::prelude::Either::Left(mut $parser_id) => {
                $($stuff)*
            }
            $crate::prelude::Either::Right(mut $parser_id) => {
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

#[cfg(not(any(test, not(release))))]
macro_rules! pinfo {
    ($($stuff:tt)*) => {
        ()
    };
}
#[cfg(any(test, not(release)))]
macro_rules! pinfo {
    ($parser:expr, $($blah:tt)*) => {if $crate::VERB {
        let (pos, max) = $parser.position();
        println!("[{}/{}] {}", pos, max, format_args!($($blah)*))
    }};
}
#[cfg(not(any(test, not(release))))]
macro_rules! pdebug {
    ($($stuff:tt)*) => {
        ()
    };
}
#[cfg(any(test, not(release)))]
macro_rules! pdebug {
    ($parser:expr, $($blah:tt)*) => {if $crate::DEBUG_VERB {
        let (pos, max) = $parser.position();
        println!("[{}/{}] {}", pos, max, format_args!($($blah)*))
    }};
}
