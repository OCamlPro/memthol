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
    pub fn setup(addr: &str, port: usize) -> Res<()> {
        mk_asset_dirs()?;
        css::generate()?;
        pics::generate()?;
        top::generate()?;
        more_js::generate(addr, port)?;
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

    /// Generates helper JS scripts.
    mod more_js {
        use super::*;

        lazy_static::lazy_static! {
            /// Script which returns the address of the server.
            static ref JS_SERVER_PATH: PathBuf = {
                let mut target = ASSET_DIR.clone();
                target.push("serverAddr.js");
                target
            };
        }

        /// Generates the helper to resolve the address
        fn js_server<W: Write>(writer: &mut W, addr: &str, port: usize) -> Res<()> {
            write!(
                writer,
                r#"
var serverAddr = {{
    get_addr: function() {{
        return `{}`;
    }},
    get_port: function() {{
        return {};
    }}
}}
            "#,
                addr, port
            )
            .chain_err(|| "while writing `js_server` file for SSE")?;
            Ok(())
        }

        pub fn generate(addr: &str, port: usize) -> Res<()> {
            let mut writer = writer_of_file(&*JS_SERVER_PATH)?;
            js_server(&mut writer, addr, port)?;
            Ok(())
        }
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
            $($(#[$doc:meta])*$id:ident : $path:expr),* $(,)*
        ) => {
            /// Paths to the assets.
            pub mod path {
                lazy_static::lazy_static! {$(
                    $(#[$doc])*
                    pub static ref $id: std::path::PathBuf = $path.into();
                )*}
            }

            // Actual assets.
            lazy_static::lazy_static! {$(
                $(#[$doc])*
                pub static ref $id: &'static [u8] = {
                    include_bytes!(
                        concat!(
                            "../../",
                            $path,
                        )
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
            HTML: asset_file!(root / "index.html"),
            /// Favicon.
            FAVICON: asset_file!(root / "favicon.png"),
            /// Memthol client's js script.
            MEMTHOL_JS: asset_file!(root / "client.js"),
            /// Memthol client's wasm code.
            MEMTHOL_WASM: asset_file!(root / "client.wasm"),
        }
    }

    /// Generates CSS-related files.
    pub mod css {
        make_generator_for! {
            /// Main CSS file.
            MAIN_CSS: asset_file!(css / "style.css"),
            /// Main CSS file map.
            MAIN_CSS_MAP: asset_file!(css / "style.css.map"),
        }
    }

    /// Generates pictures.
    pub mod pics {
        make_generator_for! {
            /// Close.
            CLOSE: asset_file!(pics / "close.png"),
            /// Add.
            ADD: asset_file!(pics / "add.png"),
            /// Arrow up.
            ARROW_UP: asset_file!(pics / "arrow_up.png"),
            /// Arrow down.
            ARROW_DOWN: asset_file!(pics / "arrow_down.png"),
            /// Expand.
            EXPAND: asset_file!(pics / "expand.png"),
            /// Collapse.
            COLLAPSE: asset_file!(pics / "collapse.png"),

            /// Refresh.
            REFRESH: asset_file!(pics / "refresh.png"),

            /// Tick, inactive.
            TICK_INACTIVE: asset_file!(pics / "tick_inactive.png"),
            /// Tick, active.
            TICK_ACTIVE: asset_file!(pics / "tick_active.png"),
        }
    }
}
