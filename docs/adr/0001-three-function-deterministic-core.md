# ADR-0001 — Three deterministic domain functions with absolute-instant outputs

- **Status:** Accepted — ratifies the already-settled first-order model. Documented here so downstream ADRs can reference it; **not re-derived**.
- **Date:** 2026-06-15
- **Scope:** Stack-agnostic domain model. No language, framework, runtime, hardware, or service is assumed.

## Context

klboards computes Jewish liturgical time and calendar data for a synagogue display. The only natural givens are a location and an instant: `(latitude φ, longitude λ, elevation h)` plus a moment `t`. Elevation is a **real axis**, not a cosmetic correction — sea-level versus terrain horizon changes the answer (this is what makes the visible-sunrise niche real; see ADR-0004).

The risk this ADR closes off is the "one big zmanim engine" shape, which hides couplings, resists oracle validation, and cannot cleanly expose the parameter knobs that community variation requires (ADR-0002).

## Decision

Model the domain as **exactly three pure, deterministic functions**, each emitting **absolute instants** (UTC instants or sun-angle events), timezone-free:

- **F1 — Solar geometry.** `(φ, λ, h, t) →` sun altitude/azimuth `→` one continuous `altitude(t)` curve. Every solar zman is a *read* off that curve: a depression angle, a horizon crossing, an extremum, or a fixed/proportional split between two crossings. Refraction and horizon optics live inside F1 (see ADR-0006).
- **F2 — Lunar geometry.** Moon position / phase / visibility. Backs Kiddush Levana. **Molad is not here** — molad is calendar arithmetic, not observed moon, and belongs to F3.
- **F3 — Hebrew-calendar arithmetic.** `t →` Hebrew date plus the full fixed structure: Metonic cycle, postponements (dechiyot), molad, parsha (including Israel/diaspora divergence), festivals/fasts/Rosh Chodesh, Omer, learning cycles, tekufa / tal-umatar, and yahrzeit anniversaries with Adar I/II handling.

The couplings are **explicit, typed dependencies**, never hidden inside a function:

- F3's day rolls at a **sun-defined instant** (an F1 read), not civil midnight.
- F3 day-type selects **which** F1 reads/offsets matter.
- tal-umatar is a tekufa (F1-class) event **gated by** an F3 date and the Israel/diaspora flag.
- Kiddush Levana is an F3 **window** confirmed by F2 visibility.

## Rationale

- **Determinism + absolute outputs make the core testable against a neutral oracle** (ADR-0003) and portable across locations and time zones. Instants are comparable to an observatory or Wolfram to the second; wall-clock strings are not.
- **Elevation as a first-class input** is required for terrain-corrected horizons; demoting it to a post-hoc correction would structurally block the one defensible differentiator the market research identifies.
- **Separating molad (F3) from observed moon (F2)** prevents conflating calendar structure with physics. The two are different in kind and fail differently.
- The three-function shape is what lets *all* variation move into parameters rather than code (ADR-0002): a single core, no per-community engines.

## Alternatives considered

- **One monolithic zmanim engine.** Rejected: hides the F3↔F1 and F3↔F2 couplings, is not independently testable, and cannot expose parameter knobs without internal branching.
- **Two functions (fold lunar into calendar).** Rejected: Kiddush Levana needs *observed-moon visibility*, not arithmetic molad. Folding them hides a genuine physics dependency and would make the moon-visibility logic untestable in isolation.
- **Civil-time-keyed days.** Rejected: the Jewish day rolls at sun-defined instants. Civil time is an edge concern, not a domain input (ADR-0007).

## Consequences

- Each function is independently testable and independently validatable against the oracle.
- Couplings must be implemented as **declared inputs/outputs between functions**, not as reach-ins. F3 consumes specific F1 reads; the dependency direction is fixed.
- All outputs are instants. Every label, format, script, language, and wall-clock rendering is strictly downstream (ADR-0007).
- The functions take a **parameter vector** in addition to `(φ, λ, h, t)`; they resolve none of those parameters themselves (ADR-0002).

## Open questions

- The exact **parameter-vector schema** that F1/F2/F3 accept — flagged and owned by ADR-0002.

## Related

ADR-0002 (parameters), ADR-0003 (engine sourcing), ADR-0006 (refraction seam in F1), ADR-0007 (civil time outside the core).
