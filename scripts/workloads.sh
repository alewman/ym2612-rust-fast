#!/usr/bin/env bash
# Make the benchmark workloads if missing and print their paths, one a line.
# The board streams are local (derived from ROMs): STREAMS, default
# ../ym2612-python/validation/streams; without them only the random walk is used.
set -euo pipefail
ROOT=$(cd "$(dirname "$0")/.." && pwd)
REF=${YM2612_PYTHON:-"$ROOT/external/ym2612-python"}
STREAMS=${STREAMS:-"$ROOT/../ym2612-python/validation/streams"}
OUT="$ROOT/external/manifests/bench"
if [ ! -f "$OUT/bench-random-walk-seed-1-40000-ym2612.json" ]; then
    "$REF/.venv/bin/python" "$ROOT/scripts/stream_manifests.py" --reference "$REF" --bench --out "$OUT" > /dev/null
fi
echo "$OUT/bench-random-walk-seed-1-40000-ym2612.json"
if [ -d "$STREAMS" ] && ls "$STREAMS"/*.bin > /dev/null 2>&1; then
    if [ ! -f "$OUT/gunstar-heroes-usa-ym2612.json" ]; then
        mkdir -p "$OUT/streams"
        cp "$STREAMS/Gunstar Heroes (USA)".* "$STREAMS/Sonic The Hedgehog (USA, Europe)".* "$OUT/streams/"
        "$REF/.venv/bin/python" "$ROOT/scripts/stream_manifests.py" --reference "$REF" --streams "$OUT/streams" --out "$OUT" > /dev/null
    fi
    echo "$OUT/gunstar-heroes-usa-ym2612.json"
    echo "$OUT/sonic-the-hedgehog-usa-europe-ym3438.json"
fi
