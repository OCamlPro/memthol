//! Macros.

/// Opens the prelude.
macro_rules! prelude {
    () => {
        use crate::common::*;
    };
}

/// Issues an alert.
#[macro_export]
macro_rules! alert {
    ($msg:expr) => (
        $crate::js::alert($msg)
    );
    ($($stuff:tt)*) => (
        $crate::js::alert(
            &format_args!($($stuff)*).to_string()
        )
    );
}

/// Generates inline-CSS styles.
#[macro_export]
macro_rules! css {
    (fmt: $fmt:expr,
        $(
            $($id:ident)+ $(
                ($($args:tt)*)
            )?
        ),*
        $(,)?
    ) => {{
        #![allow(unused_must_use)]
        $(
            $crate::css!(@($fmt)
                $($id)+ $( ($($args)*) )?
            );
        )*
    }};

    ($str:expr,
        $(
            $($id:ident)+ $(
                ($($args:tt)*)
            )?
        ),*
        $(,)?
    ) => {{
        #![allow(unused_must_use)]
        // We're going to ignore all write-results because we're writing to a string.
        let str: &mut String = $str;
        $(
            $crate::css!(@(str)
                $($id)+ $( ($($args)*) )?
            );
        )*
    }};

    (@arg $block:block) => (&$block.to_string());
    (@arg $token:tt) => (stringify!($token));

    (@($str:expr)
        if(
            $e:expr,
            $($thn_id:ident)+ $(
                ($($thn_args:tt)*)
            )?
            $(,)?
        )
    ) => {{
        if $e {
            $crate::css!($str, $($thn_id)+ $(($($thn_args)*))?)
        }
    }};
    (@($str:expr)
        if(
            $e:expr,
            {
                $($thn_args:tt)*
            }
            $(,)?
        )
    ) => {{
        if $e {
            $crate::css!($str, $($thn_args)*)
        }
    }};
    (@($str:expr)
        if(
            $e:expr,
            $($thn_id:ident)+ $(
                ($($thn_args:tt)*)
            )?
            ,
            else $($els_id:ident)+ $(
                ($($els_args:tt)*)
            )?
            $(,)?
        )
    ) => {{
        if $e {
            $crate::css!($str, $($thn_id)+ $(($($thn_args)*))?)
        } else {
            $crate::css!($str, $($els_id)+ $(($($els_args)*))?)
        }
    }};

    // #overflow
    (@($str:expr) overflow($($args:tt)*)) => {{
        write!($str, "overflow: ");
        write!($str, "{}",
            $crate::css!(@overflow($($args)*))
        );
    }};
    (@overflow(scroll)) => ("scroll; ");
    (@overflow(hidden)) => ("hidden; ");
    (@overflow(auto)) => ("auto; ");
    (@overflow(visible)) => ("visible; ");

    // #display
    (@($str:expr) display($($args:tt)*)) => {{
        write!($str, "display: {}; ", $crate::css!(@display($($args)*)));
    }};
    (@display(block)) => ("block");
    (@display(table)) => ("table");
    (@display(table cell)) => ("table-cell");
    (@display(table row)) => ("table-row");
    (@display(inline block)) => ("inline-block");
    (@display(list item)) => ("list-item");
    (@display(none)) => ("none");
    (@display(flex)) => ("flex");
    (@($str:expr) block) => ($crate::css!(@($str) display(block)));
    (@($str:expr) table) => ($crate::css!(@($str) display(table)));
    (@($str:expr) table cell) => ($crate::css!(@($str) display(table cell)));
    (@($str:expr) table row) => ($crate::css!(@($str) display(table row)));
    (@($str:expr) inline block) => ($crate::css!(@($str) display(inline block)));
    (@($str:expr) list item) => ($crate::css!(@($str) display(list item)));
    (@($str:expr) none) => ($crate::css!(@($str) display(none)));
    (@($str:expr) flex) => ($crate::css!(@($str) display(flex)));

    // #visi
    (@($str:expr) visi($($args:tt)*)) => {{
        write!($str, "visibility: {}; ", $crate::css!(@visi($($args)*)));
    }};
    (@visi(visible)) => ("visible");
    (@visi(hidden)) => ("hidden");
    (@visi(collapse)) => ("collapse");
    (@visi(initial)) => ("initial");
    (@visi(inherit)) => ("inherit");
    (@($str:expr) visible) => ($crate::css!(@($str) visi(visible)));
    (@($str:expr) hidden) => ($crate::css!(@($str) visi(hidden)));
    (@($str:expr) collapse) => ($crate::css!(@($str) visi(collapse)));
    (@($str:expr) initial) => ($crate::css!(@($str) visi(initial)));
    (@($str:expr) inherit) => ($crate::css!(@($str) visi(inherit)));

    // #cursor
    (@($str:expr) cursor($($args:tt)*)) => {{
        write!($str, "cursor: {}; ", $crate::css!(@cursor($($args)*)));
    }};
    (@($str:expr) alias) => ($crate::css!(@($str) cursor(alias)));
    (@cursor(alias)) => ("alias");
    (@($str:expr) all scroll) => ($crate::css!(@($str) cursor(all-scroll)));
    (@cursor(all scroll)) => ("all-scroll");
    (@($str:expr) auto) => ($crate::css!(@($str) cursor(auto)));
    (@cursor(auto)) => ("auto");
    (@($str:expr) cell) => ($crate::css!(@($str) cursor(cell)));
    (@cursor(cell)) => ("cell");
    (@($str:expr) context menu) => ($crate::css!(@($str) cursor(context menu)));
    (@cursor(context menu)) => ("context-menu");
    (@($str:expr) col resize) => ($crate::css!(@($str) cursor(col resize)));
    (@cursor(col resize)) => ("col-resize");
    (@($str:expr) copy) => ($crate::css!(@($str) cursor(copy)));
    (@cursor(copy)) => ("copy");
    (@($str:expr) crosshair) => ($crate::css!(@($str) cursor(crosshair)));
    (@cursor(crosshair)) => ("crosshair");
    (@($str:expr) default) => ($crate::css!(@($str) cursor(default)));
    (@cursor(default)) => ("default");
    (@($str:expr) e resize) => ($crate::css!(@($str) cursor(e resize)));
    (@cursor(e resize)) => ("e-resize");
    (@($str:expr) ew resize) => ($crate::css!(@($str) cursor(ew resize)));
    (@cursor(ew resize)) => ("ew-resize");
    (@($str:expr) grab) => ($crate::css!(@($str) cursor(grab)));
    (@cursor(grab)) => ("-webkit-grab; cursor: grab");
    (@($str:expr) grabbing) => ($crate::css!(@($str) cursor(grabbing)));
    (@cursor(grabbing)) => ("-webkit-grabbing; cursor: grabbing");
    (@($str:expr) help) => ($crate::css!(@($str) cursor(help)));
    (@cursor(help)) => ("help");
    (@($str:expr) move) => ($crate::css!(@($str) cursor(move)));
    (@cursor(move)) => ("move");
    (@($str:expr) n resize) => ($crate::css!(@($str) cursor(n resize)));
    (@cursor(n resize)) => ("n-resize");
    (@($str:expr) ne resize) => ($crate::css!(@($str) cursor(ne resize)));
    (@cursor(ne resize)) => ("ne-resize");
    (@($str:expr) nesw resize) => ($crate::css!(@($str) cursor(nesw resize)));
    (@cursor(nesw resize)) => ("nesw-resize");
    (@($str:expr) ns resize) => ($crate::css!(@($str) cursor(ns resize)));
    (@cursor(ns resize)) => ("ns-resize");
    (@($str:expr) nw resize) => ($crate::css!(@($str) cursor(nw resize)));
    (@cursor(nw resize)) => ("nw-resize");
    (@($str:expr) nwse resize) => ($crate::css!(@($str) cursor(nwse resize)));
    (@cursor(nwse resize)) => ("nwse-resize");
    (@($str:expr) no drop) => ($crate::css!(@($str) cursor(no drop)));
    (@cursor(no drop)) => ("no-drop");
    (@($str:expr) none) => ($crate::css!(@($str) cursor(none)));
    (@cursor(none)) => ("none");
    (@($str:expr) not allowed) => ($crate::css!(@($str) cursor(not allowed)));
    (@cursor(not allowed)) => ("not-allowed");
    (@($str:expr) pointer) => ($crate::css!(@($str) cursor(pointer)));
    (@cursor(pointer)) => ("pointer");
    (@($str:expr) progress) => ($crate::css!(@($str) cursor(progress)));
    (@cursor(progress)) => ("progress");
    (@($str:expr) row resize) => ($crate::css!(@($str) cursor(row resize)));
    (@cursor(row resize)) => ("row-resize");
    (@($str:expr) s resize) => ($crate::css!(@($str) cursor(s resize)));
    (@cursor(s resize)) => ("s-resize");
    (@($str:expr) se resize) => ($crate::css!(@($str) cursor(se resize)));
    (@cursor(se resize)) => ("se-resize");
    (@($str:expr) sw resize) => ($crate::css!(@($str) cursor(sw resize)));
    (@cursor(sw resize)) => ("sw-resize");
    (@($str:expr) text) => ($crate::css!(@($str) cursor(text)));
    (@cursor(text)) => ("text");
    (@($str:expr) w resize) => ($crate::css!(@($str) cursor(w resize)));
    (@cursor(w resize)) => ("w-resize");
    (@($str:expr) wait) => ($crate::css!(@($str) cursor(wait)));
    (@cursor(wait)) => ("wait");
    (@($str:expr) zoom in) => ($crate::css!(@($str) cursor(zoom in)));
    (@cursor(zoom in)) => ("zoom-in");
    (@($str:expr) zoom out) => ($crate::css!(@($str) cursor(zoom out)));
    (@cursor(zoom out)) => ("zoom-out");
    (@($str:expr) alias) => ($crate::css!(@($str) cursor(alias)));

    //#table_layout
    (@($str:expr) table_layout($($args:tt)*)) => {{
        write!($str, "table-layout: {}; ", $crate::css!(@display($($args)*)));
    }};
    (@display(fixed)) => ("fixed");
    (@display(auto)) => ("auto");

    // #margin
    (@($str:expr) margin(none)) => {{
        write!($str, "margin: 0; ");
    }};
    (@($str:expr) margin(auto)) => {{
        write!($str, "margin: auto; ");
    }};
    (@($str:expr) margin(
        $($margin:tt $unit:tt),* $(,)*
    )) => {{
        write!($str, "margin:");
        $(
            write!($str, " {}{}", $crate::css!(@arg $margin), stringify!($unit));
        )*
        write!($str, "; ");
    }};
    (@($str:expr) margin_bottom(
        $margin:tt $unit:tt $(,)*
    )) => {{
        write!($str, "margin-bottom: {}{};", $crate::css!(@arg $margin), stringify!($unit));
    }};
    (@($str:expr) margin_top(
        $margin:tt $unit:tt $(,)*
    )) => {{
        write!($str, "margin-top: {}{};", $crate::css!(@arg $margin), stringify!($unit));
    }};
    (@($str:expr) margin_left(
        $margin:tt $unit:tt $(,)*
    )) => {{
        write!($str, "margin-left: {}{};", $crate::css!(@arg $margin), stringify!($unit));
    }};
    (@($str:expr) margin_right(
        $margin:tt $unit:tt $(,)*
    )) => {{
        write!($str, "margin-right: {}{};", $crate::css!(@arg $margin), stringify!($unit));
    }};

    // #padding
    (@($str:expr) padding(none)) => {{
        write!($str, "padding: 0; ");
    }};
    (@($str:expr) padding(
        $($val:tt $unit:tt),* $(,)?
    )) => {{
        write!($str, "padding: ");
        $(
            write!($str, "{}", $crate::css!(@arg $val));
            write!($str, concat!(stringify!($unit), " "));
        )*
        write!($str, "; ");
    }};

    // #height
    (@($str:expr) height(auto)) => {
        write!($str, "height: auto; ");
    };
    (@($str:expr) height($val:tt $unit:tt)) => {{
        write!($str, "height: ");
        write!($str, "{}", $crate::css!(@arg $val));
        write!($str, concat!(stringify!($unit), "; "));
    }};
    (@($str:expr) height(min $val:tt $unit:tt)) => {{
        write!($str, "min-");
        $crate::css!(@($str) height($val $unit))
    }};
    (@($str:expr) height(max $val:tt $unit:tt)) => {{
        write!($str, "max-");
        $crate::css!(@($str) height($val $unit))
    }};
    (@($str:expr) height(between $min_val:tt $min_unit:tt and $max_val:tt $max_unit:tt)) => {{
        $crate::css!(@($str) height(min $min_val $min_unit));
        $crate::css!(@($str) height(max $max_val $max_unit));
    }};

    // #width
    (@($str:expr) width(auto)) => {
        write!($str, "width: auto; ");
    };
    (@($str:expr) width($val:tt $unit:tt)) => {{
        write!($str, "width: ");
        write!($str, "{}", $crate::css!(@arg $val));
        write!($str, concat!(stringify!($unit), "; "));
    }};
    (@($str:expr) width(min $val:tt $unit:tt)) => {{
        write!($str, "min-");
        $crate::css!(@($str) width($val $unit))
    }};
    (@($str:expr) width(max $val:tt $unit:tt)) => {{
        write!($str, "max-");
        $crate::css!(@($str) width($val $unit))
    }};
    (@($str:expr) width(between $min_val:tt $min_unit:tt and $max_val:tt $max_unit:tt)) => {{
        $crate::css!(@($str) width(min $min_val $min_unit));
        $crate::css!(@($str) width(max $max_val $max_unit));
    }};

    // #border
    (@($str:expr) border (none)) => {{
        write!($str, "border: 0; ")
    }};
    (@($str:expr) border (
        $w:tt $unit:tt, $color:tt
    )) => {{
        write!(
            $str,
            "border: {}{} double {}; ",
            $crate::css!(@arg $w),
            stringify!($unit),
            $crate::css!(@color($color))
        );
    }};
    (@($str:expr) border (
        left, $w:tt $unit:tt, $color:tt
    )) => {{
        write!(
            $str,
            "border-left: {}{} double {}; ",
            $crate::css!(@arg $w),
            stringify!($unit),
            $crate::css!(@color($color))
        );
    }};
    (@($str:expr) border (
        right, $w:tt $unit:tt, $color:tt
    )) => {{
        write!(
            $str,
            "border-right: {}{} double {}; ",
            $crate::css!(@arg $w),
            stringify!($unit),
            $crate::css!(@color($color))
        );
    }};
    (@($str:expr) border (
        top, $w:tt $unit:tt, $color:tt
    )) => {{
        write!(
            $str,
            "border-top: {}{} double {}; ",
            $crate::css!(@arg $w),
            stringify!($unit),
            $crate::css!(@color($color))
        );
    }};
    (@($str:expr) border (
        bottom, $w:tt $unit:tt, $color:tt
    )) => {{
        write!(
            $str,
            "border-bottom: {}{} double {}; ",
            $crate::css!(@arg $w),
            stringify!($unit),
            $crate::css!(@color($color))
        );
    }};

    // #border_radius
    (@($str:expr) border_radius(
        $($val:tt $unit:tt),* $(,)?
    )) => {{
        write!($str, "border-radius:");
        $(
            write!($str, " {}", $crate::css!(@arg $val));
            write!($str, stringify!($unit));
        )*
        write!($str, "; ");
    }};

    (@($str:expr) float($($args:tt)*)) => {{
        write!(
            $str,
            "float: {}; ",
            $crate::css!(@float($($args)*))
        );
    }};
    (@float(left)) => ("left");
    (@float(right)) => ("right");
    (@float(none)) => ("none");
    (@float(inherit)) => ("inherit");

    // #font_outline
    (@($str:expr) font_outline ($color:tt)) => {{
        let color = $crate::css!(@color($color));
        write!($str, "text-shadow: -1px 0 {0}, 0 1px {0}, 1px 0 {0}, 0 -1px {0}; ", color);
    }};

    (@($str:expr) box_shadow (
        $(
            $hlen:tt $hlen_unit:tt,
            $vlen:tt $vlen_unit:tt,
            $blur:tt $blur_unit:tt,
            $spread:tt $spread_unit:tt,
            $color:tt
            $(, inset $($dont_care:ident)?)?
            $(,)?
        );* $(;)?
    )) => {{
        write!(
            $str,
            "box-shadow:"
        );
        let _pref = " ";
        $(
            $(
                $($dont_care;)?
                write!($str, "{} inset", _pref);
                let _pref = " ";
            )?
            write!($str,
                "{} {}{} {}{} {}{} {}{} ",
                _pref,
                $crate::css!(@arg $hlen), stringify!($hlen_unit),
                $crate::css!(@arg $vlen), stringify!($vlen_unit),
                $crate::css!(@arg $blur), stringify!($blur_unit),
                $crate::css!(@arg $spread), stringify!($spread_unit),
            );
            let _pref = ", ";
            $crate::css!(@($str) write_color($color));
        )*
        write!($str, "; ");
    }};

    // #justify_content
    (@($str:expr) justify_content($pos:tt)) => (
        write!($str, "justify-content: ");
        write!($str, "{}", $crate::css!(@text_align($pos)));
        write!($str, "; ");
    );
    (@justify_content(center)) => ("center");
    (@justify_content(left)) => ("left");
    (@justify_content(right)) => ("right");

    // #text_align
    (@($str:expr) text_align($pos:tt)) => (
        write!($str, "text-align: ");
        write!($str, "{}", $crate::css!(@text_align($pos)));
        write!($str, "; ");
    );
    (@text_align(center)) => ("center");
    (@text_align(left)) => ("left");
    (@text_align(right)) => ("right");

    // #top
    (@($str:expr) top) => (
        write!($str, "top: 0;");
    );
    // #bottom
    (@($str:expr) bottom) => (
        write!($str, "bottom: 0;");
    );

    // #vertical_align
    (@($str:expr) vertical_align($pos:tt)) => (
        write!($str, "vertical-align: ");
        write!($str, "{}", $crate::css!(@vertical_align($pos)));
        write!($str, "; ")
    );
    (@vertical_align(baseline)) => ("baseline");
    (@vertical_align(top)) => ("top");
    (@vertical_align(middle)) => ("middle");
    (@vertical_align(bottom)) => ("bottom");
    (@vertical_align(sub)) => ("sub");
    (@vertical_align(text top)) => ("text-top");

    // #ws
    (@($str:expr) no_wrap) => (
        write!($str, "white-space: nowrap; ");
    );

    // #pos
    (@($str:expr) pos($($args:tt)*)) => {{
        write!($str, "position: ");
        write!($str, "{}",
            $crate::css!(@pos($($args)*))
        );
    }};
    (@pos(relative)) => ("relative; ");
    (@pos(absolute)) => ("absolute; ");
    (@pos(fixed)) => ("fixed; ");
    (@($str:expr) fixed(bottom)) => {{
        $crate::css!(@($str) pos(fixed));
        write!($str, "bottom: 0; ");
    }};

    // #z_index
    (@($str:expr) z_index($val:tt)) => {{
        write!($str, "z-index: ");
        write!($str, "{}", $crate::css!(@arg $val));
        write!($str, "; ");
    }};

    // #text_shadow
    (@($str:expr) text_shadow($($args:tt)*)) => {{
        let args = $crate::css!(@arg $($args)*);
        write!($str, "text-shadow: -1px 0 ");
        write!($str, "{}", args);
        write!($str, ", 0 1px ");
        write!($str, "{}", args);
        write!($str, ", 1px 0 ");
        write!($str, "{}", args);
        write!($str, ", 0 -1px ");
        write!($str, "{}", args);
        write!($str, "; ");
    }};

    // #font
    (@($str:expr) font(code)) => {{
        write!(
            $str,
            "font: {} Monaco, Consola, sans-serif; ",
            $crate::css!(@font(default))
        );
    }};
    (@($str:expr) font(code, $val:tt $unit:tt)) => {{
        write!(
            $str,
            "font: {}{} Monaco, Consola, sans-serif; ",
            $crate::css!(@arg $val),
            stringify!($unit)
        );
    }};
    (@($str:expr) font($val:tt $unit:tt)) => {{
        write!(
            $str,
            "font: {} SF Pro Display, SF Pro Icons, Helvetica Neue, Helvetica, Arial, sans-serif; ",
            $crate::css!(@arg $val),
            stringify!($unit),
        );
    }};
    (@($str:expr) font($($args:tt)*)) => {{
        write!($str, "font: ");
        write!($str, "{}",
            $crate::css!(@font($($args)*))
        );
        write!($str, "{}",
            " SF Pro Display, SF Pro Icons, Helvetica Neue, Helvetica, Arial, sans-serif; "
        );
    }};
    (@font(default)) => ("16px");

    // #font_size
    (@($str:expr) font_size($val:tt $unit:tt)) => {{
        write!($str, "font-size: {}{}; ", $crate::css!(@arg $val), stringify!($unit));
    }};

    // #font_style
    (@($str:expr) font_style($($args:tt)*)) => {{
        write!($str, "font-style: {}; ", $crate::css!(@font_style($($args)*)));
    }};
    (@font_style(normal)) => ("normal");
    (@font_style(italic)) => ("italic");
    (@font_style(oblique)) => ("oblique");
    (@($str:expr) italic) => {
        $crate::css!(@($str) font_style(italic))
    };
    (@($str:expr) oblique) => {
        $crate::css!(@($str) font_style(oblique))
    };

    // #font_weight
    (@($str:expr) font_weight($($args:tt)*)) => {{
        write!($str, "font-weight: {}; ", $crate::css!(@font_weight($($args)*)));
    }};
    (@font_weight(normal)) => ("normal");
    (@font_weight(bold)) => ("bold");
    (@($str:expr) bold) => {
        $crate::css!(@($str) font_weight(bold))
    };

    // #text_decoration
    (@($str:expr) text_decoration($($args:tt)*)) => {{
        write!($str, "text-decoration: {}; ", $crate::css!(@text_decoration($($args)*)));
    }};
    (@text_decoration(normal)) => ("normal");
    (@text_decoration(bold)) => ("bold");
    (@($str:expr) bold) => {
        $crate::css!(@($str) text_decoration(bold))
    };
    (@text_decoration(overline)) => ("overline");
    (@($str:expr) overline) => (crate::css!(@($str) text_decoration(overline)));
    (@text_decoration(line through)) => ("line-through");
    (@($str:expr) line through) => (crate::css!(@($str) text_decoration(line through)));
    (@text_decoration(underline)) => ("underline");
    (@($str:expr) underline) => (crate::css!(@($str) text_decoration(underline)));
    (@text_decoration(underline overline)) => ("underline overline");
    (@($str:expr) underline overline) => (crate::css!(@($str) text_decoration(underline overline)));

    // #bg
    (@($str:expr) bg($color:tt)) => {{
        write!($str, "background-color: ");
        write!($str, "{}", $crate::css!(@arg $color));
        write!($str, ";");
    }};
    (@($str:expr) bg(gradient $color_src:tt to $color_tgt:tt)) => {{
        write!($str, "background-image: linear-gradient(");
        write!($str, "{}", $crate::css!(@color($color_src)));
        write!($str, ", ");
        write!($str, "{}", $crate::css!(@color($color_tgt)));
        write!($str, "); ")
    }};
    // (@($str:expr) bg(radial gradient $($color:tt)) => {{
    //     write!($str, "background-image: radial-gradient(circle, ");
    //     write!($str, "{}", $crate::css!(@color($color_src)));
    //     write!($str, ", ");
    //     write!($str, "{}", $crate::css!(@color($color_tgt)));
    //     write!($str, "); ")
    // }};

    // #fg
    (@($str:expr) fg($color:tt)) => {{
        write!($str, "color: ");
        write!($str, "{}", $crate::css!(@arg $color));
        write!($str, ";");
    }};

    // #color
    (@color(transparent)) => ("rgba(255,0,0,0)");
    (@color(black)) => ("black");
    (@color(white)) => ("white");
    (@color($block:block)) => ($block);
    (@($str:expr) write_color(($r:tt, $g:tt, $b:tt))) => {
        write!(
            $str,
            "rgb({},{},{})",
            $crate::css!(@arg $r),
            $crate::css!(@arg $g),
            $crate::css!(@arg $b),
        );
    };
    (@($str:expr) write_color(($r:tt, $g:tt, $b:tt, $a:tt))) => {
        write!(
            $str,
            "rgba({},{},{}, {})",
            $crate::css!(@arg $r),
            $crate::css!(@arg $g),
            $crate::css!(@arg $b),
            $crate::css!(@arg $a),
        );
    };
    (@($str:expr) write_color($stuff:tt)) => {
        write!($str, "{}", $crate::css!(@color($stuff)));
    };

    // #extends
    (@($str:expr) extends($style:ident)) => (
        $style!($str)
    );

    (@($str:expr)) => ();
}

