# ADR core-domain/0013 — F1 optics: apparent-limb horizon, geometric depression, refraction-as-knob; type-safe engine architecture

- **Status:** Accepted
- **Date:** 2026-06-16
- **Scope:** F1 (solar) optics + engine architecture, driven by direct **Wolfram-oracle
  measurement** this session. **Supersedes the optics decisions of core-domain/0012** (which were
  based on a misread of the fixture README's "−1.144°" — that figure is the *apparent* altitude,
  not a geometric depression). Refines core-domain/0006; notes /0003 and /0009; reconfirms
  /0004 (DEM). Does **not** change the solar series.

## Context (what the oracle measurement showed)

- **Solar position is accurate.** Engine RA/Dec match Wolfram (the fixtures' oracle) to
  **0.006° / 0.0003°**. The earlier pass-4 conclusion — "low-precision series insufficient, swap to
  a higher-precision series / relax tolerance" — is **withdrawn**. Do **not** touch the solar series.
- **Wolfram sunrise = apparent upper-limb at the dipped horizon.** Near the horizon, refraction is
  physical and **Bennett matches Wolfram to ≤0.035°**.
- **Below the horizon, Wolfram's "refraction" is a non-physical model artifact** (~1.5° near −4°,
  collapsing to ~0.07° by −16°) — there is no light path 16° below the horizon, and no standard
  refraction model reproduces it (nor should it). That ~0.07° artifact is what amplified, via the
  shallow high-latitude solstice rate, into the ~1–2 min depression misses.

## Decision

1. **netz/shkia (HorizonCrossing):** `apparent = geometric + Bennett(refraction)`; the event is
   `apparent center = −(solar semidiameter ≈16′ + horizon dip)`. Physical, near-horizon, matches the
   oracle (~±15 s at all elevations). **Refines core-domain/0006.**
2. **Elevation / `HorizonMode` knob:** `Mishor` (dip = 0) | `Visible` (dip = `acos(Rₑ/(Rₑ+h))`,
   computed **in-core**) | `TerrainProfile` (core-domain/0004 skyline, future). This **reverses
   0012's "defer all elevation to 0004"** — the point-elevation visible horizon is in-core; the DEM
   skyline (Copernicus GLO-30, /0004) remains the refinement for true terrain obstruction.
   `Mishor` vs `Visible` is the halachic **sea-level vs visible-horizon** split.
3. **Depression shitot (alot/misheyakir/tzeit/R"T):** the sun's **GEOMETRIC** center depression
   (refraction OFF) — the classical/halachic definition (KosherJava-aligned) and physically correct;
   it avoids baking Wolfram's non-physical below-horizon artifact into a correctness anchor. The
   fixture's depression rows are **regenerated refraction-off** to reflect this geometric definition
   (the README's stated "refraction-independent geometry").
4. **Refraction-application is a knob** (core-domain/0006 Choice A): Bennett/Saemundsson-class model;
   **default = apparent for the horizon event, geometric (off) for depression**. A posek/community
   may select otherwise. The exact below-horizon behavior is *not* chased (non-physical there);
   residuals are governed by the ±1-min tolerance (core-domain/0003, sub-second still OPEN).
5. **Solar series unchanged** (validated accurate); **ΔT (TT = UT + ΔT)** retained (Wolfram itself
   uses TT). 
6. **Engine architecture (Rust best practices):** make the apparent/geometric bug class
   unrepresentable — newtypes `GeometricAltitude` vs `ApparentAltitude` (refraction is the *only*
   conversion), `Angle`/`JulianDay`/`UnixNanos`; `RefractionModel` and `HorizonMode` as
   **serializable enum knobs** (no_std, alloc-free, CBOR-ready per /0011); module layering
   `time / geometry / optics / events / params`; begin the **core-domain/0009 parameter-vector**
   Rust types. Preserves no_std + the /0010 exact cross-target determinism.

## Rationale

Match the neutral oracle where it is physical (horizon refraction); use the classical **geometric**
definition where the oracle is non-physical (depression); expose **every** convention as an
auditable **parameter** (halachic determinism — the core resolves none, the psak lives in the
vector); and encode the apparent/geometric distinction in the **type system** so the recurring bug
class cannot recur.

## Alternatives considered

- **0012's 50′-flat geometric target / remove-dip.** Rejected: built on the misread apparent
  −1.144°; cannot match the oracle across elevations.
- **Apparent depression (replicate Wolfram's below-horizon artifact).** Rejected: bakes a
  non-physical quirk into the correctness engine; diverges from the classical geometric shita.
- **Swap the solar series (VSOP87 / relax tolerance).** Rejected: position is validated accurate;
  the error was optics, not precision.

## Consequences

- F1 expected to validate **green (±1 min)** after implementation: netz/shkia (apparent+dip),
  depression (geometric), incl. Jerusalem 754 m (`Visible` dip reinstated) and high-latitude rows.
- The fixture's **depression rows are regenerated geometric**; netz/shkia/proportional(GRA) rows
  are unchanged; MGA-proportional expecteds (which depend on alot/tzeitR"T) are recomputed.
- `Mishor | Visible | TerrainProfile` and the refraction model become **knobs**; type-safe refactor
  + start of the /0009 schema types land with the implementation.
- Standing invariants reconfirmed: **#1** no_std ⇒ `libm`-enforced determinism gate (/0010);
  **#2** core emits `Option<AbsoluteInstant>` only, civil-day/tz at the edge (/0007).

## Related

core-domain/0001 (F1), /0003 (oracle/tolerance — noted), /0004 (DEM/terrain — reconfirmed),
/0006 (refraction — refined), /0009 (parameter-vector schema; `HorizonMode`/refraction knobs),
/0010 (determinism gate), /0012 (optics superseded here); org/0006.
