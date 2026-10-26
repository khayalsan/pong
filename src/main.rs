mod rate_limit;
mod args;

use args::Args;
use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;
use tokio::net::TcpStream;
use std::time::Duration;
use axum::{Json, Router};
use axum::extract::{ConnectInfo, Path, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use serde_json::{json};
use tokio::io::AsyncWriteExt;
use tokio::time::timeout;
use tracing::{info, Level};
use clap::Parser;

use rate_limit::create_rate_limiter;
use crate::rate_limit::IpRateLimiter;

#[tokio::main]
async fn main() -> std::io::Result<()>{
    let args = Args::parse();

    let tunnel_enabled : bool = args.tunnel;
    let limit : u32 = args.rate_limit;
    let port : u16 = args.port;

    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .init();
    let state = AppState {
        limiter: Arc::new(create_rate_limiter(limit)),
        tunnel_enabled
    };
    let app = Router::new()
        .route("/{port}", get(check_port))
        .with_state(state);

    let bind_address = if tunnel_enabled {
        format!("127.0.0.1:{port}")
    }
    else {
        format!("0.0.0.0:{port}")
    };

    let listener = tokio::net::TcpListener::bind(bind_address).await?;
    let tunnel_log = if tunnel_enabled { "with Cloudflare tunnel" } else { "with direct connection" };
    info!("Server is listening on port {port} with rate limit:{limit} {tunnel_log}");
    axum::serve(listener, app.into_make_service_with_connect_info::<SocketAddr>()).await?;

    Ok(())
}

#[derive(Clone)]
struct AppState {
    limiter: Arc<IpRateLimiter>,
    tunnel_enabled: bool,
}

async fn check_port(
    State(state): State<AppState>,
    ConnectInfo(client_addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    Path(port):
    Path<u16>
) -> Response {
    let client_ip: IpAddr;

    if state.tunnel_enabled {
        client_ip = match headers
            .get("CF-Connecting-IP")
            .and_then(|value| value.to_str().ok())
            .and_then(|value| value.parse().ok()) {
            Some(ip) => ip,
            None => return StatusCode::BAD_REQUEST.into_response()
        }
    }
    else {
        client_ip = client_addr.ip();
    }

    if state.limiter.check_key(&client_ip).is_err() {
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
