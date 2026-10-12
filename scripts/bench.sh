#!/usr/bin/env bash
# The benchmark workloads, made and run: the reference's stream (the seed-1 random
# walk of 40,000 writes, discrete chip) and, when the exported board streams are
# present (local, derived from ROMs), Gunstar Heroes on the discrete chip and Sonic
# the Hedgehog on the ASIC, each up to --seconds of emulated time (default 2).
# Usage: scripts/bench.sh [ym2612-bench options]
#   env: BENCH_BIN (another build's ym2612-bench; scripts/ab.sh sets it)
set -euo pipefail
ROOT=$(cd "$(dirname "$0")/.." && pwd)
"$ROOT/scripts/workloads.sh" > /dev/null
if [ -z "${BENCH_BIN:-}" ]; then
    cargo build --release -q --manifest-path "$ROOT/Cargo.toml" --bin ym2612-bench
    BENCH_BIN="$ROOT/target/release/ym2612-bench"
fi
mapfile -t WORKLOADS < <("$ROOT/scripts/workloads.sh")
exec "$BENCH_BIN" "$@" "${WORKLOADS[@]}"
