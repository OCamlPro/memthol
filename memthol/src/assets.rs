//! Handles the assets of the UI's client.
//!
//! The "assets" are stored in the `static` directory, and include
//!
//! - the html pages, typically `index.html`;
//! - the wasm/js code for the client (generated from `client`);
//! - the css and pictures used for styling.
//!
//! Basically, everything depends on whether we're compiling in `release` mode or not.
//!
//! # Release
//!
//! In `release`, all assets are embedded into the binary using Rust's `include_bytes` macro. When
//! the server starts, it will look for a `static` folder and delete it if it exists. Then it will
//! create a `static` folder and put all of the assets there.
//!
//! # Not Release
//!
//! When not in `release` mode, we pretty much assume we're running inside the build directory.
//! Hence, assets are not embedded and the binary will use the crate's directory's `static`
//! directory.

use std::path::PathBuf;

use crate::base::*;

/// Initializes memthol's assets.
pub fn init(addr: &str, port: usize) -> Res<()> {
    content::setup(addr, port)
}

/// String version of the path to the main asset directories.
macro_rules! asset_dir {
    (root) => {
        "static"
    };
    (css) => {
        concat!(asset_dir!(root), "/css")
    };
    (pics) => {
        concat!(asset_dir!(root), "/pics")
    };
}

/// String version of the path to a file in a main asset directory.
macro_rules! asset_file {
    ($key:tt / $path:expr) => {
        concat!(asset_dir!($key), "/", $path)
    };
}

macro_rules! asset_source {
    (build_dir $key:tt / $path:expr) => {
        concat!("", base::client_get_build_dir!(), "/", $path)
    };
    (lib $key:tt / $path:expr) => {
        concat!("../../libs/client/", asset_file!($key / $path))
    };
}

lazy_static::lazy_static! {
    /// Path to the asset directory.
    static ref ASSET_DIR: PathBuf = asset_dir!(root).into();
    /// Path to the css directory.
    static ref CSS_DIR: PathBuf = asset_dir!(css).into();
    /// Path to the picture directory.
    static ref PICS_DIR: PathBuf = asset_dir!(pics).into();
}

pub mod content {
    use std::{fs, io::Write};

    use super::*;

    /// Sets up the assets for the UI.
    pub fn setup(_addr: &str, _port: usize) -> Res<()> {
        mk_asset_dirs()?;
        css::generate()?;
        pics::generate()?;
        top::generate()?;
        Ok(())
    }

    /// Deletes the asset directory if one exists, and creates the asset directory.
    fn mk_asset_dirs() -> Res<()> {
        // Delete stuff if needed.
        rm_asset_dir()?;

        // Let's create stuff now.
        mk_asset_dir(&*ASSET_DIR)?;
        mk_asset_dir(&*CSS_DIR)?;
        mk_asset_dir(&*PICS_DIR)?;
        Ok(())
    }

    /// Deletes the asset directory if one exists.
    fn rm_asset_dir() -> Res<()> {
        let path = &*ASSET_DIR;
        match fs::read_dir(path) {
            Ok(_) => fs::remove_dir_all(path).chain_err(|| {
                format!(
                    "while removing asset directory `{}`",
                    path.to_string_lossy()
                )
            }),
            Err(e) => {
                if e.kind() == std::io::ErrorKind::NotFound {
                    Ok(())
                } else {
                    bail!(e)
                }
            }
        }
    }

    /// Creates an asset directory.
    fn mk_asset_dir(path: &PathBuf) -> Res<()> {
        fs::create_dir(path).chain_err(|| {
            format!(
                "while creating asset directory `{}`",
                path.to_string_lossy()
            )
        })?;
        Ok(())
    }

    /// Writes a file somewhere.
    fn write_writer<W: Write>(writer: &mut W, content: &[u8]) -> Res<()> {
        writer.write(content)?;
        Ok(())
    }

