#!/usr/bin/env bash
# scripts/build-wasm.sh
# Builds the rewriter to wasm32 and generates the JS glue used by the SW.
set -euo pipefail

TARGET=wasm32-unknown-unknown
OUT=packages/varex-client/wasm
mkdir -p "$OUT"

rustup target add "$TARGET" 2>/dev/null || true
cargo build -p varex-rewriter --target "$TARGET" --release

if command -v wasm-bindgen >/dev/null 2>&1; then
  wasm-bindgen \
    "target/$TARGET/release/varex_rewriter.wasm" \
    --target web \
    --out-dir "$OUT"
else
  echo "wasm-bindgen not installed; install with: cargo install wasm-bindgen-cli"
  exit 1
fi

if command -v wasm-opt >/dev/null 2>&1; then
  wasm-opt -O3 "$OUT/varex_rewriter_bg.wasm" -o "$OUT/varex_rewriter_bg.wasm"
else
  echo "wasm-opt not installed; skipping size pass"
fi

echo "built $OUT/"
