# justfile
default: check

check: rust ts wasm

rust:
    cargo fmt --all -- --check
    cargo clippy --workspace --all-targets -- -D warnings
    cargo test --workspace

ts:
    cd packages/varex-client && npm ci && npm run build
    cd packages/varex-sdk   && npm ci && npm run build

wasm:
    ./scripts/build-wasm.sh

plain:
    cargo check --workspace --no-default-features --features plain

e2e:
    cd tests/e2e && npm ci && npx playwright test
