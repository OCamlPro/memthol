//! Daemon monitoring files.

prelude! {}

use std::{
    ffi::OsString,
    path::{Path, PathBuf},
    thread::sleep,
    time::{Duration, SystemTime},
};

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
    new_diffs: Vec<AllocDiff>,

    /// Buffer for file-reading.
    buf: String,
}

impl Watcher {
    /// Spawns a watcher.
    pub fn spawn(dir: impl Into<String>) {
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
            if let Some(init) = self.try_read_init()? {
                let mut data =
                    super::get_mut().chain_err(|| "while registering the initial state")?;

                if data.init.is_some() {
                    bail!("live profiling restart is not supported yet")
                } else {
                    data.init = Some(init)
                }

                break 'first_init;
            } else {
                sleep(Duration::from_millis(200));
                continue 'first_init;
            }
        }

        // The call to `register_new_diffs` below can fail if the profiling run is restarted. If it
        // does, the error will be put here. Then, we try to read a new init file, and we drop the
        // error if that's successful. Otherwise, we return the error.
        let mut diff_error: Option<err::Err> = None;

        // Diff-reading loop.
        loop {
            if let Some(init) = self
                .try_read_init()
                .chain_err(|| "while checking whether the init file of the run has changed")?
            {
                // Discard any error that might have happened in previous calls to
                // `register_new_diffs` below.
                diff_error = None;
                self.reset_run(init)
                    .chain_err(|| "while resetting the run after init file was changed")?
            }

            // If `diff_error.is_some()`, then there was an error that was not cause by a restart of
            // the profiling run.
            if let Some(e) = std::mem::replace(&mut diff_error, None) {
                bail!(e)
            }

            let diff_res = self.register_new_diffs();

            match diff_res {
                Ok(true) => {
                    // Parsed something, keep going.
                    ()
                }
                Ok(false) => {
                    // Nothing new, sleep for a bit.
                    sleep(Duration::from_millis(100))
                }
                Err(e) => {
                    // There was a problem, remember it in `diff_error` and loop to check whether
                    // a restart happened.
                    diff_error = Some(e)
                }
            }
        }
    }
}

/// # Generic helpers.
impl Watcher {
    /// Constructor.
    fn new(dir: impl Into<String>) -> Self {
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
        debug_assert!(is_new);
        let is_new = self.known_files.insert((&self.init_file).into());
        debug_assert!(is_new)
    }

    /// Restarts the watcher and resets the data.
    ///
    /// Called when the init file of the run has changed.
    pub fn reset_run(&mut self, init: AllocInit) -> Res<()> {
        self.reset();
        let mut data = super::get_mut().chain_err(|| "while resetting the data")?;
        data.reset(init);
        Ok(())
    }

    /// Reads the content of a file and applies something to that content.
    ///
    /// - clears `self.buf` once it's done.
    /// - asserts `self.buf.is_empty()`.
    pub fn read_content<Out>(
        &mut self,
        path: impl AsRef<Path>,
        f: impl FnOnce(&str) -> Res<Out>,
    ) -> Res<Out> {
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
                debug_assert! { *lm <= last_modified }
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
                let init = AllocInit::try_from(content)?;
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
    /// - sleeps for `100` milliseconds if there are no new diffs;
    /// - asserts `self.new_diffs.is_empty()`.
    /// - returns `true` if something new was discovered.
    pub fn register_new_diffs(&mut self) -> Res<bool> {
        debug_assert!(self.new_diffs.is_empty());

        // I don't know why, but sometimes `gather_new_diffs` will miss diff files when the profiler
        // is running. Diffs following the missing diff file(s) will not make sense and will crash
        // diff registration.
        //
        // So, this first call retrieves the highest time of last modification of the diffs
        // gathered.
        let upper_bound = self.gather_new_diffs(None)?;

        // Now, `upper_bound.is_none()` iff no diff was found. In this case we do nothing.
        if upper_bound.is_some() {
            // If `upper_bound.is_some()`, we gather new diffs again but this time we give the upper
            // bound we got previously. This tells diff gathering to ignore everything more recent
            // than `upper_bound`. So, we will catch any intermediary diff we might have missed.
            self.gather_new_diffs(upper_bound)?;

            if !self.new_diffs.is_empty() {
                // Sort the diffs. This could be more efficient by having `gather_new_diffs` insert
                // in a sorted list.
                self.new_diffs
                    .sort_by(|diff_1, diff_2| diff_1.time.cmp(&diff_2.time));

                for diff in self.new_diffs.drain(0..) {
                    super::add_diff(diff)?
                }

                return Ok(true);
            }
        }

        Ok(false)
    }

    /// Gathers the new diff files.
    ///
    /// - diff files to send will be in `self.new_diffs`.
    /// - assumes `self.new_diffs.is_empty()`.
    /// - returns `true` if there was at list one new diff found (equivalent to
    ///     `!self.new_diffs.is_empty()`)
    pub fn gather_new_diffs(&mut self, upper_bound: Option<SystemTime>) -> Res<Option<SystemTime>> {
        use std::fs::read_dir;

        let mut highest_last_modified = None;

        // We need this to make sure we only work on file created **after** the init file.
        let init_last_modified = self
            .init_last_modified
            .clone()
            .ok_or("trying to gather diff file, but the init file has not been processed yet")?;

        debug_assert!(upper_bound.is_some() || self.new_diffs.is_empty());

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

            let file_path = file.path();

            let is_new = self.known_files.insert(file.file_name());

            if !is_new {
                continue;
            }

            // Was the file written after the init file?
            let last_modified = file_path
                .metadata()
                .chain_err(|| {
                    format!(
                        "could not retrieve metadata of file `{}`",
                        file_path.to_string_lossy()
                    )
                })?
                .modified()
                .chain_err(|| {
                    format!(
                        "could not retrieve time of last modification of init file`{}`",
                        file_path.to_string_lossy()
                    )
                })?;

            if last_modified < init_last_modified
                || upper_bound
                    .as_ref()
                    .map(|ubound| &last_modified > ubound)
                    .unwrap_or(false)
            {
                // Note that we remove the file from `known_files`in this case. This is because we
                // don't want to ignore this file in the future, as it might be overwritten and
                // become relevant.
                let was_there = self.known_files.remove(&file.file_name());
                debug_assert!(was_there);
                continue;
            }

            highest_last_modified = Some(if let Some(highest) = highest_last_modified {
                if highest < last_modified {
                    last_modified
                } else {
                    highest
                }
            } else {
                last_modified
            });

            let diff = self
                .read_content(&file_path, |content| {
                    let diff = AllocDiff::try_from(content)?;
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
        Ok(highest_last_modified)
    }
}
