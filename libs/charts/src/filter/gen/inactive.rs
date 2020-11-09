//! Filter generation strategy that generates nothing.

use super::*;

/// Unit-struct handling the inactive generator.
pub struct Inactive;

impl FilterGenExt for Inactive {
    type Params = ();

    const KEY: &'static str = "none";
    const FMT: Option<&'static str> = None;

    fn work(_data: &data::Data, (): Self::Params) -> Res<(Filters, Vec<chart::Chart>)> {
        let filters = Filters::new();
        let charts = chart_gen::single(&filters)?;
        Ok((filters, charts))
    }

    fn parse_args(parser: Option<Parser>) -> Option<FilterGen> {
        if parser.is_none() {
            Some(FilterGen::Inactive)
        } else {
            None
        }
    }

    fn add_help(s: &mut String) {
        s.push_str(&format!(
            "\
- none: `{0}`
    Deactivates filter generation.

\
            ",
            Self::KEY,
        ));
    }
}
