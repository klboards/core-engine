// Wasm side of the FP-determinism check (ADR core-domain/0010).
//
// Usage:  cargo run -q --example fp_probe_native | node tools/fp_probe.mjs <core_engine.wasm>
//
// Reads the native harness's lines from stdin (each carrying the inputs as exact f64
// bit-patterns + the native i64-nanosecond result), reconstructs the IDENTICAL f64 inputs,
// runs the SAME read through the freestanding wasm build, and asserts EXACT i64 equality.
// Exact divergence = platform-libm leakage the vendored-math rule exists to prevent → fail.

import { readFileSync } from "node:fs";

const wasmPath = process.argv[2];
if (!wasmPath) {
  console.error("usage: node fp_probe.mjs <wasm>");
  process.exit(2);
}

const f64FromHex = (hex) => {
  const dv = new DataView(new ArrayBuffer(8));
  dv.setBigUint64(0, BigInt("0x" + hex), false);
  return dv.getFloat64(0, false);
};

const bytes = readFileSync(wasmPath);
const { instance } = await WebAssembly.instantiate(bytes, {});
const probe = instance.exports.probe_zman_nanos;

const input = readFileSync(0, "utf8").trim();
const lines = input.length ? input.split("\n") : [];

let pass = 0;
let fail = 0;
const rows = [];
for (const line of lines) {
  const [kindS, latH, lonH, elevH, refH, angH, nativeS, label] = line.split(",");
  const kind = Number(kindS);
  const lat = f64FromHex(latH);
  const lon = f64FromHex(lonH);
  const elev = f64FromHex(elevH);
  const ref = f64FromHex(refH);
  const ang = f64FromHex(angH);
  const native = BigInt(nativeS);
  const wasm = BigInt(probe(kind, lat, lon, elev, ref, ang)); // i64 -> BigInt
  const eq = native === wasm;
  if (eq) pass++;
  else fail++;
  rows.push({ label, native: native.toString(), wasm: wasm.toString(), eq });
}

const W = Math.max(5, ...rows.map((r) => r.label.length));
console.log(`${"label".padEnd(W)}  exact?  native_nanos == wasm_nanos`);
for (const r of rows) {
  const mark = r.eq ? "OK " : "!! ";
  const detail = r.eq ? r.native : `${r.native}  !=  ${r.wasm}`;
  console.log(`${r.label.padEnd(W)}  ${mark}    ${detail}`);
}
console.log(`\nFP-determinism: ${pass}/${pass + fail} exact cross-target match (native vs wasm)`);
process.exit(fail === 0 ? 0 : 1);
