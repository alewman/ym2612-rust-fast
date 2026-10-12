# Build record

2026-10-11, forked from ym2612-rust at `5a8db28`. The earlier repository's records
are `docs/ym2612-rust-handoff-brief.md` and `docs/ym2612-rust-build-record.md`.

## Measuring without a profiler

`perf` is locked on this machine (`perf_event_paranoid` 4), and there is no valgrind.
So the cost was found by ablation: in a scratch copy, each of the cycle's 17 stage
calls was removed in turn and the Gunstar Heroes workload timed. The outputs are
wrong, but the time saved is that stage's share:

| Stage | Share |
| --- | ---: |
| `envelope_adsr` | 13.8% |
| `phase_increment` | 11.0% |
| `envelope_prepare` | 7.7% |
| `fm_generate` | 7.0% |
| `ch_generate` | 6.8% |
| `fm_prepare` | 5.7% |
| `phase_generate` | 4.5% |
| `envelope_output` | 4.3% |
| `register_write` | 3.8% |
| `timer_b` | 3.3% |
| `envelope_ssg_eg` | 3.1% |
| `select_fnum` | 2.9% |
| `ch_output` | 2.5% |
| `do_io`, `timer_a`, `key_on`, `update_lfo` | noise |

The rest, about a quarter, is the cycle's own bookkeeping: the envelope timer, the
read shifts, the sample sums and the loop. No single stage dominates. objdump shows
the whole cycle inlined into `run()` as one function of about 1,500 instructions,
with `register_write` and `key_on` left out of line.

## What was tried

| Change | Result | Kept |
| --- | --- | --- |
| `slot_ahead(k)`, a compare and a subtraction, for the ten `(cycles + k) % 24` | x1.068 | yes, `a1bc6e4` |
| one codegen unit, fat LTO | x1.036 | yes, `75c6dd2` |
| the cycle compiled once per slot (const generic, 24 copies behind a jump on `cycles`) | x0.89 | no: 24 copies of the body overflow the instruction cache |
| `#[inline(always)]` on every stage | x1.015 | no: noise |
| `FM_ALGORITHM` folded into one byte per operator and algorithm | x1.011 | no: noise |

Cumulative, `75c6dd2` against the fork point, the same harness: **x1.106** (min over
layouts; x1.100-x1.106 each).

A trap avoided along the way: `channel` is not always `cycles % 6`. After an /IC
pause moves `cycles`, the reference leaves `channel` at its reset value until the
next cycle derives it again. Incrementing `channel` instead would have broken that,
so it stays derived.

A slip, also recorded: `ab.sh` first read its emulated duration from `$SECONDS`,
which bash defines as the seconds since the shell started. Every run emulated 0 s
until it was renamed `BENCH_SECONDS`.

## Where it stands

The transcription was already 14x real time with sound. Every remaining cut measured
here was within the noise. A cycle-exact pipeline runs all 17 stages each internal
cycle, and the trace contract pins every one of them, so there is little to remove.
That is m68000-rust-fast's conclusion too, at x1.38. A step change would need a
different kind of core: for example, fast-forwarding stretches with no writes in
closed form. That would have to keep every cycle's pins exact to stay on the ladder.
Nothing measured here says it is needed. At this speed the FM chip costs a Mega
Drive board about 6% of one desktop core, and about 5% of real time in wasm under
Node.

## WebAssembly

`scripts/wasm-check.sh` builds `wasm-check/`, a 3.9 MB module with the thirteen
example manifests and reference traces built in, and runs it under Node 22.19:

- rung 1: both tables, from Rust's own libm, hash to the pinned values;
- rung 2: 13 of 13 examples match;
- voices workload: 20.7x real time (native: 29.6x, so wasm is 70% of native).

A seeded fault, the ladder's +1 removed, drops rung 2 to 6 of 13, and the check exits
non-zero.
