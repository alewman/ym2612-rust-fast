// Run wasm-check's exports under Node: rung 1 (the tables' SHA-256), rung 2 (the
// example manifests against the reference traces), and a speed figure.
import { readFileSync } from "fs";
import { createHash } from "crypto";
const { instance } = await WebAssembly.instantiate(readFileSync(process.argv[2]));
const e = instance.exports;
let ok = true;
const pinned = {
  log_sin: "a099b16dd3aa2f56cabc1fee590db31ee412ee361da4f0c67497092cf57c4776",
  exp: "a0ca448cb3178dd131cd23425b2adc8ee155d1905f2fa083d570954a47f3ede0",
};
const base = e.tables();
for (const [i, name] of ["log_sin", "exp"].entries()) {
  const bytes = new Uint8Array(e.memory.buffer, base + i * 512, 512);
  const got = createHash("sha256").update(bytes).digest("hex");
  const same = got === pinned[name];
  console.log(`rung 1 in wasm: ${name} ${got} ${same ? "(pinned)" : "DIFFERENT"}`);
  ok &&= same;
}
const failures = e.manifest_failures();
console.log(`rung 2 in wasm: ${e.manifest_count() - failures} of ${e.manifest_count()} examples match the reference traces`);
if (failures) {
  const packed = e.report();
  const bytes = new Uint8Array(e.memory.buffer, Number(packed >> 32n), Number(packed & 0xffffffffn));
  console.log(new TextDecoder().decode(bytes));
  ok = false;
}
const realTime = 53_693_175 / 42;
const cycles = Math.round(realTime * 2);
e.voices(100_000);
let best = Infinity;
for (let i = 0; i < 7; i++) {
  const t = performance.now();
  e.voices(cycles);
  best = Math.min(best, performance.now() - t);
}
const rate = cycles / (best / 1000);
console.log(`voices-ym2612 in wasm, 2 s emulated: ${(rate / 1e6).toFixed(2)} M cycles/s, ${(rate / realTime).toFixed(2)}x real time (best of 7)`);
process.exit(ok ? 0 : 1);
