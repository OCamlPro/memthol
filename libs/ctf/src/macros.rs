#[macro_export]
macro_rules! bail {
    ($e:expr) => {
        return Err($e.into());
    };
    ($($fmt:tt)*) => {
        return Err(format!($($fmt)*));
    };
}

#[macro_export]
macro_rules! prelude {
    () => {
        use $crate::prelude::*;
    };
}

macro_rules! err {
    (|| $($tail:tt)*) => {
        || { err!($($tail)*) }
    };

    (expected $exp:expr) => {
        err!(expected $exp, found "EOF")
    };
    (expected $exp:expr, found $found:expr) => {
        format!("expected `{}`, found `{}`", $exp, $found)
    };
}

// macro_rules! back_return {
//     ($parser:expr => $pos:expr, $res:expr) => {{
//         $parser.backtrack($pos);
//         return $res;
//     }};
// }
