# Architecture

> Status: only the server half (Wisp relay, SSRF guard, limits, metrics) exists as code.
> Everything browser-side is design. Sections marked *(planned)* are not implemented.

## Data flow

```
Page (patched by $varex runtime)                       (planned)
  -> Service Worker: intercepts /proxy/<encoded-url>   (planned)
     -> TransportManager: policy + failover            (planned)
        |- WispTransport   -- WebSocket /wisp/ --+
        '- HttpRelayTransport -- /relay ---------+--> varex-server -> upstream
```

## Components

| Component | Crate / package | Role |
|---|---|---|
| Core | `varex-core` | Config, errors, SSRF policy, rate limiter, DNS cache, metric names, plugin trait |
| Wisp server | `varex-wisp` | Wisp relay over WebSocket; binary `varex-server` |
| HTTP relay | `varex-http-relay` | Alternate path with TLS fingerprint emulation (optional) |
| Rewriter | `varex-rewriter` | WASM HTML/CSS/JS URL rewriting (*planned*, currently a URL resolver) |
| Plugin host | `varex-plugin-host` | In-process example plugin; wasmtime loader is *planned* |
| Client | `packages/varex-client` | Service Worker, transports, cookie jar (*planned*) |

## Wisp vs. HTTP relay

- **Wisp** carries raw TCP/UDP streams. TLS and HTTP must then run in the browser
  (for example Epoxy-TLS or libcurl.js compiled to WASM). The server never sees
  plaintext, so server-side TLS fingerprinting is irrelevant on this path.
- **HTTP relay** makes the request server-side. It is the only path where the
  server's TLS fingerprint matters, and the only path where the server sees
  plaintext. It is the fallback for origins that reject browser-side TLS stacks.

## Flow control (implemented)

Per-stream queue capacity equals `wisp.credit_window`. The WebSocket reader uses
`try_send`; a full queue means the client ignored its credit, so only that stream
is closed. After each successful upstream write the writer sends
`CONTINUE(stream_id, window - queue_len)`, keeping `client_credit + queue_len = window`.
Stream termination is reported on one shared channel per connection.

**Verify against the Wisp spec**: whether CONTINUE is per-stream or global, and
whether its value replaces or adds to the client's credit, was reasoned from the
spec text, not tested against a third-party client. See `PROTOCOLS.md`.

## Origin isolation (planned)

Single-origin mode mixes cookies/storage between proxied sites unless the runtime
namespaces them. Wildcard-subdomain mode (one subdomain per target origin) gives
real browser isolation and is the recommended production mode.

## Streaming

Server paths stream bodies (the HTTP relay still buffers request bodies; see
KNOWN_LIMITS). The rewriter is intended to run in a `TransformStream` in the
Service Worker so large documents are not buffered.
