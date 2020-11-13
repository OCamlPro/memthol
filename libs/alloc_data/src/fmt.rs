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

//! `Display` implementations for all relevant types.

prelude! {}

base::implement! {
    impl Display for Loc {
        |&self, fmt| write!(fmt,
            "`{}`:{}:{}-{}", self.file, self.line, self.span.lbound, self.span.ubound
        )
    }

    impl Display for CLoc {
        |&self, fmt| write!(fmt, "{}#{}", self.loc, self.cnt)
    }

    impl Display for AllocKind {
        |&self, fmt| write!(fmt, "{}", self.as_str())
    }

    impl Display for Alloc {
        |&self, fmt| {
            let my_labels = self.labels.get();
            let my_trace = self.trace.get();

            write!(fmt, "{}: {}, {}, ", self.uid, self.kind, self.size)?;

            // Write the trace.
            write!(fmt, "[")?;
            for cloc in my_trace.iter() {
                write!(fmt, " {}#{}", cloc.loc, cloc.cnt)?
            }
            write!(fmt, " ], ")?;

            // Write the labels.
            write!(fmt, "[")?;
            for label in my_labels.iter() {
                write!(fmt, " {}", label)?
            }
            write!(fmt, " ], ")?;

            write!(fmt, "{}, ", self.toc)?;

            if let Some(tod) = &self.tod {
                write!(fmt, "{}", tod)?
            } else {
                write!(fmt, "_")?
            }
            write!(fmt, " }}")
        }
    }
}
