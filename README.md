# Varex

An experimental interception-based web proxy, written as a ground-up
alternative to [Ultraviolet](https://github.com/titaniumnetwork-dev/Ultraviolet)
and [Scramjet](https://github.com/MercuryWorkshop/scramjet).

> **Status: early development (v0.1.0).** The Rust Wisp server is written and
> being brought to a green build. The browser side (Service Worker, rewriter,
> runtime) is mostly scaffolding. Do not deploy this publicly yet.

## How it is meant to work

```
Page (patched by runtime)
  -> Service Worker (intercepts requests, rewrites responses)
    -> TransportManager (policy + failover)
       |- Wisp transport  --- WebSocket /wisp/ ---> varex-server -> upstream
       '- HTTP relay       --- POST /relay -------->      "
```

- **Wisp server (Rust):** relays TCP/UDP streams over one WebSocket, with
  flow control, SSRF protection, per-IP rate limiting and Prometheus metrics.
- **Rewriter (Rust/WASM):** streaming HTML/CSS and AST-based JS rewriting.
- **Service Worker + runtime (TypeScript):** intercepts requests and patches
  browser APIs so pages behave as if they run on the target origin.
- **HTTP relay (optional):** an alternate path with TLS fingerprint emulation.

## Status

| Component | State |
|---|---|
| Wisp v1.2 server (TCP/UDP, flow control) | Written, needs verification |
| SSRF guard, rate limiter, DNS cache, metrics | Written, needs verification |
| HTTP relay | Skeleton, not wired into the server |
| Streaming HTML/CSS rewriter | Not started (stub) |
| JS rewriter and `$varex` runtime | Not started |
| Service Worker, cookie jar, transports | Scaffolding only |
| Browser-side TLS over Wisp | Not started |

See `docs/KNOWN_LIMITS.md` for what will never work (DRM, WebAuthn,
bot-detection challenges).

## Quick start

### Docker

```bash
docker compose up --build
# open http://localhost:4000
```

### From source

Requires Rust (stable) and Node 20+.

```bash
cargo run --release -p varex-wisp --bin varex-server -- --config varex.example.toml
# open http://localhost:4000
```

The demo page opens a Wisp connection and logs the handshake. It cannot load
real sites until the browser-side transport is built.

## Checks

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

## Configuration

Copy `varex.example.toml` and edit it. Key settings:

```toml
[server]
bind = "0.0.0.0:4000"

[security]
allowed_ports = [80, 443, 8080, 8443]   # empty = allow all
denied_hosts  = ["*.internal", "*.local", "localhost"]
per_ip_rate_per_sec = 20.0
```

## Endpoints

| Path | Purpose |
|---|---|
| `/` | Demo page |
| `/wisp/` | Wisp WebSocket |
| `/healthz`, `/readyz` | Health checks |
| `/metrics` | Prometheus metrics |

## Responsible use

A public proxy attracts abuse. Keep the SSRF protections on, set rate limits,
and follow the laws where you operate.

## License

AGPL-3.0-only. Not affiliated with Ultraviolet, Scramjet or Titanium Network.
