# ADR core-domain/0012 — Validation-driven F1 refinements: horizon-event convention (50′ sun-center), solar TT/ΔT, DEM reconfirmation

- **Status:** Accepted
- **Date:** 2026-06-16
- **Scope:** F1 (solar) refinements forced by first contact with the Wolfram oracle
  (`tests/fixtures/golden_vectors.csv`). Refines core-domain/0006 (refraction) and core-domain/0003
  (oracle/tolerance); reconfirms core-domain/0004 (DEM). Records decisions only; the per-row
  validation numbers are an engineering result, not part of this ADR.

## Context

The F1 engine was validated against the 69-row Wolfram-verified oracle. Two systematic gaps
surfaced (quantified, refraction-independent depression rows isolating the solar core from the
horizon optics):

- **Horizon (netz/shkia):** uniformly off (netz ≈ +180 s, shkia ≈ −150 s), **independent of
  elevation** → the engine's bare Bennett horizon refraction (34.5′) is ~16′ too shallow.
- **Solar (depression rows):** a smaller symmetric day-length error growing with latitude near the
  solstice → solar-position accuracy.

The oracle's convention was derived (not guessed): **Wolfram `Sunrise`/`Sunset` place the sun's
CENTER at 50′ (0.8333°) below the horizon** — 34′ refraction + 16′ apparent solar radius bundled
into one standard, exposed as `ReferenceAltitude`. The README confirms sun-center, no *separate*
semidiameter term — the 16′ is folded into the 50′.

## Decision

1. **Horizon-crossing event = 50′ (0.8333°) sun-center standard depression.** The netz/shkia event
   uses the standard `−0.8333°` center depression (refraction + solar-radius bundled), matching the
   oracle. This **refines core-domain/0006**: Bennett (and the other Choice-A models) remain the
   selectable *general* refraction model, but the sunrise/sunset *event* baseline is the 50′
   standard, exposed as a **`reference_altitude` knob** (aligns with the core-domain/0009 schema).
2. **Solar position at TT = UT + ΔT.** The sun's position is computed at Terrestrial Time
   (`ΔT ≈ +69 s` for 2026); Greenwich sidereal time stays on UT. A **time-varying ΔT model/table is
   a flagged TODO**. **Notes core-domain/0003:** the low-precision NOAA/Meeus series against a
   ±1-min bar is **marginal at high latitude near the solstice**; the sub-second oracle tolerance
   remains OPEN, and a higher-precision solar series may be required (open).
3. **Elevation is the DEM/terrain path, not a core point-dip.** The global open DEM is **Copernicus
   GLO-30** (open license, best-in-class 30 m, AWS Open Data) — **reconfirms core-domain/0004**
   (GLO-90 coarse fallback; some GLO-30 tiles withheld). Accurate **elevated/terrain horizons are
   the core-domain/0004 horizon-profile (provisioning) path** composed on-device with F1; the F1
   core itself computes the **sea-level astronomical horizon** (50′). A geometric elevation dip is
   retained in the code only as a flagged interim helper, **not** the oracle-matched elevated model.

## Rationale

- **Match the neutral oracle's published convention** (core-domain/0003 validation): the 50′
  sun-center standard is exactly Wolfram's definition; bare Bennett 34.5′ structurally cannot match.
- **One engine, knobs not branches** (core-domain/0002/0009): the horizon baseline is a
  `reference_altitude` parameter, not a code path.
- **Elevation correctness belongs to the DEM/terrain path** (core-domain/0004), where it is done
  properly from real terrain, **not** a one-point dip fit to the single elevated fixture site
  (which would be overfitting).

## Alternatives considered

- **Keep bare Bennett (34.5′) at the horizon.** Rejected: ~16′ too shallow; cannot match the oracle.
- **Bennett + an explicit 16′ semidiameter term.** Numerically equivalent to the 50′ standard but
  reintroduces a separate limb term the convention bundles; rejected for the event baseline (kept
  conceptually as the `reference_altitude` knob's meaning).
- **One-point dip fit so Jerusalem 754 m goes green.** Rejected: overfitting a single elevation
  sample; elevation is the core-domain/0004 terrain path.

## Consequences

- Low-elevation netz/shkia now validate to the ±1-min bar; **elevated-horizon (e.g. Jerusalem
  754 m) is explicitly DEFERRED to the core-domain/0004 terrain-profile path** and reported as
  known-deferred, not a failure.
- **High-latitude-near-solstice depression rows may remain off ~1–2 min** — a solar-precision
  matter (core-domain/0003): either a higher-precision solar series or an accepted-tolerance
  decision, both OPEN. Not resolved here.
- **ΔT** is provisional (2026 constant); a real ΔT model is a TODO (core-domain/0003/0007).
- Standing engineering invariants reconfirmed: **(no_std⇒libm)** determinism is compile-enforced —
  the freestanding wasm build is the gate (core-domain/0010); **(edge boundary)** the F1 core emits
  `Option<AbsoluteInstant>` only, with civil-day/tz labelling at the edge (core-domain/0007).

## Related

core-domain/0001 (F1), core-domain/0002 (knobs), core-domain/0003 (oracle/tolerance — noted),
core-domain/0004 (DEM/terrain — reconfirmed), core-domain/0006 (refraction — refined),
core-domain/0009 (`reference_altitude` knob), core-domain/0010 (determinism gate); org/0006.
