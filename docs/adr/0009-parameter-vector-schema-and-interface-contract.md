# ADR core-domain/0009 — Parameter-vector schema + F1/F2/F3 interface contract

- **Status:** Accepted
- **Date:** 2026-06-16
- **Scope:** Stack-agnostic domain model. Resolves the open question owned by core-domain/0002 (the
  exact parameter-vector schema) and closes the refraction-model **default** sub-question flagged by
  core-domain/0006. Names no language, framework, runtime, or serialization format. The core
  **exposes** every knob and **resolves none**; no halachic value is selected here.

## Context

core-domain/0002 made variation a matter of parameters over the three first-order functions of
core-domain/0001, and flagged the **exact parameter-vector schema** as an open question that
**blocks F1/F2/F3 interface finalization** (also referenced by core-domain/0001, core-domain/0004,
core-domain/0006, core-domain/0008). core-domain/0006 left the **refraction-model choice** open, to
be settled jointly with the schema. core-domain/0008 handed the **engine-selection** knob to this
schema thread as a *correctness-bearing* parameter.

A DRAFT proposal worked the schema, the interface contract, and three choices-for-ratification.
This ADR records the ratified outcome; the full living contract is
`docs/spec/parameter-vector-and-interface-contract.md`.

## Decision

Adopt the parameter-vector schema and the F1/F2/F3 interface contract as specified in
`docs/spec/parameter-vector-and-interface-contract.md`. Salient points:

1. **Zmanim are data, not code.** Each zman is a typed **read-spec** off F1's `altitude(t)` curve
   (`depression-angle` · `horizon-crossing` · `fixed-minute-offset` · `proportional` · `extremum`).
   "GRA vs Magen Avraham" and "day definition (netz→shkia vs alot→tzeit)" are settings of one
   `proportional_day_bounds` knob, not branches. Adding a community opinion = adding a data row.
2. **Terrain/elevation ON/OFF, refraction model + horizon optics** are knobs consumed by F1
   (core-domain/0004, core-domain/0006); provisioning and runtime must read the **same** refraction
   model so the shipped horizon profile composes correctly.
3. **engine-selection is a required, correctness-bearing knob** (core-domain/0008) — same class as
   the day-boundary knobs; the no-drift guarantee holds only within one engine choice.
4. **Civil time / DST and all labeling are outside the core** (core-domain/0007): the labeling knobs
   travel in the vector but are consumed at the display/input boundary, never by F1/F2/F3.
5. **The four core-domain/0001 couplings are declared as typed dependencies** (F3 day-roll consumes
   an F1 read; F3 day-type selects which F1 reads apply; tal-u-matar = an F1-class tekufa gated by
   F3 date + realm; Kiddush Levana = an F3 window confirmed by F2 visibility).

**Ratified choices:**

- **A — Refraction-model default:** `standard-atmospheric` (Bennett/Saemundsson-class) is the
  shipped base-preset default; `meeus-noaa` and `halachic-fixed-coefficient` are exposed as
  selectable alternatives. The core still requires the selector and supplies no default — the
  default lives in the configuration/preset layer (core-domain/0002). **This closes
  core-domain/0006's refraction sub-question** (the *default* is chosen; the *selector* remains a
  knob).
- **B — Required-vs-optional cut:** required (the core errors if absent for a requested output) =
  `schema.version`, `engine.selection`, `locale.realm`, `horizon.mode`, `refraction.model`,
  `solar.position_reference`, `solar.limb_reference`, plus the conditionals (terrain profile,
  fixed-coefficient, proportional bounds, rounding, tal-u-matar, yahrzeit) and each requested
  zman's read-spec; everything else is optional/feature-gated (absent → not emitted, never
  silently defaulted).
- **C — Preset/stream precedence:** `base preset → per-tenant override → per-site override`,
  last-writer-wins per knob-key with deep-merge of `zman_definitions` (by zman-key) and the
  customs/label maps. Layering happens entirely in the configuration layer; the core receives one
  flattened vector and only validates completeness.

## Rationale

- **A single deterministic core, parameterized by data, stays oracle-testable and auditable**
  (core-domain/0002, core-domain/0003): the read-spec union makes every opinion a fixture, not a
  branch.
- **The standard-atmospheric refraction model is continuous in altitude**, so it composes with
  terrain horizon profiles (core-domain/0004) at arbitrary horizon angles, where a fixed
  coefficient — calibrated only at the sea-level horizon — degrades. It is oracle-comparable;
  `meeus-noaa` aids cross-validation against the baseline/`zmanim-core` oracle (core-domain/0008);
  the halachic fixed coefficient remains available because some poskim mandate it.
- **Keeping resolution and defaults outside the core** preserves "the core resolves none": an
  unset *required* knob is a configuration error, not a silent default (core-domain/0002).

## Alternatives considered

- **Enumerate a fixed list of zmanim with baked-in values.** Rejected: smuggles policy into the
  core, defeats data-driven testing, and contradicts core-domain/0002.
- **Hard-code one refraction model / sea-level horizon.** Rejected by core-domain/0006: collapses a
  required parameter into a constant and blocks visible sunrise.
- **A fixed halachic coefficient as the default.** Rejected as a *default*: it does not generalize
  to terrain/elevated horizons (kept as a selectable alternative).
- **Perform layering/defaulting inside the core.** Rejected: violates "core resolves none";
  layering stays in the configuration layer.

## Consequences

- **F1/F2/F3 interfaces are now finalizable** — the blocking open question of core-domain/0002 is
  resolved; the living contract is `docs/spec/parameter-vector-and-interface-contract.md` (the
  repo's first non-ADR artifact).
- **core-domain/0006's refraction sub-question is closed** for the *default*; the selector and the
  provisioning↔runtime same-model invariant remain as specified.
- **engine-selection** is recorded in the schema as required and correctness-bearing
  (core-domain/0008).
- **High-latitude fallback (deliberate deferral, not a new decision):** when a primary read returns
  `does-not-occur` (e.g. Alot/Tzeit R"T at −16.1° at high latitude in summer), the substitute to
  display is a community-supplied alternate read selected at the preset/edge layer — expressed with
  the existing `fixed-minute-offset` read-spec — not a new core knob. The core resolves none: it
  returns the typed `does-not-occur`; choosing what to show instead is a preset/edge policy
  (consistent with the spec §1.F / core-domain/0007).
- **Still open (unchanged):** the **on-wire serialization** of the parameter vector + horizon
  profile is a cross-repo encoding contract (core-domain/0008) — this ADR fixes only the *logical*
  shape; and the **Israel high-resolution DTM source** (core-domain/0004) is untouched.
- The DRAFT proposal is superseded by this ADR + the spec and is removed.

## Related

core-domain/0001 (the three functions + couplings), core-domain/0002 (parameters; owns the schema
question — now resolved here), core-domain/0004 (terrain horizon profile; refraction must match),
core-domain/0006 (refraction seam; default closed here), core-domain/0007 (civil time / labeling
outside the core), core-domain/0008 (engine-selection knob; serialization open). Meta: `org/0004`
(correctness/management seam), `org/0006` (edge envelope; runtime Profile A).
