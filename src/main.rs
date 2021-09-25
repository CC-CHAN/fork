use axum::{
    handler::{get, post},
    http::StatusCode,
    response::IntoResponse,
    AddExtensionLayer, Json, Router,
};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

mod auth;
mod init;

#[tokio::main]
async fn main() {
    // initialize tracing
    tracing_subscriber::fmt::init();

    // build our application with a route
    let app = Router::new()
        // `GET /` goes to `root`
        .route("/", get(root))
        // auth related control
        // `GET /` login with cookie and session
        .route("/login", post(auth::controller::login))
        .route("/logout", get(auth::controller::logout))
        .layer(AddExtensionLayer::new(init::init_session_store()));

    // run our app with hyper
    // `axum::Server` is a re-export of `hyper::Server`
    let addr = SocketAddr::from(([127, 0, 0, 1], 3030));
    tracing::debug!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

// basic handler that responds with a static string
async fn root() -> &'static str {
    "Hello, World!"
}
