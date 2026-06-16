# ADR core-domain/0015 — F2 (lunar geometry) + molad-moment + Kiddush Levana

- **Status:** Accepted
- **Date:** 2026-06-16
- **Scope:** F2 (lunar geometry) of ADR core-domain/0001, the precise **molad-moment** deferred by
  /0014, and the **Kiddush Levana** window. New `src/lunar.rs`, `src/kiddush_levana.rs`; molad in
  `src/calendar.rs`; `Body` generalization in `src/events.rs`; KL knobs in `src/params.rs`. Stack:
  Rust/no_std (core-domain/0010, /0013).

## Context

F2 is the last of the three deterministic domain functions (F1 solar green 66/66; F3 calendar exact
27/27). The Moon is needed for moonrise/moonset and as the visibility input to Kiddush Levana; the
**molad** (mean conjunction) is calendar arithmetic (F3), distinct from the observed moon (/0001),
and was deferred from /0014 to land here with KL.

## Decision

1. **Lunar position = Meeus *Astronomical Algorithms* ch. 47 (abridged ELP-2000/82)** — full 47.A
   (longitude/distance, 60 terms) + 47.B (latitude, 60 terms) + additive terms. Validated **exactly
   against Meeus's own worked example 47.a** (λ=133.162655°, β=−3.229126°, Δ=368409.7 km). Double
   precision, all trig via `libm` (the /0010 FP gate applies).
2. **Topocentric parallax is mandatory, not optional** — geocentric→topocentric reduction (Meeus
   ch. 40) with elevation-aware observer coordinates; the Moon's parallax (~1°) is first-order at
   the horizon. (A sign error here — `H' = H + Δα` instead of `H − Δα` — was caught by oracle
   measurement: it inverted the parallax and put altitude ~1.4° high; the fix brought engine
   apparent altitude to **0.0025°** of Wolfram. **Measure, don't hypothesize** — the /0013 lesson.)
3. **Moonrise/moonset reuse the F1 crossing machinery** via a `Body { Sun, Moon }` parameter
   threaded through `effective_alt_deg`/`find_crossing`/`bisect` (the Sun path stays
   byte-for-byte unchanged — regression-guarded by 66/66 + FP-determinism). The event target is the
   shared `optics::horizon_target_deg = −(semidiameter + dip)` with refraction added by the solver —
   but with the Moon's **distance-dependent semidiameter** (`sin s = 0.272481·sin π`, ~14.7′–16.7′),
   not the Sun's fixed 16′. The Moon search spans a **full civil day** (rise/set don't bracket local
   noon as the Sun's do).
4. **Molad-moment = exact-integer chalakim** (D–R molad: `months_elapsed·765433 + epoch`, the synodic
   month = 29d 12h 793p). Pinned by the canonical **BaHaRaD** anchor (molad Tishrei AM 1 = 5h 204
   chalakim into Monday night — reproduced **exactly**) + the exact 765433-chalakim interval; the RH
   derivation that consumes it is already triple-validated (/0014, 23 rows). Structural determinism
   (no float) for the chalakim; **only the UT projection (`molad_instant`) is float** and carries a
   **flagged meridian assumption** (see Open).
5. **Kiddush Levana window** = molad + knobs: **`KiddushLevanaStart` {ThreeDays (Rema/Ashkenaz,
   default), SevenDays (Sephardi/AriZal), Molad}** and **`KiddushLevanaEnd` {HalfMonth (Rema, molad +
   ½ synodic, default), FifteenDays}**. The core resolves none. A thin F2 `moon_visible` primitive
   (topocentric apparent altitude > 0) ships too.
6. **Oracle = Wolfram `MoonPosition`** (primary, apparent topocentric altitude) + the moonrise/set
   reciprocal scan; molad anchored on BaHaRaD + Hebcal-confirmed RH derivation (build/test only,
   /0003/0008).

## Scope boundary & deferrals

In: lunar topocentric position/altitude, moonrise/moonset, distance-dependent semidiameter,
molad-moment (chalakim + civil + UT instant), KL window bounds + knobs, `moon_visible`. **Deferred to
Phase 3 (the /0001 couplings):** the *full* Kiddush Levana answer = **window ∩ F2 moon-up ∩ F1
night** (mirrors /0014's F3↔F1 day-roll deferral). Also deferred: nutation in the lunar position
(<0.01° — immaterial at the ±1-min/arc-second bar), time-varying ΔT (provisional +69 s, /0012).

## Rationale

The hard, new physics in F2 is the ELP series + topocentric parallax; both are validated to
arc-seconds against an independent oracle, so reusing the already-proven F1 crossing engine (rather
than a parallel lunar event path) keeps one tested code path. The molad is fixed arithmetic like F3,
so it is exact and anchored on the universally-agreed BaHaRaD — no precision/tolerance question for
the chalakim; the single UT-projection float is isolated and flagged. Halachic conventions are
knobs (/0002/0009), and the night∩visible intersection is a *coupling*, kept for the coupling phase.

## Consequences

- `tests/lunar.rs` validates **11/11** F2 vectors: 7 apparent-altitude rows (≤19″ vs Wolfram) +
  4 moonrise/moonset rows (within ±18 s, tolerance ±60 s tied to /0003). `tests/calendar.rs` grows
  to **27/27** (+4 molad rows) plus a dedicated molad/KL test (BaHaRaD 5h 204ch, exact 765433
  interval, KL window offsets). F1 golden **66/66** unaffected (Sun path unchanged).
- **FP-determinism extended to F2:** `ffi.rs` probes 7–10 (moonrise/moonset/moon-alt/molad) →
  **15/15** exact native==wasm. no_std-clean; clippy clean (`deny(missing_docs)`/`unsafe_code`).
- Below-horizon apparent altitude is **out of scope** (refraction undefined below 0°; the one
  below-horizon oracle point diverged 0.1° and was excluded — position is validated by the
  above-horizon rows to arc-seconds).

## Open / flagged

- **Molad meridian convention** (`MOLAD_MERIDIAN_DEG_EAST = 35.2354°`, Jerusalem ≈ traditional
  2h21m): the molad's day-of-week + hour + chalakim are meridian-free and exact; **only the
  absolute-UT projection depends on this constant**. It is an *assumption*, not a derived fact —
  surfaced here, not buried. KL windows are days wide, so this is immaterial to their practical use.
- **KL start (3 vs 7 days)** is genuinely contested (Ashkenaz vs Sephardi) → a knob with a documented
  default, never a silent single answer.

## Related

core-domain/0001 (F2 in the three-function model; molad ≠ observed moon), /0002 (knobs), /0003 +
/0008 (oracle), /0006 + /0013 (refraction/horizon optics seam, reused for the Moon), /0009
(parameter-vector; KL knobs), /0010 (no_std/FP determinism, extended to F2), /0014 (F3; molad
deferral fulfilled here). Phase 3: the /0001 couplings. Meta: org/0006.
