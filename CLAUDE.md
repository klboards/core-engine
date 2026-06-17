# core-engine

The **deterministic zmanim/calendar core engine** for klboards — the anchor of the
correctness side of the product (org/0004). A git submodule of the `klboards/klboards`
meta-repo at `engine/core-engine`.

## Responsibility

- The three deterministic domain functions **F1 (solar) / F2 (lunar) / F3 (Hebrew-calendar)**
  emitting **absolute instants** (see `docs/adr/0001`).
- **Parameter resolution** — the engine exposes knobs and resolves none; it **owns the
  parameter-vector schema** (`docs/adr/0002`).
- **Refraction model** and **on-device horizon-profile composition** (`docs/adr/0004`, `0006`).
- Consumed by device, cloud, and apps as an embeddable library.

## Seam: correctness — inherited constraints (do not violate)

- **Fully offline** through a multi-day chag + Shabbat; the network is never on the
  correctness path (`docs/adr/0005`).
- **Own oracle-validated engine** behind a **pluggable engine interface**; **no external
  engine on the correctness path** (`docs/adr/0003`, `0008`). `zmanim-core` is a build/test
  oracle + optional customer-selectable alternative — **not** the foundation.
- **Oracle-validated** (Wolfram / observatory) as a test fixture, never a runtime dependency.
- Civil time / DST is an **edge axiom outside the core** (`docs/adr/0007`).

## Engine posture (ADR-0008)

We **build our own** primary engine (owned, oracle-validated, shipped), exposed behind a
**pluggable engine interface**. `zmanim-core` plays two distinct roles — a build/test
validation oracle (never ships) and an optional customer-selectable alternative engine behind
the interface. Relinkability (org/0006) is satisfied by this architecture, not a license claim.
**Engine-selection, once ≥2 engines ship, is a calendar-correctness-bearing knob** (handed to
the parameter-vector schema, `docs/adr/0002`) — the no-drift guarantee holds only within one
engine choice.

## Stack (DECIDED — ADR-0010)

**Rust**, **freestanding `no_std`** on-device (org/0006 runtime Profile A). One source → all targets
(native + WASM) with **vendored deterministic FP** (`libm` on every build); F1/F2 double-precision, F3
exact integer. `#![deny(unsafe_code)]` (the one C-ABI export in `ffi.rs` is the justified exception) +
`#![deny(missing_docs)]`. The default `std` feature exists ONLY so the integration-test harness and the
freestanding WASM artifact coexist; the engine source uses only `core` + `libm`. Custom target dir:
this checkout builds into `/home/brx/Benjwho/forge/target` (not `./target`).

## Current status (as of ADR-0020)

**F1 (solar) / F2 (lunar) / F3 (Hebrew calendar) + the four ADR-0001 couplings are COMPLETE and
validated; the CBOR intake boundary + the TerrainProfile moat are BUILT (/0018); the first-order
read-spec vocabulary is COMPLETE (/0020) — limb reference, fixed/seasonal minute-offset read, and
CBOR read-spec decode, so any posek vector composes with no remaining first-order gap.** Modules:
`geometry`/`optics`/`events` (F1; `terrain_horizon_crossing` = the azimuth-dependent terrain path;
`LimbReference` + `FixedMinuteOffset`), `lunar` (F2), `calendar` (F3 + molad + day-type), `tekufa`,
`kiddush_levana`, `couplings` (the only F1↔F3 point), `params` (knob catalog), `wire` (the
`no_std`/no-alloc minicbor reader: `ParameterVector` + `HorizonProfile` + `decode_read_spec`), `ffi`
(FP-determinism probe / relinkability boundary).

- **Validated:** F1 golden 66/66 (Wolfram), F2 11/11, F3 38/38 (Wolfram+Hebcal+MyZmanim), tekufa/
  tal-u-matar 10/10, couplings 3/3, properties 10/10, fuzz (incl. decoder, no panic), regression
  120/120, cross-engine 48/48 vs **KosherJava** (≈2.1 s), wire 7/7, read-vocab 4/4 (limb ordering +
  fixed/seasonal offset; /0020), terrain differential, offline-autonomy.
- **FP-determinism:** 659/659 exact native==wasm (the one-core-no-drift gate, /0010; +lower-limb netz
  + fixed/seasonal minute-offset float paths, /0020).
- **Intake (/0018, /0020):** `wire::decode_parameter_vector` / `decode_horizon_profile` /
  `decode_read_spec` (CBOR, CDE-deterministic, fixed-point integers — milliarcminutes / microdegrees /
  mm / milli-minutes; minicbor no-alloc). Param-vector retires `Optics::default()`-only + resolves
  `LimbReference`; horizon-profile drives `HorizonMode::TerrainProfile`; read-spec decodes one read (the
  `zman_definitions` catalog stays second-order, /0019). CDDL contracts in `docs/spec/*.cddl`.
- **NOT yet built:** **COSE_Sign1 verification** (↔ org/0006 §7 root-of-trust); the CBOR **writer**
  (provisioning-builder, Phase B); the `zman_definitions` read-spec **catalog** (second-order/management)
  + `solar.position_reference` (baked) + `obligation_sense`/rounding plumbing; `meeus-noaa` /
  `halachic-fixed-coefficient` refraction models.
- **Open gates / flags:** see `docs/adr/0016` §Open, `0017` §Open, `0018` §Open, `0020` §Open (molad meridian;
  bein-hashmashot default; realm geography = provisioned input; high-latitude fallback; /0003 tolerance;
  Rav-Ada anchor; `AbsoluteInstant` ~2262 CE horizon; COSE/root-of-trust; spec↔engine refraction/
  horizon_mode gaps). Israel high-res DTM (/0004) is the sole open top-level hard-TODO.

## Build & test

```
cargo test                                                   # full suite (oracle+property+fuzz+regression+cross-engine+offline)
cargo build --no-default-features --target wasm32-unknown-unknown   # freestanding no_std artifact
cargo run -q --example fp_probe_native | node tools/fp_probe.mjs <wasm>   # native==wasm determinism gate
cargo clippy --all-targets -- -D warnings && cargo fmt --check
BLESS_SNAPSHOT=1 cargo test --test regression                # regenerate the golden snapshot after an intended change
```
Cross-engine fixture regeneration (needs the LGPL jar, never vendored): see `tools/KosherDiff.java`.

## Conventions

Org-wide conventions (stack-agnostic rule, ADR cross-ref prefix, seam vocabulary,
decision+rationale → ADR) are inherited from the `klboards-org` plugin and the meta-repo
root `CLAUDE.md` — not duplicated here. Architecture decisions live in `docs/adr/`
(migrated from the meta-repo; org/0004).