#[macro_export]
macro_rules! inline_css {
    ($($stuff:tt)*) => {{
        use std::fmt::Write;
        let mut str = String::with_capacity(42);
        $crate::css!(&mut str, $($stuff)*);
        str.shrink_to_fit();
        str
    }};
}

#[macro_export]
macro_rules! define_style {
    (
        $(#[$meta:meta])*
        $name:ident = { $($stuff:tt)* };

        $($tail:tt)*
    ) => {
        lazy_static::lazy_static! {
            $(#[$meta])*
            static ref $name: String = $crate::inline_css!($($stuff)*);
        }

        $crate::define_style! { $($tail)* }
    };
    (
        $(#[$meta:meta])*
        pub $name:ident = { $($stuff:tt)* };

        $($tail:tt)*
    ) => {
        lazy_static::lazy_static! {
            $(#[$meta])*
            pub static ref $name: String = $crate::inline_css!($($stuff)*);
        }

        $crate::define_style! { $($tail)* }
    };

    (
        $(#[$meta:meta])*
        $name:ident ! = { $($def:tt)* };

        $($tail:tt)*
    ) => {
        $(#[$meta])*
        macro_rules! $name {
            ($str:expr) => ($crate::css!($str, $($def)*));
        }

        $crate::define_style! { $($tail)* }
    };

    () => {};
}
