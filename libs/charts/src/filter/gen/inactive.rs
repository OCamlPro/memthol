/*<LICENSE>
    This file is part of Memthol.

    Copyright (C) 2020 OCamlPro.

    Memthol is free software: you can redistribute it and/or modify
    it under the terms of the GNU General Public License as published by
    the Free Software Foundation, either version 3 of the License, or
    (at your option) any later version.

    Memthol is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
    GNU General Public License for more details.

    You should have received a copy of the GNU General Public License
    along with Memthol.  If not, see <https://www.gnu.org/licenses/>.
*/

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
