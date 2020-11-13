/*<LICENSE>
    This file is part of Memthol.

    Copyright (C) 2020 OCamlPro.

    Memthol is free software: you can redistribute it and/or modify
    it under the terms of the GNU General Public License as published by
    the Free Software Foundation, either version 3 of the License, or
    (at your option) any later version.

    Memthol is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
    GNU General Public License for more details.

    You should have received a copy of the GNU General Public License
    along with Memthol.  If not, see <https://www.gnu.org/licenses/>.
*/

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
