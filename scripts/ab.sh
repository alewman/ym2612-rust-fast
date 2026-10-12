#!/usr/bin/env bash
# A/B the benchmark: a base revision against the working tree, alternately, on one
# core, best of ROUNDS each, under each code layout.
#
# Why: the machine is shared, and separate runs move by tens of percent with other
# people's load. Two builds timed alternately on one core see the same load, and
# interference only ever slows a sample, so best-of recovers the undisturbed rate.
# The ratio is the result (m68000-rust-fast's and z80-rust-fast's method). Both sides
# run the working tree's benchmark program, so only the core differs.
#
# Usage: scripts/ab.sh [BASE (default HEAD)]
#   env: CPU (default 31), ROUNDS (default 7), LAYOUTS (default "default noalign nf5"),
#        MODE (on, the default, or quiet), SECONDS (default 2)
set -euo pipefail
ROOT=$(cd "$(dirname "$0")/.." && pwd)
BASE=${1:-HEAD}
CPU=${CPU:-31}
ROUNDS=${ROUNDS:-7}
LAYOUTS=${LAYOUTS:-"default noalign nf5"}
MODE=${MODE:-on}
SECONDS_=${SECONDS:-2}
declare -A FLAGS=(
    [default]=""
    [noalign]="-C opt-level=3"
    [nf5]="-C llvm-args=-align-all-nofallthru-blocks=5"
)
BASE_TREE="$ROOT/target/ab-base"
rev=$(git -C "$ROOT" rev-parse --short "$BASE")
if [ ! -d "$BASE_TREE" ]; then
    git -C "$ROOT" worktree add -q --detach "$BASE_TREE" "$rev"
fi
git -C "$BASE_TREE" checkout -q -f --detach "$rev"
cp "$ROOT/src/bin/ym2612-bench.rs" "$BASE_TREE/src/bin/"
build() { # tree layout -> binary path
    local tree=$1 layout=$2 target
    target="$tree/target/ab-$layout"
    RUSTFLAGS="${FLAGS[$layout]}" CARGO_TARGET_DIR="$target" \
        cargo build --release -q --manifest-path "$tree/Cargo.toml" --bin ym2612-bench >&2 || return 1
    echo "$target/release/ym2612-bench"
}
seconds() { sed -nE 's/^total ([0-9.]+) s$/\1/p'; }
"$ROOT/scripts/workloads.sh" > /dev/null
echo "$MODE: base $rev vs working tree, cpu $CPU, $ROUNDS rounds, load $(cut -d' ' -f1 /proc/loadavg)"
best_base_all=""; best_new_all=""
for layout in $LAYOUTS; do
    # A failed build must stop the run: timing a stale binary is worse than no number.
    a=$(build "$BASE_TREE" "$layout") || { echo "base build failed" >&2; exit 1; }
    b=$(build "$ROOT" "$layout") || { echo "working-tree build failed" >&2; exit 1; }
    best_a=""; best_b=""
    for _ in $(seq "$ROUNDS"); do
        for side in a b; do
            bin=$a; [ $side = b ] && bin=$b
            t=$(BENCH_BIN=$bin taskset -c "$CPU" "$ROOT/scripts/bench.sh" --repeat 1 --mode "$MODE" --seconds "$SECONDS_" | seconds)
            if [ $side = a ]; then
                [ -z "$best_a" ] || awk "BEGIN{exit !($t < $best_a)}" && best_a=$t
            else
                [ -z "$best_b" ] || awk "BEGIN{exit !($t < $best_b)}" && best_b=$t
            fi
        done
    done
    echo "$layout: base $best_a s, new $best_b s, x$(awk "BEGIN{printf \"%.3f\", $best_a/$best_b}")"
    [ -z "$best_base_all" ] || awk "BEGIN{exit !($best_a < $best_base_all)}" && best_base_all=$best_a
    [ -z "$best_new_all" ] || awk "BEGIN{exit !($best_b < $best_new_all)}" && best_new_all=$best_b
done
echo "min over layouts: base $best_base_all s, new $best_new_all s, x$(awk "BEGIN{printf \"%.3f\", $best_base_all/$best_new_all}")"