    /// Extracts the writer from a file.
    fn writer_of_file(path: &PathBuf) -> Res<std::fs::File> {
        use std::fs::OpenOptions;
        let writer = OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(path)
            .chain_err(|| format!("while generating asset file `{}`", path.to_string_lossy()))?;
        Ok(writer)
    }

    /// Writes a file somewhere.
    pub fn write_file(path: &PathBuf, content: &[u8]) -> Res<()> {
        let mut writer = writer_of_file(path)?;
        write_writer(&mut writer, content).chain_err(|| {
            format!(
                "while writing content for asset file `{}`",
                path.to_string_lossy()
            )
        })?;
        Ok(())
    }

    /// Builds a `generate` function that generates asset files.
    ///
    /// All asset file paths are relative from the client's crate directory.
    ///
    /// # Examples
    ///
    /// ```rust, ignore
    /// mod content {
    ///     make_generator_for! {
    ///         /// Main HTML file.
    ///         HTML: "static/path/to/index.html",
    ///         /// Main CSS file.
    ///         CSS: "static/path/to/style.css",
    ///     }
    /// }
    /// ```
    ///
    /// This only works if the client's static directory contains both files at the path passed
    /// above. The result is a `content::generate` function that dumps the content of the files from
    /// the client's static directory to `"static/path/to/<file>"`.
    macro_rules! make_generator_for {
        (
            $($(#[$doc:meta])*$id:ident : $path:tt / $name:expr, from $src_kind:tt),* $(,)*
        ) => {
            /// Paths to the assets.
            pub mod path {
                lazy_static::lazy_static! {$(
                    $(#[$doc])*
                    pub static ref $id: std::path::PathBuf = asset_file!($path / $name).into();
                )*}
            }

            // Actual assets.
            lazy_static::lazy_static! {$(
                $(#[$doc])*
                pub static ref $id: &'static [u8] = {
                    include_bytes!(
                        asset_source!($src_kind $path / $name)
                    )
                };
            )*}

            /// Dumps some static assets to where they belong.
            pub fn generate() -> crate::base::Res<()> {
                $(crate::assets::content::write_file(&*path::$id, &*$id)?;)*
                Ok(())
            }
        };
    }

    /// Generates CSS-related files.
    pub mod top {
        make_generator_for! {
            /// Main HTML file.
            HTML: root / "index.html", from lib,
            /// Favicon.
            FAVICON: root / "favicon.png", from lib,
            /// Memthol client's js script.
            MEMTHOL_JS: root / "client.js", from build_dir,
            /// Memthol client's wasm code.
            MEMTHOL_WASM: root / "client_bg.wasm", from build_dir,
        }
    }

    /// Generates CSS-related files.
    pub mod css {
        make_generator_for! {
            /// Main CSS file.
            MAIN_CSS: css / "style.css", from lib,
            /// Main CSS file map.
            MAIN_CSS_MAP: css / "style.css.map", from lib,
        }
    }

    /// Generates pictures.
    pub mod pics {
        make_generator_for! {
            /// Close.
            CLOSE: pics / "close.png", from lib,
            /// Add.
            ADD: pics / "add.png", from lib,
            /// Arrow up.
            ARROW_UP: pics / "arrow_up.png", from lib,
            /// Arrow down.
            ARROW_DOWN: pics / "arrow_down.png", from lib,
            /// Expand.
            EXPAND: pics / "expand.png", from lib,
            /// Collapse.
            COLLAPSE: pics / "collapse.png", from lib,

            /// Refresh.
            REFRESH: pics / "refresh.png", from lib,
            /// Refresh.
            SAVE: pics / "save.png", from lib,
            /// Undo.
            UNDO: pics / "undo.png", from lib,

            /// Tick, inactive.
            TICK_INACTIVE: pics / "tick_inactive.png", from lib,
            /// Tick, active.
            TICK_ACTIVE: pics / "tick_active.png", from lib,
        }
    }
}
