# Threat Model

## Assets

Users' browsing traffic, operator's network position, other tenants' sessions.

## Threats and status

| Threat | Mitigation | Status |
|---|---|---|
| SSRF into the operator's network (loopback, RFC1918, link-local, cloud metadata) | Resolve, validate every IP, connect to the IP | Implemented on Wisp; **relay not pinned** |
| DNS rebinding | Connect to the validated address | Wisp: yes (single resolve). Relay: no |
| Open proxy abuse (spam, scanning, attacks) | Rate limits, port allowlist, host denylist | Partial. **No authentication** |
| Resource exhaustion (streams, connections, memory) | Per-connection and per-IP stream caps, queue bounds, idle timeouts | Implemented |
| Proxied page attacking the proxy UI (same origin) | Serve UI from a different origin; wildcard-subdomain mode | Planned |
| Cookie / storage leakage between proxied sites | Cookie jar + namespaced storage, or subdomain isolation | Planned |
| Malicious WASM plugin | Fuel, memory, time limits; no WASI by default | Planned |
| Client IP misattribution behind a proxy | Trusted-proxy `X-Forwarded-For` | Planned |

## Known SSRF gaps to close

IPv6 transition ranges (NAT64, 6to4, Teredo), unusual hostname encodings,
redirect re-validation on every hop for the relay.

## Reporting

Add a SECURITY.md with a private contact before any public release.
