# The request and the plan

## The request

2026-10-11, after ym2612-rust was pushed: "create the private repo and push, then
ym2612-rust-fast".

ym2612-python's brief (`/data/emu/ym2612-python-HANDOFF.md`, "Later phases"):
"`ym2612-rust`, transcribed and proven against the kit; `ym2612-rust-fast`; a
WebAssembly check of the kind m68000-rust-fast has (`scripts/wasm-check.sh`). This is
where the phone question gets its answer." The owner's purpose: to play
megadrive-python's emulator in its faster forms while testing the cartridges being
built, the Python FM chip being far too slow (0.104x real time).

## The plan, read from the family's shape

The shape is m68000-rust-fast's and z80-rust-fast's: a fork of the transcription
that keeps its crate name and API, so a host written for ym2612-rust compiles
unchanged, and stays on the whole ladder on every commit that touches `src/`.

1. Harness first: `ym2612-bench` over three workloads (the reference's random walk,
   and, locally, two games' register traffic, one per variant); `scripts/ab.sh`,
   base against working tree alternately on one core under three code layouts, best
   of N, the ratio the result; `scripts/ladder.sh`, rungs 1-5, failing fast.
2. Each optimization one commit with its measurement; what does not pay is
   reverted and recorded as not kept.
3. `scripts/wasm-check.sh`: rungs 1 and 2 inside WebAssembly under Node, the tables
   hashed there (they come from Rust's own libm in wasm32), a seeded fault to show
   it bites, and a speed figure against native.
4. Local only until the owner asks (m68000-rust-fast stayed local by choice).
