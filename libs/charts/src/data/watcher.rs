//! Daemon monitoring files.

use std::{
    ffi::OsString,
    path::{Path, PathBuf},
    thread::sleep,
    time::{Duration, SystemTime},
};

use crate::base::*;

/// Daemon monitoring files.
pub struct Watcher {
    /// Directory to watch.
    dir: String,
    /// Temporary file used by memthol to write dumps.
    tmp_file: String,
    /// Init file.
    init_file: String,
    /// Last date of modification of the init file.
    ///
    /// This is used to detect new runs by checking whether the init file has been modified.
    init_last_modified: Option<SystemTime>,
    /// Files that have already been sent to the client and must be ignored.
    ///
    /// **Always** contains `self.tmp_file` and `self.init_file`.
    known_files: Set<OsString>,

    /// New diffs.
    new_diffs: Vec<Diff>,

    /// Buffer for file-reading.
    buf: String,
}

impl Watcher {
    /// Spawns a watcher.
    pub fn spawn<S>(dir: S)
    where
        S: Into<String>,
    {
        let mut watcher = Self::new(dir);

        let _ = std::thread::spawn(move || match watcher.run() {
            Ok(()) => (),
            Err(e) => super::add_err(e.pretty()),
        });
    }

    /// Runs the watcher.
    pub fn run(&mut self) -> Res<()> {
        // First init read.
        'first_init: loop {
            match self.try_read_init()? {
                Some(init) => {
                    let mut data =
                        super::get_mut().chain_err(|| "while registering the initial state")?;

                    if data.init.is_some() {
                        bail!("live profiling restart is not supported yet")
                    } else {
                        data.init = Some(init)
                    }

                    break 'first_init;
                }
                None => {
                    sleep(Duration::from_millis(200));
                    continue 'first_init;
                }
            }
        }

        // Diff-reading loop.
        loop {
            self.register_new_diffs()?
        }
    }
}

/// # Generic helpers.
impl Watcher {
    /// Constructor.
    fn new<S>(dir: S) -> Self
    where
        S: Into<String>,
    {
        let dir = dir.into();
        let tmp_file = "tmp.memthol".into();
        let init_file = "init.memthol".into();
        let init_last_modified = None;
        let known_files = Set::new();
        let new_diffs = vec![];
        let buf = String::new();
        let mut slf = Self {
            dir,
            tmp_file,
            init_file,
            init_last_modified,
            known_files,
            new_diffs,
            buf,
        };
        slf.reset();
        slf
    }

    /// Resets the watcher's state.
    ///
    /// - clears `self.known_files` and `self.new_diffs`;
    /// - adds `self.tmp_file` and `self.init_file` to `self.known_files`.
    pub fn reset(&mut self) {
        self.known_files.clear();
        self.new_diffs.clear();
        let is_new = self.known_files.insert((&self.tmp_file).into());
        debug_assert! { is_new }
        let is_new = self.known_files.insert((&self.init_file).into());
        debug_assert! { is_new }
    }

    /// Reads the content of a file and applies something to that content.
    ///
    /// - clears `self.buf` once it's done.
    /// - asserts `self.buf.is_empty()`.
    pub fn read_content<P, F, Out>(&mut self, path: P, f: F) -> Res<Out>
    where
        F: for<'a> FnOnce(&'a str) -> Res<Out>,
        P: AsRef<Path>,
    {
        use std::{fs::OpenOptions, io::Read};
        debug_assert!(self.buf.is_empty());
        let path = path.as_ref();
        let mut file_reader = OpenOptions::new().read(true).write(false).open(path)?;
        file_reader.read_to_string(&mut self.buf)?;
        let res = f(&self.buf);
        self.buf.clear();
        res
    }
}

