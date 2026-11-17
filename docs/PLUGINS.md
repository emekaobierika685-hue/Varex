# Plugins

## In-process (Rust, implemented)

Implement `varex_core::plugin::Plugin` (`on_connect`, `on_request`, `on_response`, `on_close`).
`BlocklistPlugin` in `crates/varex-plugin-host/src/example_blocklist.rs` is a template.
Note: the server does not call plugins yet; hooks still need wiring into `varex-wisp`.

## WASM (planned)

Planned ABI: a module exports `memory` and `varex_on_connect(ptr: i32, len: i32) -> i32`
(0 = allow). The host writes a JSON connect context into guest memory.
Requirements before enabling: fuel limit, memory limit, wall-clock timeout, no WASI.

## Client plugins (planned)

`@varex/sdk` defines `Plugin` and `PluginBus` types (`request`, `response`,
`rewrite:*`, `cookie`, `navigate`, `transport:select` events); the Service Worker
does not call them yet.
