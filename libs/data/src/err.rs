//! Error types, mostly for parse errors.

use std::fmt;

/// Parse error information.
#[derive(Debug, Clone)]
pub struct Position {
    /// Row (starts at 0).
    pub row: usize,
    /// Column (starts at 0).
    pub col: usize,
    /// Line of the error (no newline).
    pub line: String,
}
impl Position {
    /// Constructor.
    pub fn new(row: usize, col: usize, line: String) -> Self {
        Self { row, col, line }
    }
}
impl fmt::Display for Position {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        let line_idx = (self.row + 1).to_string();
        writeln!(fmt, " {0: <1$} |", "", line_idx.len())?;
        writeln!(fmt, " {} | {}", line_idx, self.line)?;
        write!(fmt, " {0: <1$} | {0: >2$}^", "", line_idx.len(), self.col)
    }
}

error_chain::error_chain! {
    types {
        Err, ErrorKind, ResultExt, Res;
    }

    errors {
        ParseErr(pos: Position, blah: String) {
            description("parse error")
            display(
                "{}, line {} column {}\n{}", blah, pos.row + 1, pos.col + 1, pos
            )
        }
    }
}

impl Err {
    /// Multi-line representation of a trace of errors.
    pub fn pretty(&self) -> String {
        let mut s = "error: ".to_string();

        // Reverse errors.
        let mut errs = vec![];
        for e in self.iter() {
            errs.push(e)
        }

        let mut is_first = true;
        for e in errs.into_iter().rev() {
            if is_first {
                is_first = false
            } else {
                s.push_str("\n")
            }
            s.push_str(&e.to_string())
        }

        s
    }
}
