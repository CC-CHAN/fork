use axum::{
    handler::{get, post},
    AddExtensionLayer, Router,
};
use fork_backend::auth::{self, session::UserSession};
use fork_backend::init::init_appliations;
use std::net::SocketAddr;
use tracing::info;

#[tokio::main]
async fn main() {
    let app_connections = init_appliations();
    info!("build app router");
    // build our application with a route
    let app = Router::new()
        // `GET /` goes to `root`
        .route("/", get(root))
        // auth related control
        // `GET /` login with cookie and session
        .route("/login", post(auth::controller::login))
        .route("/logout", post(auth::controller::logout))
        .layer(AddExtensionLayer::new(app_connections));

    // run our app with hyper
    // `axum::Server` is a re-export of `hyper::Server`
    let addr = SocketAddr::from(([127, 0, 0, 1], 3030));
    info!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

// basic handler that responds with a static string
async fn root() -> &'static str {
    "Hello, World!"
}
