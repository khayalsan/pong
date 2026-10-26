mod rate_limit;

use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpStream;
use std::time::Duration;
use axum::{Json, Router};
use axum::extract::{ConnectInfo, Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use serde_json::json;
use tokio::io::AsyncWriteExt;
use tokio::time::timeout;
use tracing::{info, Level};

use rate_limit::create_rate_limiter;
use crate::rate_limit::IpRateLimiter;

#[tokio::main]
async fn main() -> std::io::Result<()>{
    let args: Vec<String> = std::env::args().collect();

    let limit : u32;

    limit = match args.get(1) {
        Some(value) => { value.parse::<u32>().unwrap_or(5) }
        None => { 5 }
    };

    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .init();

    let limiter = Arc::new(create_rate_limiter(limit));

    let app = Router::new()
        .route("/{port}", get(check_port))
        .with_state(limiter);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:6000").await?;
    info!("Server is listening on port 6000 with rate limit:{limit}");
    axum::serve(listener, app.into_make_service_with_connect_info::<SocketAddr>()).await?;

    Ok(())
}

type AppState = Arc<IpRateLimiter>;

async fn check_port(State(limiter): State<AppState>, ConnectInfo(client_addr): ConnectInfo<SocketAddr>, Path(port): Path<u16>) -> Response {
    let client_ip = client_addr.ip();

    if limiter.check_key(&client_ip).is_err() {
        info!("{client_ip}: Rate limit reached");
        return StatusCode::TOO_MANY_REQUESTS.into_response();
    }


    let address = format!("{client_ip}:{port}");
    let duration = Duration::from_secs(3);
    let connection = TcpStream::connect(&address);
    let result = timeout(duration, connection).await;

    let mut reachable: bool = false;

    match result {
        Ok(Ok(mut stream)) => {
            info!("{client_ip}: Connected to {address}");
            reachable = true;
            stream.shutdown().await.expect("Shutdown should succeed");
        }
        Ok(Err(error)) => {
            info!("{client_ip}: Failed to connect to {address}: {error:?}");
        }
        Err(_) => {
            info!("{client_ip}: Connection Timed out");
        }
    }

    (
        StatusCode::OK,
        Json(json!({
            "reachable": reachable,
        })),
    ).into_response()
}
