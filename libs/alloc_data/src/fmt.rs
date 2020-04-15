//! `Display` implementations for all relevant types.

use crate::*;

base::impl_display! {
    fmt(&self, fmt)

    Uid = self.uid.fmt(fmt);

    Span = write!(fmt, "{}-{}", self.start, self.end);

    Loc = write!(fmt, "`{}`:{}:{}", self.file, self.line, self.span);

    Labels = {
        write!(fmt, "[")?;
        for label in &self.labels {
            write!(fmt, " `{}`", label)?
        }
        write!(fmt, " ]")
    }

    AllocKind = {
        write!(fmt, "{}", self.as_str())
    }

    Alloc = {
        let my_labels = labels::get(self.labels);
        let my_trace = locs::get(self.trace);
        let mut labels = "[".to_string();
        for label in my_labels.iter() {
            labels.push_str(" ");
            labels.push_str(label)
        }
        labels.push_str(" ]");
        write!(fmt, "{}: {}, {}, ", self.uid, self.kind, self.size)?;

        // Write the trace.
        write!(fmt, "[")?;
        for (loc, count) in my_trace.iter() {
            write!(fmt, " {}#{}", loc, count)?
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

    SinceStart = {
        let mut nanos = format!(".{:>09}", (*self).subsec_nanos());
        // Remove trailing zeros.
        loop {
            match nanos.pop() {
                // Remove zeros.
                Some('0') => (),
                // There was nothing but zeros, remove dot as well (last character).
                Some('.') => break,
                // Otherwise it's a number, we must keep it and stop removing stuff.
                Some(c) => {
                    nanos.push(c);
                    break;
                }
                None => unreachable!(),
            }
        }
        write!(fmt, "{}{}", (*self).as_secs(), nanos)
    }

    Date = write!(fmt, "{}", self.date());
}