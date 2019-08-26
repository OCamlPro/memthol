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

use crate::base::*;

/// Initializes memthol's assets.
pub fn init(addr: &str, port: usize) -> Res<()> {
    content::setup(addr, port)
}

mod content {
    use std::io::Write;
    use std::{fs, path::PathBuf};

    use lazy_static::lazy_static;

    use crate::base::*;

    lazy_static! {
        /// Path to the asset directory.
        static ref ASSET_DIR: PathBuf = "static".into();
        /// Path to the css directory.
        static ref CSS_DIR: PathBuf = {
            let mut path = ASSET_DIR.clone();
            path.push("css");
            path
        };
        /// Path to the picture directory.
        static ref PICS_DIR: PathBuf = {
            let mut path = ASSET_DIR.clone();
            path.push("pics");
            path
        };
    }

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
    fn write_file(path: &PathBuf, content: &[u8]) -> Res<()> {
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

    /// Generates CSS-related files.
    mod top {
        use std::path::PathBuf;

        use super::*;

        lazy_static::lazy_static! {
            /// Main HTML file.
            static ref HTML_PATH: PathBuf = {
                let mut target = ASSET_DIR.clone();
                target.push("index.html");
                target
            };
            /// Favicon.
            static ref FAVICON_PATH: PathBuf = {
                let mut target = ASSET_DIR.clone();
                target.push("favicon.png");
                target
            };
            /// Memthol client's js script.
            static ref JS_PATH: PathBuf = {
                let mut target = ASSET_DIR.clone();
                target.push("client.js");
                target
            };
            /// Memthol client's wasm code.
            static ref WASM_PATH: PathBuf = {
                let mut target = ASSET_DIR.clone();
                target.push("client.wasm");
                target
            };
        }

        /// Content of the CSS files.
        mod content {
            lazy_static::lazy_static! {
                /// Main HTML file.
                pub static ref HTML: &'static [u8] = include_bytes!(
                    "../../static/index.html"
                );
                /// Favicon.
                pub static ref FAVICON: &'static [u8] = include_bytes!(
                    "../../static/favicon.png"
                );
                /// Memthol client's js script.
                pub static ref JS: &'static [u8] = include_bytes!(
                    "../../static/client.js"
                );
                /// Memthol client's wasm code.
                pub static ref WASM: &'static [u8] = include_bytes!(
                    "../../static/client.wasm"
                );
            }
        }

        lazy_static::lazy_static! {
            /// All top files.
            static ref ALL_TOP_FILES: [(&'static PathBuf, &'static [u8]) ; 4] = [
                (&*HTML_PATH, &*content::HTML),
                (&*FAVICON_PATH, &*content::FAVICON),
                (&*JS_PATH, &*content::JS),
                (&*WASM_PATH, &*content::WASM)
            ];
        }

        pub fn generate() -> Res<()> {
            for (path, content) in ALL_TOP_FILES.iter() {
                write_file(path, content)?
            }
            Ok(())
        }
    }

    /// Generates CSS-related files.
    mod css {
        use std::path::PathBuf;

        use super::*;

        lazy_static::lazy_static! {
            /// Main CSS file.
            static ref CSS_PATH: PathBuf = {
                let mut target = CSS_DIR.clone();
                target.push("client.css");
                target
            };
            /// CSS file map.
            static ref CSS_MAP_PATH: PathBuf = {
                let mut target = CSS_DIR.clone();
                target.push("client.css.map");
                target
            };
            /// SASS file.
            static ref SASS_PATH: PathBuf = {
                let mut target = CSS_DIR.clone();
                target.push("client.sass");
                target
            };
        }

        /// Content of the CSS files.
        mod content {
            lazy_static::lazy_static! {
                /// Main CSS file.
                pub static ref CSS: &'static [u8] = include_bytes!(
                    "../../static/css/client.css"
                );
                /// CSS file map.
                pub static ref CSS_MAP: &'static [u8] = include_bytes!(
                    "../../static/css/client.css.map"
                );
                /// SASS file.
                pub static ref SASS: &'static [u8] = include_bytes!(
                    "../../static/css/client.sass"
                );
            }
        }

        lazy_static::lazy_static! {
            /// All CSS-related files.
            static ref ALL_CSS_FILES: [(&'static PathBuf, &'static [u8]) ; 3] = [
                (&*CSS_PATH, &*content::CSS),
                (&*CSS_MAP_PATH, &*content::CSS_MAP),
                (&*SASS_PATH, &*content::SASS)
            ];
        }

        pub fn generate() -> Res<()> {
            for (path, content) in ALL_CSS_FILES.iter() {
                write_file(path, content)?
            }
            Ok(())
        }
    }

    /// Generates pictures.
    mod pics {
        use std::path::PathBuf;

        use super::*;

        lazy_static::lazy_static! {
            /// Header `h1` picture.
            static ref H1_PATH: PathBuf = {
                let mut target = PICS_DIR.clone();
                target.push("h1_background.png");
                target
            };
            /// Highlighting picture.
            static ref HILI_PATH: PathBuf = {
                let mut target = PICS_DIR.clone();
                target.push("hili_background.png");
                target
            };

            /// Arrow up picture.
            static ref ARROW_UP_PATH: PathBuf = {
                let mut target = PICS_DIR.clone();
                target.push("arrow_up.png");
                target
            };
            /// Arrow down picture.
            static ref ARROW_DOWN_PATH: PathBuf = {
                let mut target = PICS_DIR.clone();
                target.push("arrow_down.png");
                target
            };
            /// Arrow up picture.
            static ref EXPAND_PATH: PathBuf = {
                let mut target = PICS_DIR.clone();
                target.push("expand.png");
                target
            };
            /// Arrow up picture.
            static ref COLLAPSE_PATH: PathBuf = {
                let mut target = PICS_DIR.clone();
                target.push("collapse.png");
                target
            };
        }

        /// Content of the CSS files.
        mod content {
            lazy_static::lazy_static! {
                /// Header `h1` picture.
                pub static ref H1: &'static [u8] = include_bytes!(
                    "../../static/pics/h1_background.png"
                );
                /// Highlighting picture.
                pub static ref HILI: &'static [u8] = include_bytes!(
                    "../../static/pics/hili_background.png"
                );

                /// Arrow up.
                pub static ref ARROW_UP: &'static [u8] = include_bytes!(
                    "../../static/pics/arrow_up.png"
                );
                /// Arrow down.
                pub static ref ARROW_DOWN: &'static [u8] = include_bytes!(
                    "../../static/pics/arrow_down.png"
                );
                /// Expand.
                pub static ref EXPAND: &'static [u8] = include_bytes!(
                    "../../static/pics/expand.png"
                );
                /// Collapse.
                pub static ref COLLAPSE: &'static [u8] = include_bytes!(
                    "../../static/pics/collapse.png"
                );
            }
        }

        lazy_static::lazy_static! {
            /// All picture files.
            static ref ALL_PICS_FILES: [(&'static PathBuf, &'static [u8]) ; 6] = [
                (&*H1_PATH, &*content::H1),
                (&*HILI_PATH, &*content::HILI),
                (&*ARROW_UP_PATH, &*content::ARROW_UP),
                (&*ARROW_DOWN_PATH, &*content::ARROW_DOWN),
                (&*EXPAND_PATH, &*content::EXPAND),
                (&*COLLAPSE_PATH, &*content::COLLAPSE),
            ];
        }

        pub fn generate() -> Res<()> {
            for (path, content) in ALL_PICS_FILES.iter() {
                write_file(path, content)?
            }
            Ok(())
        }
    }
}
