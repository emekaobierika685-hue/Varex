# Known Limits

## Not implemented yet

| Area | State |
|---|---|
| Browser-side TLS/HTTP over Wisp | Not started |
| Service Worker, runtime patches, cookie jar | Scaffolding / not started |
| Streaming HTML/CSS rewriter, JS rewriter | Not started |
| HTTP relay cookie forwarding, redirect handling | Not started |
| Relay DNS pinning (relay resolves DNS itself, so DNS-rebinding can bypass the SSRF check) | Not fixed |
| Relay request-body streaming (currently buffered up to the limit) | Not fixed |
| Wisp v2 | Not implemented |
| Authentication on Wisp/relay (a public instance is an open proxy) | Not implemented |
| `X-Forwarded-For` trust (behind a reverse proxy every client looks like one IP) | Not implemented |
| E2E against loopback fixtures (SSRF blocks loopback; needs a test-only flag) | Not implemented |

## Will not work reliably (inherent to this kind of proxy)

- **DRM / EME video.**
- **WebAuthn / hardware-bound authentication** (origin-bound by design).
- **Anti-bot challenges** that fingerprint the outer frame or detect proxies.
- **SharedArrayBuffer** (needs COOP/COEP headers, which are stripped).
- **`document.domain`**-dependent scripts.
- **Payment Request API**, full Clipboard API.
- **Target sites registering their own Service Workers** (blocked by design).

Ultraviolet and Scramjet have comparable limits. Keep a tested compatibility matrix
rather than promising "works on everything".
