//! Creates the router for memthol's server.

use gotham::{
    handler::assets::FileOptions,
    router::{
        builder::{build_simple_router, DefineSingleRoute, DrawRoutes},
        Router,
    },
};

/// Creates the router.
pub fn new() -> Router {
    build_simple_router(|route| {
        route.get("/").to_file("static/index.html");
        route.get("/index.html").to_file("static/index.html");

        route.get("static/*").to_dir(
            FileOptions::new("static")
                .with_cache_control("no-cache")
                .with_gzip(true)
                .build(),
        );
        route.get("css/*").to_dir(
            FileOptions::new("static/css")
                .with_cache_control("no-cache")
                .with_gzip(true)
                .build(),
        );
        route.get("pics/*").to_dir(
            FileOptions::new("static/pics")
                .with_cache_control("no-cache")
                .with_gzip(true)
                .build(),
        );
        route.get("client_bg.wasm").to_file("static/client_bg.wasm");
        route.get("client.js").to_file("static/client.js");
    })
}
