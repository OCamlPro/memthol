//! `Display` implementations for all relevant types.

prelude! {}

use base::impl_display;

impl_display! {
    fmt(&self, fmt)

    Uid = self.uid.fmt(fmt);

    SinceStart = {
        let date = self;
        let mut secs = date.as_secs();
        let mut mins = secs / 60;
        secs = secs - mins * 60;
        let hours = mins / 60;
        mins = mins - hours * 60;

        if hours > 0 {
            write!(fmt, "{}h", hours)?
        }
        if mins > 0 {
            write!(fmt, "{}m", mins)?
        }
        write!(fmt, "{}", secs)?;
        let millis = date.subsec_millis();
        if millis != 0 {
            write!(fmt, ".{}", millis)?
        }
        write!(fmt, "s")
    }
    Date = write!(fmt, "{}", self.date());

    Span = write!(fmt, "{}-{}", self.start, self.end);

    Loc = write!(fmt, "`{}`:{}:{}", self.file, self.line, self.span);
    CLoc = write!(fmt, "{}#{}", self.loc, self.cnt);

    AllocKind = {
        write!(fmt, "{}", self.as_str())
    }

    Alloc = {
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

    Diff = {
        write!(fmt, "{}; new: {{\n", self.time)?;
        for alloc in &self.new {
            write!(fmt, "    {},\n", alloc)?
        }
        write!(fmt, "}};\ndead {{\n")?;
        for (uid, date) in &self.dead {
            write!(fmt, "    #{}: {},\n", uid, date)?
        }
        write!(fmt, "}}\n")
    }

    Init = {
        writeln!(fmt, "start: {}", self.start_time)?;
        writeln!(fmt, "word_size: {}", self.word_size)?;
        Ok(())
    }
}
