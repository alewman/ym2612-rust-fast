# ym2612-rust-fast

A fast fork of [ym2612-rust](https://github.com/alewman/ym2612-rust), the Rust
transcription of [ym2612-python](https://github.com/alewman/ym2612-python), the
Yamaha YM2612 (OPN2) and YM3438. It stays provably equivalent to the reference on
every internal cycle, with the same ladder, every rung, on every commit that touches
`src/`. Each optimization is one commit with its measurement. A change that does not
pay is reverted and recorded as not kept.

The crate keeps the name `ym2612_rust` and ym2612-rust's API, so a host written for
ym2612-rust compiles unchanged. [docs/handoff-brief.md](docs/handoff-brief.md) is the
request and its reading, and [docs/build-record.md](docs/build-record.md) is what
happened.

## Speed

The measure is `scripts/bench.sh`: three workloads driven through `run()` as a board
drives the chip, the first two emulated seconds of each, with the generator on:
- the reference's benchmark stream, the seed-1 random walk of 40,000 writes on the
  discrete chip;
- where the exported board streams are present (local, since they are derived from
  ROMs), Gunstar Heroes' own register traffic on the discrete chip;
- the same for Sonic the Hedgehog on the ASIC.

Every ratio is from `scripts/ab.sh`. The base commit and the candidate are built
under three code layouts (64- and 32-byte branch-target alignment, and none) and run
alternately on one core of a shared i9-13900K, best of 5 or 7 each. The ratio is the
result.

| Commit | Change | Ratio (min over layouts) |
| --- | --- | ---: |
| `111e670` | the fork point: ym2612-rust's transcription, with the harness | -- |
| `a1bc6e4` | `slot_ahead(k)`, a compare and a subtraction, for the ten `(cycles + k) % 24` | x1.068 |
| `75c6dd2` | release profile: one codegen unit, fat LTO | x1.036 |
| | **at `75c6dd2` against `111e670`** | **x1.106** |

Measured and not kept: the cycle compiled once per slot (const generic, 24 copies),
x0.89, because the copies overflow the instruction cache. `#[inline(always)]` on every
stage gave x1.015 and the algorithm table folded into bitmasks x1.011, both noise.

At `75c6dd2`, best of 10, one pinned core, 2026-10-11:

| Workload | Generator on | Quiet path |
| --- | ---: | ---: |
| random walk, discrete | 20.0 M cycles/s, **15.7x real time** | 68.6x |
| Gunstar Heroes, discrete | 24.0 M cycles/s, **18.8x** | 82.4x |
| Sonic the Hedgehog, ASIC | 24.1 M cycles/s, **18.9x** | 76.8x |

The reference on CPython 3.14 runs the random walk at 0.104x real time.

**Where it stands.** No single stage dominates; an ablation (the build record) puts
the largest at 14% of the cycle. Every further cut measured was within the noise: the
trace contract pins all 17 stages on every internal cycle, so there is little left to
remove. That is m68000-rust-fast's conclusion too. A step change would need a
different kind of core, such as stretches without writes fast-forwarded in closed
form with every pin kept exact. At this speed the FM chip costs a Mega Drive board
about 6% of one desktop core, so nothing yet says that is needed.

## On WebAssembly (phones and browsers)

`scripts/wasm-check.sh` proves the core in WebAssembly rather than assuming the
native results carry over. In wasm32 the ROM tables come from Rust's own `libm`, not
the system's. The script builds `wasm-check/`, a 3.9 MB module with the thirteen
example manifests and reference traces built in, and runs it under Node. At
`75c6dd2`, 2026-10-11, Node 22.19 on the i9:

| Check | Result |
| --- | --- |
| Rung 1: both tables hashed in wasm against ym2612-python's pinned SHA-256 | both pinned |
| Rung 2: the 13 example manifests, every line against the reference trace | 13 of 13 match |
| Speed: `voices-ym2612` (six sounding channels) for 2 emulated seconds | 26.5 M cycles/s, **20.7x real time** (native: 29.6x, so wasm is 70%) |

The ladder's +1 removed drops rung 2 to 6 of 13, so the check bites. Rungs 3-5 run
natively only; the module runs the same source. A phone will be slower than this
desktop. At wasm's 70% of native, even a phone at a quarter of this machine's speed
would run the FM chip at about 5x real time. Measuring that on a real phone is the
next step, with the board.

## Certification

Pinned oracles:

| What | Version |
| --- | --- |
| ym2612-python (reference core and conformance kit) | `2c2ab235aed619b395ba152111cd79a6193390b3` |
| Manifest and trace formats | manifest version 1, `# ym2612-trace 1` |

The ladder is the reference's own (its `docs/conformance.md`, "Certification ladder
for a port"). Every rung is rerun by `scripts/ladder.sh` on each commit that touches `src/`, most
recently at `75c6dd2` on 2026-10-11 (Linux x86_64, rustc 1.93.1, the reference on
CPython 3.14.4); the results below are unchanged from ym2612-rust.

| Rung | What | Result | Script |
| --- | --- | --- | --- |
| 1 | The two ROM tables, packed as 256 little-endian u16, against the SHA-256 the reference's `docs/contract.md` pins | `log_sin` `a099b16d…`, `exp` `a0ca448c…`: both pinned | `rung1.sh` |
| 2 | Every example manifest in the reference's `examples/conformance/` | 13 of 13 identical | `rung2.sh` |
| 3 | The seven synthetic streams of the reference's `scripts/run_fm_streams.py` (two random walks of 4,000 writes, channel 3's special mode and CSM, SSG-EG's eight shapes, the LFO at 8 speeds × 8 PMS × 4 AMS, both timers, the DAC), both variants | 14 of 14 identical, 6,124,839 cycles each variant | `rung3.sh` |
| 4 | The register streams megadrive-python's board fed its chip: Nemesis's six test ROMs and five cartridges (Sonic the Hedgehog, Streets of Rage 2, Gunstar Heroes, Castlevania Bloodlines, Phantasy Star IV), both variants | 22 of 22 identical, 104,377,686 cycles each variant (3 min 47 s) | `rung4.sh` |
| 5 | 128 seeded random manifests: six sounding voices, then 116,335 port writes at any tick, 1,899 reads on every port, 91 /IC pulses at both lags with the voices reprogrammed after each, 14 on the quiet path, 58 with the $27 delay, 20 probes of the driven value's decay; both variants | 128 of 128 identical, 18.5 M cycles | `rung5.sh` |

Rungs 3 and 4 together are every cycle of the reference's Nuked-OPN2 record
(110,502,525 per variant), as a trace identical to the reference's here is the run
the reference compared with the die model. Each stream's manifest writes at
`cycle × 42` ticks, which is where the reference's `replay` writes.

**A gate that cannot fail proves nothing.** `scripts/seeded_faults.py` breaks the
core in twelve places, one at a time, in a copy of the repository: the ladder's +1,
BUSY's length, detune's sign, feedback, the slowest LFO reload, SSG-EG's rate, timer
B's period, /IC's pause carry, the DAC's sign bit, the $27 delay, key-on's operator
order, and the status decay. The thirteen examples catch nine. Rungs 3 and 5 catch
the other three:

- the LFO reload: 6 of 14 synthetic manifests differ, and 11 of the 128 random ones;
- key-on's operator order: 4 synthetic and 67 random;
- the status decay: only the random decay probes, 7 of them.

The reference records the decay gap in its own examples. The other two are gaps in
its examples that this port found (see "Notes for the reference").

CI (`.github/workflows/ci.yml`) runs `cargo fmt --check`, `cargo clippy -D
warnings`, `cargo test` and rung 1 on main and pull requests. `cargo test` replays
the thirteen example manifests against copies of the reference traces, without
Python. Rungs 2-5 need the reference checked out (`scripts/fetch_ym2612python.sh`),
and rung 4 the ROM-derived board streams, so they run locally.

### Reproducing

```text
scripts/fetch_ym2612python.sh     # the reference at the pin into external/, with a venv
scripts/rung1.sh
scripts/rung2.sh
scripts/rung3.sh                  # writes the manifests into external/manifests/synthetic
scripts/rung4.sh [streams dir]    # needs the reference's exported board streams, below
scripts/rung5.sh [count] [seed]   # 128 from seed 1 by default
python3 scripts/seeded_faults.py  # after rungs 3 and 5, which write the manifests it reuses
scripts/ladder.sh                 # rungs 1-5, stopping at the first that fails
scripts/bench.sh
scripts/ab.sh [BASE]              # base against the working tree, alternately
scripts/wasm-check.sh
```

Each rung writes this core's trace and pipes it to the reference's own
`python -m ym2612_python.conformance diff manifest.json -`, 16 at a time
(`scripts/diff_all.sh`). Rung 4's streams come from ROMs, so neither repository
commits them, nor the manifests made from them (`external/`, ignored). The reference
exports them with
`python scripts/export_board_streams.py --board ../megadrive-python --roms --games`.

## Using the core

```rust
use ym2612_rust::{YM2612, YM2612_DISCRETE};

let mut chip = YM2612::new(YM2612_DISCRETE, true);
let mut tick = 0i64;
let mut write = |chip: &mut YM2612, register: u8, value: u8| {
    for (port, byte) in [(0, register), (1, value)] {
        chip.run(tick); // bring the chip to the moment of the access
        chip.write(port, byte);
        tick += 24 * 42; // leave the chip a sample between writes
    }
};
for (register, value) in [(0x30, 0x01), (0x40, 0x00), (0x50, 0x1F), (0x80, 0x0F),
                          (0xB0, 0x07), (0xB4, 0xC0), (0xA4, 0x22), (0xA0, 0x69),
                          (0x28, 0x10)] {
    write(&mut chip, register, value);
}
chip.run(tick + 53_693_175 / 10); // a tenth of a second (ticks are the MD's MCLK)
let samples = chip.take_samples(); // (left, right) at 53,267 Hz
```

The contract is the reference's `docs/contract.md`, in Rust form:

- `run(t)` takes **ticks**, seven to an input clock, which on the Mega Drive is the
  master clock; it completes every internal cycle that ends by `t`.
- `write(port, value)` and `read(port)` are the two ports.
- `reset_line(held, Some(t), lag)` is /IC (`lag` is the board's:
  `chip::IC_SLOT_LAG` for the reset button, `chip::IC_WRITE_SLOT_LAG` for a write
  to $A11200), and `irq()` is /IRQ.
- `mol` and `mor` are the DAC pins after the last cycle, and `take_samples()`
  returns their sums, one pair per 24 cycles.
- `enabled` (the constructor's second argument) is the generator switch, and
  `control_delay` is register $27's timer delay.

The pipeline's state is public so a debugger can look at it. It is not a contract.

## Binaries

- `ym2612-trace <manifest> [--out FILE]`: run a conformance manifest and write its
  trace (rungs 2-5).
- `ym2612-tables log_sin|exp`: a table as rung 1 hashes it.
- `ym2612-bench [--seconds S] [--repeat N] [--mode on|quiet|both] [--past-end] <manifest>...`:
  the benchmark, through `run()`, with a `total` line for `scripts/ab.sh`.

## Notes for the reference

The seeded faults found two rules that none of ym2612-python's thirteen examples
reach: the slowest LFO speed's counter reload (108), and which of operators 2 and 3
a key-on bit reaches. Its synthetic streams catch both, so its Nuked-OPN2 record is
unaffected; only a port that stops at rung 2 would miss them. An example for each
would close the gap.

## Where the brief is

This fork's: [docs/handoff-brief.md](docs/handoff-brief.md) and
[docs/build-record.md](docs/build-record.md). ym2612-rust's, carried over:
[docs/ym2612-rust-handoff-brief.md](docs/ym2612-rust-handoff-brief.md) and
[docs/ym2612-rust-build-record.md](docs/ym2612-rust-build-record.md).

## License

MIT, like the reference. Nuked-OPN2 (LGPL-2.1) and YM2608-LLE (GPL-2.0) are cited
through the reference and never read into this repository.
