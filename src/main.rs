mod config;
mod model;
mod router;

use axum::Router;
use router::{healthcheck, root};

#[tokio::main]
async fn main() {
    // initialize tracing
    tracing_subscriber::fmt()
        // .with_max_level(tracing::Level::TRACE)
        .init();

    // build our application with a route
    let app = Router::new().merge(root()).merge(healthcheck());

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
