//! `Display` implementations for all relevant types.

prelude! {}

base::implement! {
    impl Display for Span {
        |&self, fmt| write!(fmt, "{}-{}", self.start, self.end)
    }

    impl Display for Loc {
        |&self, fmt| write!(fmt, "`{}`:{}:{}", self.file, self.line, self.span)
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
