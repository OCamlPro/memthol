//! Creates the router for memthol's server.

use gotham::router::Router;

/// Functions that load assets.
pub mod handlers {
    use gotham::{
        hyper::{
            header::{self, HeaderValue},
            Body, Response,
        },
        state::State,
    };

    /// Loads the index page.
    pub fn index_handler(state: State) -> (State, Response<Body>) {
        (state, Response::new(Body::from(crate::assets::INDEX)))
    }
    /// Loads the index page's favicon.
    pub fn favicon(state: State) -> (State, Response<Body>) {
        (state, Response::new(Body::from(crate::assets::FAVICON)))
    }

    /// Loads the wasm client, *i.e.* the actual client code.
    pub fn client_wasm(state: State) -> (State, Response<Body>) {
        let mut response = Response::new(Body::from(crate::assets::CLIENT_WASM));
        // Need to set the MIME-type to `application/wasm`.
        response.headers_mut().insert(
            header::CONTENT_TYPE,
            HeaderValue::from_static("application/wasm"),
        );
        (state, response)
    }
    /// Loads the JS part of the client.
    pub fn client_js(state: State) -> (State, Response<Body>) {
        let mut response = Response::new(Body::from(crate::assets::CLIENT_JS));
        // Need to set the MIME-type to `text/javascript`.
        response.headers_mut().insert(
            header::CONTENT_TYPE,
            HeaderValue::from_static("text/javascript"),
        );
        (state, response)
    }
}

/// Creates the router.
pub fn new() -> Router {
    use gotham::router::builder::{build_simple_router, DefineSingleRoute, DrawRoutes};
    build_simple_router(|route| {
        route.get("/").to(handlers::index_handler);

        route.get("index.html").to(handlers::index_handler);
        route.get("favicon.png").to(handlers::favicon);
        route.get("client_bg.wasm").to(handlers::client_wasm);
        route.get("client.js").to(handlers::client_js);
    })
}