/// # Init-file related functions.
impl Watcher {
    /// Reads the init file in the dump directory.
    ///
    /// Returns `None` when
    ///
    /// - init file does not exist, **or**
    /// - `init_last_modified = Some(t)` and the init file was modified more recently than `t`.
    ///
    /// This function is used *i)* during initialization to read the init file, and *ii)* to check
    /// whether it was overwritten by a new run to relaunch everything.
    ///
    /// If the result isn't `None`, `self.init_last_modified` is updated to the date of last
    /// modification of the init file.
    pub fn try_read_init(&mut self) -> Res<Option<AllocInit>> {
        let mut init_path = PathBuf::new();
        init_path.push(&self.dir);
        init_path.push(&self.init_file);

        if !(init_path.exists() && init_path.is_file()) {
            return Ok(None);
        }

        // Time of last modification of init file.
        let last_modified = init_path
            .metadata()
            .chain_err(|| {
                format!(
                    "could not retrieve metadata of init file `{}`",
                    init_path.to_string_lossy()
                )
            })?
            .modified()
            .chain_err(|| {
                format!(
                    "could not retrieve time of last modification of init file`{}`",
                    init_path.to_string_lossy()
                )
            })?;

        // Is it our first time loading the init file?
        if let Some(lm) = self.init_last_modified.as_mut() {
            // Not the first time, has the init file changed?
            if last_modified != *lm {
                // Yes, update
                debug_assert! { last_modified <= *lm }
                *lm = last_modified
            } else {
                // No, no need to load the file.
                return Ok(None);
            }
        } else {
            // First time, update time of last modification.
            self.init_last_modified = Some(last_modified)
        }

        self.read_content(init_path, |content| {
            if content.is_empty() {
                return Ok(None);
            } else {
                let init = AllocInit::from_str(content)?;
                Ok(Some(init))
            }
        })
        .chain_err(|| format!("while reading content of init file `{}`", self.init_file))
    }
}

/// # Diff-related functions.
impl Watcher {
    /// Gathers and registers new diffs.
    ///
    /// - sleeps for `200` milliseconds if there are no new diffs;
    /// - asserts `self.new_diffs.is_empty()`.
    pub fn register_new_diffs(&mut self) -> Res<()> {
        debug_assert!(self.new_diffs.is_empty());
        self.gather_new_diffs()?;

        if !self.new_diffs.is_empty() {
            // Sort the diffs. This could be more efficient by having `gather_new_diffs` insert in a
            // sorted list.
            self.new_diffs
                .sort_by(|diff_1, diff_2| diff_1.time.cmp(&diff_2.time));

            for diff in self.new_diffs.drain(0..) {
                super::add_diff(diff)?
            }
        } else {
            sleep(Duration::from_millis(200))
        }
        Ok(())
    }

    /// Gathers the new diff files.
    ///
    /// - diff files to send will be in `self.new_diffs`.
    /// - assumes `self.new_diffs.is_empty()`.
    /// - returns `true` if there was at list one new diff found (equivalent to
    ///     `!self.new_diffs.is_empty()`)
    pub fn gather_new_diffs(&mut self) -> Res<()> {
        use std::fs::read_dir;

        debug_assert!(self.new_diffs.is_empty());

        let dir = read_dir(&self.dir)
            .chain_err(|| format!("while reading dump directory `{}`", self.dir))?;

        for file in dir {
            let file = file.chain_err(|| format!("while reading dump directory `{}`", self.dir))?;
            let file_type = file.file_type().chain_err(|| {
                format!(
                    "failed to retrieve file/dir information for `{}`",
                    file.file_name().to_string_lossy()
                )
            })?;

            if !file_type.is_file() {
                continue;
            }

            let is_new = self.known_files.insert(file.file_name());

            // File is
            if is_new {
                let mut file_path = PathBuf::new();
                file_path.push(&self.dir);
                file_path.push(file.file_name());

                let diff = self
                    .read_content(&file_path, |content| {
                        let diff = Diff::from_str(content)?;
                        Ok(diff)
                    })
                    .chain_err(|| {
                        format!(
                            "while reading content of file `{}`",
                            file_path.to_string_lossy()
                        )
                    })?;
                self.new_diffs.push(diff)
            }
        }
        Ok(())
    }
}
