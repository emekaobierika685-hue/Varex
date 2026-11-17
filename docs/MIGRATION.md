# Migration

> Varex is not a drop-in replacement yet. This document records the intended mapping.

## From Ultraviolet

| Ultraviolet | Varex (planned) |
|---|---|
| `uv.config.js` (`prefix`, `encodeUrl`, `decodeUrl`) | Typed config: prefix + codec |
| XOR codec | Available for compatibility only; prefer AES-GCM |
| Bare server (TompHTTP) | Wisp or HTTP relay; Bare endpoint is a possible compat layer |
| bare-mux transports | Same interface goal |

## From Scramjet

| Scramjet | Varex (planned) |
|---|---|
| Controller / frame API | Similar controller API |
| Wisp server | Wisp relay in `varex-wisp` (check protocol version compatibility) |
| WASM rewriter | `varex-rewriter` (different internals) |

## Steps (when the client lands)

1. Deploy `varex-server`; point your existing Wisp URL at `/wisp/`.
2. Swap the Service Worker and runtime files.
3. Re-test your target sites against the compatibility matrix.
