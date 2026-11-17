# Contributing

## Checks

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cd tests/e2e && npm install && npx playwright install chromium && npx playwright test
```

Build without the BoringSSL toolchain: use the `plain` feature of `varex-http-relay`.

## Rules

- No placeholders or `todo!()`; list gaps in `docs/KNOWN_LIMITS.md` instead.
- Every bug fix gets a regression test.
- Check crate APIs against docs.rs for the exact pinned version.
- Cite the protocol spec section in comments for wire-format code.
- Do not copy code from Ultraviolet or Scramjet; check licenses before referencing.

## Testing notes

- `tests/e2e/tests/server.spec.ts`, `wisp.spec.ts`, `wisp-security.spec.ts` run today.
- `proxy-*.spec.ts` are gated by `VAREX_E2E_FULL=1` and need the Service Worker.
- The fixture origins are on loopback, which the SSRF guard blocks. Before enabling the
  gated tests, add a **test-only** config flag that allows loopback, and make sure it
  cannot be enabled in release builds.

## Site patches (planned)

Data-driven hostname-glob hooks so a site fix does not touch core code; each patch ships with a test.

## PR checklist

- [ ] fmt, clippy, tests pass
- [ ] regression test added
- [ ] docs updated if behavior changed
