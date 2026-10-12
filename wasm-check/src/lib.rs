//! The ladder's first two rungs inside WebAssembly, and a speed figure, for
//! `scripts/wasm-check.sh` to run under Node.
//!
//! Native results do not carry over by assumption: in wasm32 the ROM tables come from
//! Rust's own `libm` port, not the system's, and the core runs on another machine.
//! So the module exports the two tables for the host to hash (rung 1, against the
//! SHA-256 pinned in ym2612-python's `docs/contract.md`), runs every example manifest
//! against the reference's trace (rung 2), and runs a workload for a speed figure.

use ym2612_rust::chip::MCLK_PER_CYCLE;
use ym2612_rust::conformance::{manifest_from_value, trace_manifest, variant, Kind, Manifest};
use ym2612_rust::{exp_rom, log_sin_rom, YM2612};

/// (name, manifest JSON, reference trace) for every example in tests/data/conformance.
static CASES: &[(&str, &str, &str)] = include!(concat!(env!("OUT_DIR"), "/cases.rs"));

static mut TABLES: [u8; 1024] = [0; 1024];
static mut REPORT: String = String::new();

fn manifest(json: &str) -> Manifest {
    manifest_from_value(&serde_json::from_str(json).unwrap()).unwrap()
}

/// The two tables, `LOG_SIN` then `EXP`, each 256 little-endian u16: a pointer to
/// 1,024 bytes in this module's memory.
#[no_mangle]
pub extern "C" fn tables() -> *const u8 {
    let mut bytes = [0u8; 1024];
    for (i, v) in log_sin_rom().iter().chain(exp_rom().iter()).enumerate() {
        bytes[i * 2..i * 2 + 2].copy_from_slice(&v.to_le_bytes());
    }
    // SAFETY: wasm32 here is single-threaded, and nothing else touches TABLES.
    unsafe {
        TABLES = bytes;
        core::ptr::addr_of!(TABLES) as *const u8
    }
}

#[no_mangle]
pub extern "C" fn manifest_count() -> u32 {
    CASES.len() as u32
}

/// Run every example and compare its trace with the reference's, line for line.
/// Returns the number that differ; `report` describes them.
#[no_mangle]
pub extern "C" fn manifest_failures() -> u32 {
    let mut failures = 0;
    let mut report = String::new();
    for (name, json, reference) in CASES {
        let mut trace = Vec::new();
        trace_manifest(&manifest(json), &mut trace).unwrap();
        let ours = String::from_utf8(trace).unwrap();
        let first = reference
            .lines()
            .zip(ours.lines())
            .position(|(a, b)| a != b)
            .or((reference.lines().count() != ours.lines().count()).then_some(usize::MAX));
        if let Some(line) = first {
            failures += 1;
            report.push_str(&format!("{name}: differs at line {}\n", line.wrapping_add(1)));
        }
    }
    // SAFETY: as above.
    unsafe { REPORT = report };
    failures
}

/// The report of the last `manifest_failures`: (pointer << 32) | length.
#[no_mangle]
pub extern "C" fn report() -> u64 {
    // SAFETY: as above.
    let report = unsafe { &*core::ptr::addr_of!(REPORT) };
    ((report.as_ptr() as u64) << 32) | report.len() as u64
}

/// The speed workload: the `voices-ym2612` example (six sounding channels, every
/// algorithm, the LFO) driven through `run()` as a board drives the chip, then run
/// on to `cycles` internal cycles. Returns the number of samples made.
#[no_mangle]
pub extern "C" fn voices(cycles: u32) -> u32 {
    let (_, json, _) = CASES.iter().find(|(name, _, _)| *name == "voices-ym2612").unwrap();
    let manifest = manifest(json);
    let mut chip = YM2612::new(variant(&manifest.variant).unwrap(), true);
    let end = cycles as i64 * MCLK_PER_CYCLE;
    for event in &manifest.events {
        if event.t >= end {
            break;
        }
        if let Kind::Write { port, data } = event.kind {
            chip.run(event.t);
            chip.write(port, data);
        }
    }
    chip.run(end);
    chip.take_samples().len() as u32
}
