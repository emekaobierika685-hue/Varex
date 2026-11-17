# Deployment

> The system is pre-release. Do not expose it publicly: there is no authentication.

## Docker

```bash
cp varex.example.toml varex.toml
docker compose up --build
```

The image is distroless, non-root, read-only rootfs. Mount your config at `/etc/varex/varex.toml`.

## From source

```bash
cargo run --release -p varex-wisp --bin varex-server -- --config varex.example.toml
```

## TLS and reverse proxy

Terminate TLS in Caddy or nginx. The Wisp WebSocket (`/wisp/`) is long-lived:
disable response buffering and use HTTP/1.1 for the upgrade. Validate the
Caddyfile (`caddy validate`) before use.

## Wildcard subdomains (planned)

Point `*.proxy.example.com` at the same host and issue a wildcard certificate
(requires DNS-01 validation). Each target origin maps to its own subdomain.

## Configuration

All keys are in `varex.example.toml`. Environment overrides use `VAREX__SECTION__KEY`;
currently only `server.bind`, `server.public_url`, `wisp.max_connections`,
`wisp.auth_password` are wired (check `Config::apply_env_overrides`; it may not be
called by `main.rs` in your tree).

## Monitoring

- `/healthz` liveness, `/readyz` returns 503 at the connection cap, `/metrics` Prometheus.
- Metrics: `varex_connections_active`, `varex_streams_active`, `varex_bytes_in_total`,
  `varex_bytes_out_total`, `varex_connect_seconds`, `varex_close_total`,
  `varex_rate_limited_total`, `varex_ssrf_blocked_total`.

## systemd (sketch)

```ini
[Service]
ExecStart=/usr/local/bin/varex-server --config /etc/varex/varex.toml
User=varex
NoNewPrivileges=true
ProtectSystem=strict
LimitNOFILE=65536
Restart=on-failure
```
