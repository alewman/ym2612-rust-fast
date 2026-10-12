//! Embed every example manifest and reference trace of tests/data/conformance.
use std::fmt::Write;
use std::path::Path;

fn main() {
    let data = Path::new(env!("CARGO_MANIFEST_DIR")).join("../tests/data/conformance");
    println!("cargo:rerun-if-changed={}", data.display());
    let mut manifests: Vec<_> = std::fs::read_dir(&data)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.extension().is_some_and(|e| e == "json"))
        .map(|path| path.canonicalize().unwrap())
        .collect();
    manifests.sort();
    let mut out = String::from("&[\n");
    for path in &manifests {
        let name = path.file_stem().unwrap().to_string_lossy();
        writeln!(
            out,
            "    ({name:?}, include_str!({:?}), include_str!({:?})),",
            path,
            path.with_extension("trace")
        )
        .unwrap();
    }
    out.push(']');
    let target = Path::new(&std::env::var("OUT_DIR").unwrap()).join("cases.rs");
    std::fs::write(target, out).unwrap();
}
