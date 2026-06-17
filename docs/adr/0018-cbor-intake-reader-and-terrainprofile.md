# ADR core-domain/0018 — CBOR intake reader (parameter vector + horizon profile) + TerrainProfile

- **Status:** Accepted
- **Date:** 2026-06-17
- **Scope:** Builds the **reader side** of the ADR-0011 serialization seam — the `no_std` CBOR decoder
  for the **parameter vector** and the **horizon profile** — and **activates `HorizonMode::TerrainProfile`**
  (the visible-sunrise moat). New `src/wire.rs`; `geometry::solar_azimuth_deg`;
  `events::terrain_horizon_crossing`; `minicbor` dependency; two CDDL files in `docs/spec/`. Closes two
  of ADR-0011's four open sub-items. The **writer** is provisioning-builder (Phase B), out of scope.

## Context

The engine was complete (≤0017) but took only a Rust API + `Optics::default()` — it could not ingest a
parameter vector or a horizon profile, and `TerrainProfile` silently fell back to the scalar elevation
dip. ADR-0011 decided the wire format (CBOR/CDDL/COSE) but left it unbuilt and four sub-items open. This
ADR builds the reader and lights up the terrain path, making the engine a **consumable component**.

## Decision

1. **Reader = `minicbor`** (no_std, **no-alloc** structured decode — the only mainstream CBOR codec that
   decodes typed values with no heap; Blue Oak license). First non-`libm` runtime dep; the freestanding
   wasm build links it (verified). Test-only `alloc` feature (dev-dep) lets the round-trip tests *encode*.
2. **Deterministic-CBOR profile = CDE / RFC 8949 §4.2.1 core-deterministic** (definite-length, preferred
   serialization, lexicographic map-key order). **NOT dCBOR** — its float→int numeric reduction is
   unwanted for a signed artifact and is draft-only. *(Closes ADR-0011 sub-item 1.)*
3. **Wire numerics = fixed-point integers → float-free wire** *(closes sub-item 2)*: horizon angles
   **milliarcminutes `i32`** (1/60000°, ~0.06″), lat/lon **microdegrees `i32`**, elev **mm `i32`**,
   refraction coefficient **micro-arcminutes**. Determinism reduces to integer preferred encoding + map
   ordering; the decoder converts to `f64` on read. (f16/f32 remain the fallback under size pressure.)
4. **Two decoded types** (`src/wire.rs`), integer-keyed CBOR maps: **`ParameterVector`** (§1 knobs;
   `resolve_optics` + calendar-knob accessors — **retires `Optics::default()` as the only input path**)
   and **`HorizonProfile<'_>`** (binding metadata + a borrowed, zero-copy packed `i32`-LE
   milliarcminute angle byte string; `horizon_angle_deg_at` interpolation + 360° wrap; `check_binding`
   enforcing the 0004/0006 φ/λ/h invariant). All failures are a typed **`DecodeError`** — never a panic
   (the /0017 invariant, extended to the reader and fuzzed).
5. **TerrainProfile activated** (the moat): `geometry::solar_azimuth_deg` (compass; reuses the altitude
   ephemeris) + `events::terrain_horizon_crossing` — an **azimuth-dependent** target
   `centre = horizon_angle(azimuth(t)) − semidiameter`, with its own scan/bisect reusing the /0017
   **geometric-slope gate**. Self-contained: the scalar `find_crossing` and Mishor/Visible stay
   byte-unchanged (golden 66/66 + FP grid + regression snapshot all unmoved).
6. **COSE_Sign1 verification deferred** (sub-item 4 — coupled to org/0006 §7's open root-of-trust);
   this layer decodes the payload only. **One-vs-two channels** (sub-item 3) also stays open.

## Findings (surfaced, not papered over)

- **Terrain skyline sign** (bug caught by the differential test): the horizon-profile angle is the
  **signed skyline altitude** (positive = a ridge above the astronomical horizon → *later* sunrise), so
  the centre-altitude target is `angle − semidiameter`, not `−(semidiameter + angle)` (the dip
  convention, which inverts mountains). A pure sea-horizon dip is just a negative angle, so the
  convention stays consistent with `Visible`.
- **Refraction-model namespace gap:** the spec's `refraction.model {standard-atmospheric, meeus-noaa,
  halachic-fixed-coefficient}` is broader than the engine's `RefractionModel {None, Saemundsson,
  Bennett}`. The reader maps `standard-atmospheric → Bennett`/None (/0013) and returns **`Unimplemented`**
  for `meeus-noaa` / `halachic-fixed-coefficient` (real options, not yet built — not silently substituted).
- **horizon_mode 3-vs-2:** spec §1.B lists only `{sea-level, terrain-profile}` but the engine has three
  modes (`visible` added in /0013). The reader accepts 0/1/2; the spec narrative should add `visible`.
- **Fixed-behaviour knobs:** `solar_position_reference` / `solar_limb_reference` are currently baked
  (apparent / upper-limb, /0013). The reader accepts only the matching values (0/0) and flags others
  `Unimplemented` rather than ignoring them.

## Consequences

- New: `tests/wire.rs` (5 — round-trip, resolve, interpolation, binding, malformed-no-panic),
  `tests/terrain.rs` (the moat differential: flat ≈ Mishor; uniform shifts both; east-only shifts netz
  not shkia). `fuzz_no_panic` gains a 100k-iteration decoder fuzz; the regression snapshot gains 4
  terrain rows (120 total). **FP-determinism 456 → 536** native==wasm (+azimuth +terrain probes,
  kinds 15/16). **F1 66 / F2 11 / F3 38 / couplings / properties / cross-engine all unchanged-green.**
- `minicbor` links in the freestanding `no_std`/no-alloc wasm build (the shipped reader is heap-free;
  dev-deps never reach the lib/wasm build). clippy `-D warnings` clean, fmt clean.

## Open / flagged

COSE verification ↔ org/0006 §7 root-of-trust (sub-item 4); one-vs-two channels (sub-item 3); the
refraction-model + horizon_mode + fixed-behaviour spec gaps above; `meeus-noaa` /
`halachic-fixed-coefficient` refraction models; `fixed-minute-offset` read-spec + the `zman_definitions`
catalog + `obligation_sense`/rounding plumbing (Phase 4b — the engine lacks those types); Israel DTM
(/0004, unaffected — the reader consumes profiles, does not build them).

## Related

core-domain/0002/0009 (param-vector schema), /0004 (horizon profile + binding), /0006 + /0013 (optics
seam; the terrain target sign), /0007 (tz-free instants), /0008 (cross-language writer rule), /0011
(CBOR/CDDL/COSE — sub-items 1 & 2 closed here), /0010 (no_std/FP, extended), /0017 (no-panic, extended
to the reader). Meta: org/0006 (§7 root-of-trust, signed-artifact). Phase B: provisioning-builder writes
these two CDDL contracts.
