use std::net::SocketAddr;
use tokio::net::TcpStream;
use std::time::Duration;
use axum::{Json, Router};
use axum::extract::{ConnectInfo, Path};
use axum::handler::HandlerWithoutStateExt;
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
    axum::serve(listener, app.into_make_service_with_connect_info::<SocketAddr>()).await?;

    Ok(())
}

async fn check_port(ConnectInfo(client_addr): ConnectInfo<SocketAddr>, Path(port): Path<u16>) -> impl IntoResponse {
    let client_ip = client_addr.ip();
    let address = format!("{client_ip}:{port}");
    let duration = Duration::from_secs(3);
    let connection = TcpStream::connect(&address);
    let result = timeout(duration, connection).await;

    let mut reachable: bool = false;

    match result {
        Ok(Ok(mut stream)) => {
            info!("Connected to {address}");
            reachable = true;
            stream.shutdown().await.expect("Shutdown should succeed");
        }
        Ok(Err(error)) => {
            info!("Failed to connect to {address}: {error:?}");
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
