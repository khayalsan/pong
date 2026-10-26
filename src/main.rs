use tokio::net::TcpStream;
use std::time::Duration;
use axum::{Json, Router};
use axum::extract::Path;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::get;
use serde_json::json;
use tokio::io::AsyncWriteExt;
use tokio::time::timeout;
use tracing::{info, Level};

#[tokio::main]
async fn main() -> std::io::Result<()>{
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .init();

    let app = Router::new()
        .route("/{port}", get(check_port));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:6000").await?;
    info!("Server is listening on port 6000");
    axum::serve(listener, app).await?;

    Ok(())
}

async fn check_port(Path(port): Path<u16>) -> impl IntoResponse {
    let address = format!("127.0.0.1:{port}");
    let duration = Duration::from_secs(3);
    let connection = TcpStream::connect(address);
    let result = timeout(duration, connection).await;

    let mut reachable: bool = false;

    match result {
        Ok(Ok(mut stream)) => {
            info!("Connected to {port}");
            reachable = true;
            stream.shutdown().await.expect("Shutdown should succeed");
        }
        Ok(Err(error)) => {
            info!("Failed to connect to {port}: {error:?}");
        }
        Err(_) => {
            info!("Connection Timed out");
        }
    }

    (
        StatusCode::OK,
        Json(json!({
            "reachable": reachable,
        })),
    )
}
