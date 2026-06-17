# ADR core-domain/0019 — The poskim spectrum is second-order; the first-order core stays mechanism-only

- **Status:** Accepted
- **Date:** 2026-06-17
- **Scope:** A **layering ruling**, not new engine code. Fixes where the multi-axis halachic spectrum
  for sunrise/zmanim lives: in the **second-order parameter/preset layer** (management side, org/0004),
  *over* first-order outputs — **not** in the deterministic core. Records the one demonstration test
  (`tests/second_order.rs`) and the only genuine first-order mechanism gaps. No knobs added to the core.

> **Follow-ups (later ADRs):** the three first-order gaps in §4 are now **closed by core-domain/0020**
> (`limb_reference`, `fixed-minute-offset`, `decode_read_spec`). The second-order layer's home is the
> **`halacha-model`** bounded context (**org/0010**); its knowledge sources + license-gating are org/0011.

## Context

Pressing on "what about the other poskim?" surfaced that sunrise/zmanim disagreement is **multi-axis**,
not a mishor-vs-nireh binary: horizon reference (astronomical-at-height / sea-level / sea-level-at-
altitude / **fixed reference altitude** — Sh"A HaRav & Igrot Moshe O"C 1:97 "all at Jerusalem ~800 m" /
**visible terrain** — ChaiTables, Jerusalem poskim); the **netz-definition** dispute (beginning of the
disc's ascent = upper limb, majority/Biur Halacha, vs **end** of ascent = lower limb, Ish Matzliach /
Yalkut Yosef 89:3); shaos-zmaniyos basis (GRA/MA); **area / radius-of-influence** (exact point vs
community-earliest); refraction; and **layered** setups (Baal HaTanya: standard netz/shkiah for the
hours, a degree-offset for the actual netz/shkiah). A draft wrongly proposed pushing presets / area
policy / a poskim catalog into the engine.

## Decision

1. **First-order core (core-engine) = convention-free mechanism.** Given `(φ, λ, h, t)` + a parameter
   vector it computes raw positions and *typed reads* against thresholds (ADR-0001/0009). It **selects
   no shita**; it must only be **expressive enough** that any posek's vector composes from its
   primitives. It is oracle-validated and stays lean.
2. **The entire poskim spectrum is SECOND-ORDER** — "variation is parameters over first-order outputs,
   not code paths" (ADR-0002), realized by "the cloud config/template layer binding parameter vectors
   per tenant" (ADR-0009 §(b)), on the management side of the org/0004 seam. The posek→vector mapping,
   the **preset library**, mishor-vs-nireh defaults, the **area/radius-of-influence** policy, and the
   **multi-luach validation** all live there, never in the engine.
3. **Most axes need ZERO engine change** — they are parameter *use* over existing primitives:
   - Fixed reference altitude (Sh"A HaRav / Igrot Moshe) → the second-order layer **passes
     `elev = 800 m`** to the existing reads.
   - Area / radius-of-influence (ChaiTables community-earliest) → **`min` over several engine reads**.
   - mishor / sea-level-at-altitude / nireh → pick `horizon_mode` (+ supply/omit a horizon profile).
   - Each posek/luach → a **parameter vector + preset**, layered base → tenant → site.
   - Refraction / shaos-zmaniyos basis → pick `refraction.model` / `proportional_day_bounds`.
   Demonstrated in `tests/second_order.rs` against the **unchanged** engine (no core edit).
4. **The ONLY genuine first-order gaps** (small, convention-free mechanism; deferred follow-ons, not
   this pass): `solar.limb_reference {upper,center,lower}` (the netz-definition `±semidiameter` shift;
   only upper-limb wired today); the `fixed-minute-offset` read; the `zman_definitions` read-spec map
   decode (Phase 4b). **Nothing else** about the spectrum belongs in the core.
5. **Validation is a panel** (second-order, non-circular per /0003/0008, open-decisions #9): each vector
   vs *its* luach — mishor ↔ MyZmanim/OU; visible ↔ **ChaiTables** (observation-validated, Rav Druk);
   Sephardi/Yalkut Yosef ↔ **royzmanim "Zemaneh Yosef"** (open-source, uses ChaiTables); Chabad/Baal
   HaTanya ↔ **KosherJava**; Jerusalem ↔ Tukachinsky/Itim L'Bina. Raw astronomy stays **Wolfram** only.

## Rationale

Keeping policy out of the deterministic core preserves the properties that make it valuable: one
oracle-validated, bit-reproducible, offline engine with no per-stream branches (ADR-0002/0008/0010).
The spectrum is genuinely *parametric over* the engine's outputs, and the demonstration test shows the
current primitives already span the named positions by substitution — so the architecture is vindicated
*and* the core does not grow. Placing presets/area/validation in the management layer matches the seam
(org/0004) and the schema's preset-precedence (ADR-0009 base→tenant→site).

## Consequences

- **No engine knobs added.** `tests/second_order.rs` proves fixed-reference-altitude (via `elev`),
  community-earliest (via `min` over reads), and mishor-vs-visible all work over the unchanged engine.
- The preset library, posek→vector resolver, area policy, provisioning (DEM→profile, the **free**
  NASADEM/GLO-30 path) and the validation panel are **management/preset-layer** work (future repos),
  not core-engine.
- First-order follow-ons, when scheduled, are limited to the three mechanism gaps above.

## Open / flagged

Whether `limb_reference` warrants a first-order knob vs expressing end-of-ascent as a depression read
(decide when scheduled); which presets ship first (second-order, posek-set; default mishor);
ChaiTables/luchot access for the oracle panel; Israel DTM via the free path (/0004). All policy =
posek choices, never silent; the engine resolves none.

## Related

core-domain/0001 (first-order model), /0002 (parameters-over-outputs, no denomination engines), /0006
(physics in F1, conventions as parameters), /0008 (own engine, no per-posek forks), /0009 (parameter-
vector + the cloud config/template layer §(b); base→tenant→site presets), /0013 (optics seam), /0018
(the intake reader + TerrainProfile the spectrum rides on). Meta: org/0004 (correctness ↔ management seam).
