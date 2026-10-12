//! `ym2612-bench [--seconds S] [--repeat N] [--mode on|quiet|both] <manifest.json>...`:
//! internal cycles per second and the fraction of real time, measured as the
//! reference's `benchmarks/bench.py` measures it.
//!
//! Each manifest's writes are driven through `run()` as a board drives the chip, for
//! its first `--seconds` of emulated time (or all of it, if shorter), in the
//! manifest's own variant, with the generator on and/or off (the quiet path).
//! `scripts/bench.sh` makes the workloads: the reference's stream (the seed-1 random
//! walk of 40,000 writes) and, where the exported board streams are present, two
//! games' own register traffic. Real time is 53,693,175 / 42 = 1,278,409 internal
//! cycles a second (NTSC). Reports the best of `--repeat` runs, since the machine is
//! shared, and a `total` line (the sum of the bests) that `scripts/ab.sh` reads.

use std::hint::black_box;
use std::process::ExitCode;
use std::time::Instant;

use ym2612_rust::chip::MCLK_PER_CYCLE;
use ym2612_rust::conformance::{load_manifest, variant, Kind, Manifest};
use ym2612_rust::YM2612;

const REAL_TIME: f64 = 53_693_175.0 / MCLK_PER_CYCLE as f64;

fn run(manifest: &Manifest, cycles: i64, enabled: bool) -> f64 {
    let mut chip = YM2612::new(variant(&manifest.variant).unwrap(), enabled);
    let started = Instant::now();
    for event in &manifest.events {
        if event.t >= cycles * MCLK_PER_CYCLE {
            break;
        }
        if let Kind::Write { port, data } = event.kind {
            chip.run(event.t);
            chip.write(port, data);
        }
    }
    chip.run(cycles * MCLK_PER_CYCLE);
    black_box(chip.take_samples());
    started.elapsed().as_secs_f64()
}

fn main() -> ExitCode {
    let mut args: Vec<String> = std::env::args().skip(1).collect();
    let mut take = |name: &str| -> Option<String> {
        let i = args.iter().position(|a| a == name)?;
        let value = args.get(i + 1).cloned();
        args.drain(i..(i + 2).min(args.len()));
        value
    };
    let seconds: f64 = take("--seconds").and_then(|v| v.parse().ok()).unwrap_or(2.0);
    let repeat: usize = take("--repeat").and_then(|v| v.parse().ok()).unwrap_or(10);
    let mode = take("--mode").unwrap_or_else(|| "both".into());
    if args.is_empty() || !["on", "quiet", "both"].contains(&mode.as_str()) {
        eprintln!("usage: ym2612-bench [--seconds S] [--repeat N] [--mode on|quiet|both] <manifest.json>...");
        return ExitCode::from(2);
    }
    let modes: Vec<(&str, bool)> = [("generator on", true), ("quiet path", false)]
        .into_iter()
        .filter(|(_, on)| mode == "both" || (mode == "on") == *on)
        .collect();
    let mut total = 0.0;
    for path in &args {
        let manifest = match load_manifest(path) {
            Ok(manifest) => manifest,
            Err(error) => {
                eprintln!("error: {error}");
                return ExitCode::from(2);
            }
        };
        let cycles = ((REAL_TIME * seconds) as i64).min(manifest.ticks / MCLK_PER_CYCLE);
        println!("{} ({}), {cycles} cycles, best of {repeat}", manifest.name, manifest.variant);
        for &(label, enabled) in &modes {
            let best = (0..repeat)
                .map(|_| run(&manifest, cycles, enabled))
                .fold(f64::INFINITY, f64::min);
            total += best;
            let rate = cycles as f64 / best;
            println!(
                "  {label:13} {rate:>14.0} cycles/s  {:8.2}x real time  ({best:.4} s)",
                rate / REAL_TIME
            );
        }
    }
    println!("total {total:.4} s");
    ExitCode::SUCCESS
}
