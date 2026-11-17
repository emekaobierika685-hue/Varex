# Protocols

## Wisp (implemented: v1-style framing)

Source: <https://github.com/MercuryWorkshop/wisp-protocol> (v1 `protocol.md`).

- Packet: `type:u8 | stream_id:u32 | payload`, all integers little-endian.
- `0x01 CONNECT`: `stream_type:u8 | port:u16 | hostname (rest, UTF-8)`; 1 = TCP, 2 = UDP.
- `0x02 DATA`: raw bytes.
- `0x03 CONTINUE`: `buffer_remaining:u32`. Server sends an initial CONTINUE on stream 0 after the socket opens.
- `0x04 CLOSE`: `reason:u8`.
- `0x05 INFO`: `version:u8 | extensions...` (parsed, otherwise ignored).

### Items that must be checked against the spec before release

1. **CLOSE reason codes.** The server currently uses `0x01` unspecified, `0x02` timeout,
   `0x03` refused, `0x04` network error. The spec defines its own table of reason codes;
   these values are provisional and may not match it. If they change, update
   `crates/varex-wisp/src/protocol.rs`, `VarexError::close_reason`, and `REFUSED` in
   `tests/e2e/tests/wisp-security.spec.ts`.
2. **CONTINUE semantics** (per-stream vs. global credit; replace vs. add). See ARCHITECTURE.md.
3. **Wisp v2** handshake and extension IDs. `crates/varex-wisp` does not implement v2.
   Earlier drafts of this project guessed the extension IDs; do not rely on them.
4. **Malformed packets** are currently dropped without a CLOSE (the connection stays up).

## HTTP relay (design)

- Request: `POST|GET|... /relay` with header `x-varex-target: <absolute URL>`.
- Browsers forbid scripts from setting `Cookie` and reading `Set-Cookie`, so cookies
  must travel in custom headers (planned: `x-varex-cookie`, `x-varex-set-cookie`).
  **Not implemented yet.**
- Redirects are returned to the client, not followed by the server.
- Security headers (CSP, X-Frame-Options, HSTS, COOP/COEP/CORP) are stripped from responses.

## bare-mux / TompHTTP Bare (planned)

The client `Transport` interface is meant to mirror bare-mux's `BareTransport` so
community transports can plug in. Verify the exact interface against the
published `@mercuryworkshop/bare-mux` version when implementing.

## URL encoding (planned)

`/proxy/<encoded>` where `<encoded>` comes from a pluggable codec: plain, base64url,
or AES-GCM. A persisted key is required for AES-GCM because Service Workers restart often.
