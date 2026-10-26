# Pong

A lightweight TCP port reachability checker built with **Rust** and **Axum**.

Pong provides a simple HTTP API for checking whether a TCP port is publicly reachable from the server's perspective. It was developed for [Scerite](https://github.com/khayalsan/scerite), a Minecraft server manager, but can be used independently.

A free, publicly hosted instance of Pong is available at:

```
https://pong.khayal.dev/{port}
```
Example: 
``` 
curl https://pong.khayal.dev/25565

{
  "reachable": true
}

```
Rate limit: 15 requests per minute per IP address.
## How It Works

When a client requests a port check, Pong attempts to establish a TCP connection to the client's public IP address on the specified port.

1. The client sends an HTTP GET request containing the port number.
2. Pong determines the client's IP address.
3. The request is checked against the per-IP rate limiter.
4. Pong attempts a TCP connection to the client's IP and requested port.
5. Pong returns whether the connection succeeded.

Pong only tests TCP connectivity. It does not scan arbitrary IP addresses, perform application-level protocol checks, or guarantee that the service behind an open port is functioning correctly.

## Installation

### Requirements

- Rust and Cargo
- A machine capable of accepting HTTP requests or running behind Cloudflare Tunnel

### Build from Source

Clone the repository:

```bash
git clone https://github.com/khayalsan/pong.git
cd pong
```

Build an optimized binary:

```bash
cargo build --release
```

The executable will be available at:

```text
target/release/pong
```

On Windows, the executable is `pong.exe`.

## Usage

Start Pong with the default configuration:

```bash
./target/release/pong
```

By default, Pong listens on port `6000` in direct mode, with a rate limit of 5 requests per minute per IP.

### CLI Arguments

| Argument | Short | Description | Default |
|---|---|---|---|
| `--port <PORT>` | `-p <PORT>` | HTTP listening port | `6000` |
| `--rate-limit <LIMIT>` | `-r <LIMIT>` | Rate limit per IP, in requests per minute | `5` |
| `--tunnel` | `-t` | Enable Cloudflare Tunnel mode | Disabled |
| `--help` | `-h` | Display available options | |

Examples:

```bash
# Run on port 8080
./target/release/pong --port 8080

# Allow 10 requests per minute per IP
./target/release/pong --rate-limit 10

# Run behind Cloudflare Tunnel
./target/release/pong --tunnel

# Combine options
./target/release/pong --port 6000 -r 10 -t
```

## API Reference

### Check TCP Port

`GET /{port}`

Checks whether the specified TCP port is reachable at the requesting client's public IP address.

**Parameters**

| Parameter | Type | Description |
|---|---|---|
| `port` | `u16` | TCP port to check (1–65535) |

**Example request**

```bash
curl https://pong.khayal.dev/25565
```

**Successful response - Port reachable**

HTTP `200 OK`

```json
{
  "reachable": true
}
```

**Successful response - Port unreachable**

HTTP `200 OK`

```json
{
  "reachable": false
}
```

A connection refusal, connection failure, or timeout results in `reachable: false`.

**Rate limit exceeded**

HTTP `429 Too Many Requests`

Pong uses per-IP rate limiting to restrict excessive requests.

### Response Codes

| Status | Meaning |
|---|---|
| `200 OK` | The connectivity check completed |
| `400 Bad Request` | Invalid or missing client IP information in tunnel mode |
| `429 Too Many Requests` | The client exceeded its rate limit |

Other HTTP error codes may be returned for invalid routes, unsupported methods, or server errors.

## Deployment

Pong supports two connection modes.

### Direct Mode

```bash
./target/release/pong --port 6000
```

Pong listens on `0.0.0.0:6000` and determines the client's IP address from the TCP connection.

Ensure that your firewall and hosting provider allow incoming connections to the configured port.

For public deployments, use HTTPS through a reverse proxy or another TLS termination layer.

### Cloudflare Tunnel Mode

```bash
./target/release/pong --tunnel --port 6000
```

Pong listens on `127.0.0.1:6000`, making the HTTP server accessible only through the local loopback interface.

Configure a Cloudflare Tunnel published application route:

| Setting | Value |
|---|---|
| Public hostname | `pong.example.com` |
| Service type | `HTTP` |
| Service URL | `127.0.0.1:6000` |

In tunnel mode, Pong reads the client's public IP from the `CF-Connecting-IP` header rather than the direct TCP peer address.

**Security:** Only enable tunnel mode when the application is accessible through a trusted Cloudflare Tunnel. Forwarded client-IP headers must not be trusted from arbitrary external clients.

### Running with systemd

For Linux servers, systemd can keep Pong running in the background, restart it after failures, and start it automatically on boot.

Create `/etc/systemd/system/pong.service`:

```ini
[Unit]
Description=Pong TCP Port Reachability Checker
After=network-online.target
Wants=network-online.target

[Service]
Type=simple
User=ubuntu
WorkingDirectory=/home/ubuntu/pong
ExecStart=/home/ubuntu/pong/target/release/pong --tunnel --port 6000 --rate-limit 5
Restart=on-failure
RestartSec=5

[Install]
WantedBy=multi-user.target
```

Adjust the executable path and user for your environment.

Enable and start the service:

```bash
sudo systemctl daemon-reload
sudo systemctl enable --now pong
```

Check its status:

```bash
sudo systemctl status pong
```

View logs:

```bash
sudo journalctl -u pong -f
```

After updating the source code:

```bash
git pull
cargo build --release
sudo systemctl restart pong
```

## Rate Limiting

Pong uses [Governor](https://github.com/boinkor-net/governor) to enforce rate limits independently for each client IP address.

The default limit is 5 requests per minute per IP.

```bash
./target/release/pong --rate-limit 10
```

Governor uses a replenishing rate-limit algorithm rather than a fixed one-minute window. Depending on the configured quota, requests may be permitted in bursts.

Clients sharing the same public IP address, such as users behind NAT, also share the same rate limit.

## Security Considerations

Pong is designed for a narrow purpose: checking TCP reachability at the requesting client's public IP address.

- **No arbitrary targets:** Clients cannot specify an IP address to scan.
- **Per-IP rate limiting:** Limits excessive requests.
- **Connection timeout:** TCP checks have a 3-second timeout.
- **Trusted proxy headers:** Cloudflare client-IP headers are only appropriate when requests originate from the trusted tunnel.
- **Restricted exposure:** Tunnel mode binds the HTTP server to localhost.

Pong does not authenticate users. Deployments should consider additional controls, such as connection concurrency limits and protections against requests targeting sensitive ports or addresses.

A successful TCP connection confirms that the port accepted a connection from Pong at the time of testing. It does not guarantee that other clients can connect or that the target application is healthy.

## Technology Stack

- [Rust](https://www.rust-lang.org/) - Programming language
- [Tokio](https://tokio.rs/) - Asynchronous runtime and TCP networking
- [Axum](https://github.com/tokio-rs/axum) - HTTP server framework
- [Governor](https://github.com/boinkor-net/governor) - Rate limiting
- [Clap](https://github.com/clap-rs/clap) - Command-line argument parsing
- [Tracing](https://github.com/tokio-rs/tracing) - Structured logging

## License
