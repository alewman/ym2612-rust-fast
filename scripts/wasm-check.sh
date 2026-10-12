#!/usr/bin/env bash
# The phone/browser target checked on its own terms: build wasm-check for
# wasm32-unknown-unknown and run it under Node. Inside WebAssembly: rung 1 (the
# tables, from Rust's own libm, hashed), rung 2 (the 13 example manifests against the
# reference traces), and the voices workload for a speed figure; then the same
# workload natively, for the ratio.
# Needs the wasm32-unknown-unknown target and node.
#   env: TOOLCHAIN (rustup toolchain with the wasm32 target, default stable)
set -euo pipefail
ROOT=$(cd "$(dirname "$0")/.." && pwd)
TOOLCHAIN=${TOOLCHAIN:-stable}
RUSTC=$(rustup which --toolchain "$TOOLCHAIN" rustc) \
    rustup run "$TOOLCHAIN" cargo build --release -q \
    --manifest-path "$ROOT/wasm-check/Cargo.toml" --target wasm32-unknown-unknown
wasm="$ROOT/wasm-check/target/wasm32-unknown-unknown/release/ym2612_wasm_check.wasm"
echo "$(basename "$wasm"): $(stat -c %s "$wasm") bytes, node $(node --version)"
status=0
node "$ROOT/scripts/wasm-check.mjs" "$wasm" || status=$?
cargo build --release -q --manifest-path "$ROOT/Cargo.toml" --bin ym2612-bench
echo "native, the same workload: $("$ROOT/target/release/ym2612-bench" --past-end --mode on --repeat 7 \
    "$ROOT/tests/data/conformance/voices-ym2612.json" | sed -n 's/^ *generator on *//p')"
exit $status
