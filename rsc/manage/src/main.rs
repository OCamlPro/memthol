use std::path::Path;

use error_chain::bail;

const DIRS: [&'static str; 2] = ["./memthol", "./libs"];
const LICENSE_PREF: &str = "/*<LICENSE>";
const LICENSE: &[u8] = include_bytes!("../license_header");

fn check() -> Res<()> {
    if LICENSE.len() < LICENSE_PREF.len() {
        bail!("license header and prefix do not match, please edit `rsc/manage/src/main.rs`")
    }
    if &LICENSE[0..LICENSE_PREF.len()] != LICENSE_PREF.as_bytes() {
        bail!("license header and prefix do not match, please edit `rsc/manage/src/main.rs`")
    }

    Ok(())
}

error_chain::error_chain! {
    types {
        Error, ErrorKind, ResultExt, Res;
    }

    foreign_links {
        Io(::std::io::Error);
    }
}

fn main() {
    let mut builder = pretty_env_logger::formatted_builder();

    // builder.filter(None, log::LevelFilter::Info);
    // builder.filter(None, log::LevelFilter::Debug);
    builder.init();

    match work() {
        Ok(()) => (),
        Err(e) => {
            for e in e.iter() {
                for (idx, line) in e.to_string().lines().enumerate() {
                    let pref = if idx == 0 { "" } else { "    " };
                    log::error!("{}{}", pref, line)
                }
            }
            std::process::exit(2)
        }
    }
}

fn work() -> Res<()> {
    check()?;
    for dir in &DIRS {
        let path = Path::new(dir);
        if !path.exists() {
            bail!("directory `{}` does not exist", dir)
        }
        if !path.is_dir() {
            bail!("expected directory, found `{}`")
        }
    }

    // At this point, all paths in `DIRS` denote directories.

    for dir in &DIRS {
        work_on_dir(dir).chain_err(|| format!("while working on directory `{}`", dir))?
    }

    Ok(())
}

fn work_on_dir(path: impl AsRef<Path>) -> Res<()> {
    let path = path.as_ref();
    log::info!("|===| working on directory `{}`", path.display());

    'rust_files: for entry in walkdir::WalkDir::new(path)
        .max_depth(usize::MAX)
        .into_iter()
        .filter_map(Result::ok)
    {
        let path = entry.path();

        if !path.is_file() {
            continue 'rust_files;
        }

        if let Some(ext) = path.extension().and_then(std::ffi::OsStr::to_str) {
            if ext != "rs" {
                continue 'rust_files;
            }
        } else {
            continue 'rust_files;
        }

        Rewriter::new().work_on_file(path)?
    }

    log::info!("|===|");

    Ok(())
}

pub enum HasLicense {
    No,
    Old,
    Current,
}

pub struct Rewriter {
    // license_buf: Vec<u8>,
    buf: Vec<u8>,
}
impl Rewriter {
    pub fn new() -> Self {
        Self {
            // license_buf: vec![0u8; LICENSE.len()],
            buf: Vec::with_capacity(3_000),
        }
    }

    fn starts_with(&self, bytes: &[u8]) -> bool {
        if bytes.len() > self.buf.len() {
            return false;
        }
        for (idx, byte) in bytes.into_iter().cloned().enumerate() {
            if self.buf[idx] != byte {
                return false;
            }
        }
        true
    }

    fn has_license_pref(&self) -> bool {
        self.starts_with(LICENSE_PREF.as_bytes())
    }
    fn has_current_license(&self) -> bool {
        self.starts_with(LICENSE)
    }

    fn has_license(&self) -> HasLicense {
        let mut res = HasLicense::No;

        if self.has_license_pref() {
            res = HasLicense::Old
        } else {
            return res;
        }

        if self.has_current_license() {
            res = HasLicense::Current
        } else {
            return res;
        }

        res
    }

    fn remove_license_pref(&self) -> Res<&[u8]> {
        debug_assert!(self.has_license_pref());

        let mut prev_is_star = false;
        let mut cursor = 0;
        let mut done = false;

        let mut bytes = self.buf.iter().cloned();

        while let Some(byte) = bytes.next() {
            if byte == b'*' {
                prev_is_star = true;
            } else if byte == b'/' && prev_is_star {
                done = true;
            } else {
                prev_is_star = false;
            }

            cursor += 1;

            if done {
                // Consume whitespaces between license text and actual content.
                'consume_ws: while let Some(byte) = bytes.next() {
                    if byte.is_ascii_whitespace() {
                        cursor += 1;
                    } else {
                        break 'consume_ws;
                    }
                }
                return Ok(&self.buf[cursor..]);
            }
        }

        bail!("could not find end of license comment `*/`")
    }

    pub fn work_on_file(&mut self, path: impl AsRef<Path>) -> Res<()> {
        use std::fs::OpenOptions;

        self.buf.clear();

        let path = path.as_ref();
        log::info!("| working on file `{}`", path.display());

        debug_assert!(self.buf.is_empty());
        let read = {
            use std::io::Read;
            OpenOptions::new()
                .read(true)
                .open(path)
                .chain_err(|| "while opening file in read mode")?
                .read_to_end(&mut self.buf)
                .chain_err(|| "while reading file")?
        };
        log::debug!("|     read {} bytes from file ({})", read, self.buf.len());

        let content: Option<&[u8]> = match self.has_license() {
            HasLicense::No => {
                log::debug!("|     no license header found");
                Some(&self.buf)
            }
            HasLicense::Old => {
                log::debug!("|     old license header found");
                Some(
                    self.remove_license_pref()
                        .chain_err(|| "while removing license header from file content")?,
                )
            }
            HasLicense::Current => {
                log::debug!("|     current license header found");
                None
            }
        };

        if let Some(content) = content {
            use std::io::Write;
            let mut file = OpenOptions::new()
                .write(true)
                .truncate(true)
                .open(path)
                .chain_err(|| "while opening file in write mode")?;
            log::debug!("|     writing new license header");
            file.write(LICENSE)
                .chain_err(|| "while writing license disclaimer")?;
            log::debug!("|     appending file content ({})", content.len());
            file.write(content)
                .chain_err(|| "while re-writing file content")?;
        } else {
            log::debug!("|     nothing to do");
        }

        Ok(())
    }
}
