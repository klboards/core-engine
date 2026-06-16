# ADR core-domain/0017 — Test hardening: properties, fuzz, determinism breadth, cross-engine

- **Status:** Accepted
- **Date:** 2026-06-17
- **Scope:** A robustness/regression pass over the now-complete F1/F2/F3 + couplings engine. No new
  engine capability — new test surfaces (`tests/properties.rs`, `tests/fuzz_no_panic.rs`,
  `tests/regression.rs`, `tests/offline_autonomy.rs`, `tests/cross_engine.rs`), an expanded
  FP-determinism grid, a provisional CI workflow, and **two correctness fixes the new tests surfaced**.

## Context

The oracle suites (Wolfram/Hebcal/MyZmanim) prove correctness *at sampled points*; they don't prove
behaviour *across the input domain*, guard against panics (a `no_std` panic is `loop{}` — a hung
device), lock values against silent regression, or cross-check against an independent engine. For an
offline, unattended, observance-relevant engine with an explicit "never silently wrong" invariant
(org/0006), that breadth is correctness-bearing, not gold-plating.

## Decision

Layer the suite by what each kind of test guarantees:

1. **Property / invariant** (`tests/properties.rs`, dependency-free seeded SplitMix64, 2k–20k iters):
   Gregorian/Hebrew round-trips + date validity; molad interval exactly synodic; tekufa spacing
   exactly year/4 (Shmuel); weekday consistency; `classify_day` invariants; KL window well-formed;
   **zman ordering** (alot < netz < chatzot < shkia < tzeit); day-roll advances ≈ one Hebrew day per
   solar day (metamorphic).
2. **Fuzz-for-panic** (`tests/fuzz_no_panic.rs`, ~125k adversarial calls): hammer the C-ABI probe
   (`probe_zman_nanos`, the device/FFI surface) and the readers with extreme/NaN/∞ inputs and
   out-of-range month/season — assert every call returns (device-safety).
3. **Determinism breadth** (`examples/fp_probe_native.rs` grid → `tools/fp_probe.mjs`): the native==wasm
   exact-`i64` gate expanded from 20 to **456** points across latitude/longitude/elevation/hemisphere/
   season + the year-based kinds.
4. **Golden regression snapshot** (`tests/regression.rs` + `tests/fixtures/regression_snapshot.csv`,
   116 rows): recompute-and-compare locks every current output against silent change; `BLESS_SNAPSHOT=1`
   regenerates deliberately. Guards *change*, not correctness.
5. **Cross-engine differential** (`tests/cross_engine.rs` vs **KosherJava**): KosherJava's sea-level
   NOAA sunrise/sunset (zenith 90.833°) matched by our `DepressionAngle{0.8333°, geometric}` —
   isolating the solar-position algorithm. Oracle-as-committed-fixture (`tools/KosherDiff.java` →
   `tests/fixtures/kosherjava_vectors.csv`); the LGPL jar is a build-tool, never vendored/shipped.
6. **Offline-autonomy** (`tests/offline_autonomy.rs`): a 10-day Sukkot 5787 chag+Shabbat span computes
   every zman + day-roll + day-type with zero network, asserting coherent advance (open-decisions #9).
7. **Provisional CI** (`.github/workflows/ci.yml`): fmt/clippy/test/wasm/FP-determinism. Flagged
   provisional — does **not** ratify GitHub Actions (open-decisions #9 pipeline tooling stays open).

## Bugs found and fixed (the hardening earned its keep)

- **High-elevation horizon-crossing misfire (F1, the visible-sunrise / moat path).** `find_crossing`
  judged rising-vs-setting from the **effective** (refracted) altitude slope. At a large elevation dip
  (e.g. 1868 m → target ≈ −1.66°) Bennett refraction extrapolated below the horizon is non-monotonic
  and faked an ascending crossing during the evening descent, returning an evening "sunrise". **Fix:**
  judge the slope from the **geometric** altitude (refraction can never turn a sunset into a sunrise);
  regression-safe (only rejects wrong answers — golden 66/66 and FP-determinism unchanged). Pinned by
  `regression_high_elevation_netz_is_morning`. (The below-horizon-refraction caveat is ADR-0013's.)
- **Far-future overflow panic (Kiddush Levana).** `shift_days` used checked `+`; for Hebrew years
  beyond ~6022 (≈ 2262 CE, the `i64`-nanosecond `AbsoluteInstant` domain limit) the molad approaches
  `i64::MAX` and the offset overflowed — a debug panic / release silent-wrap. **Fix:** `saturating_add`
  (defined clamp at the domain edge, honouring "never silently wrong").

## Open / flagged (not resolved here)

- **Input-domain guarding:** `hebrew_from_fixed` uses a correct-then-adjust search bounded only for
  in-domain Rata Die; an absurd far-future/past instant could iterate long. On-device the clock is a
  trusted bounded source, so not a live hazard — but boundary saturation/clamping is an explicit open
  robustness item (the fuzz bounds time inputs to sane ranges).
- **Cross-engine scope:** sea-level sunrise/sunset only (isolates solar position). Elevation/terrain
  and depression-shitot differentials vs KosherJava are deferred (convention alignment needed).
- **Offline-autonomy `N`:** the *certified* window is still open (open-decisions #5); the test uses a
  representative 10-day chag+Shabbat span.
- **CI provider** (open-decisions #9) and the `AbsoluteInstant` ~2262 CE horizon remain open.

## Consequences

- New: properties **10/10**, fuzz **3/3** (~125k calls, no panic), regression **116/116**,
  offline-autonomy **1/1**, cross-engine **48/48 vs KosherJava (max residual ≈ 2.1 s)**;
  FP-determinism **456/456** native==wasm. Existing F1 66 / F2 11 / F3 38 / knobs 3 unchanged-green.
- Two latent correctness bugs fixed (one on the moat path), each now regression-guarded.
- clippy clean (`-D warnings`), fmt clean, `no_std` wasm builds. No new runtime dependency (the PRNG is
  hand-rolled; KosherJava/Java are build-time only via committed fixtures).

## Related

core-domain/0003/0008 (KosherJava = build/test oracle + the cross-check of open-decisions #9), /0005
(offline autonomy), /0006 + /0013 (the optics seam; the below-horizon-refraction caveat behind the
high-elevation fix), /0009 (the read-spec surface under test), /0010 (FP-determinism, broadened),
/0014–/0016 (F1/F2/F3 + couplings, the systems under test). Meta: org/0006 (never-silently-wrong).
