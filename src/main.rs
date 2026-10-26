use axum::{Router};
use axum::routing::get;
use tracing::Level;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .init();

    let app = Router::new()
        .route("/", get(root));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:6000").await.unwrap();
    info!("Server is listening on port 6000");
    axum::serve(listener, app).await.unwrap();
}

async fn root() -> &'static str {
    "Root"
}
