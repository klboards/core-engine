# ADR-0002 — Variation is parameters over first-order outputs, not code paths

- **Status:** Accepted — ratifies the settled "variation = parameters, not code" principle. Documented for reference; not re-derived.
- **Date:** 2026-06-15
- **Scope:** Stack-agnostic domain model.

## Context

Communities differ by **custom, stream, and stringency**, not by physics or arithmetic. The math is identical across Ashkenazi, Sephardi, Chabad, and every nusach; what differs is which opinion (shita) is applied and how results are rounded, defined, and labeled. The market research is explicit that the calculation layer is a commodity and the differentiation is elsewhere — so encoding community variety as code would multiply maintenance cost for zero strategic gain.

## Decision

**Every stream, custom, and stringency is a parameter over first-order outputs.** The first-order functions (ADR-0001) **expose** the knobs and **resolve none** of them. Resolution policy lives in a configuration layer outside the core.

The knob set includes at least:

- depression angle / fixed offset for a given zman;
- **GRA vs Magen Avraham** — i.e. which two crossings bound the proportional ("seasonal hour") day;
- day definition — `netz→shkia` vs `alot→tzeit`;
- **elevation / terrain horizon ON or OFF** (mishor vs visible horizon; composes with ADR-0004/0006);
- nusach calendar customs;
- rounding direction / stringency (lbehakel vs lehachmir);
- candle-lighting offset;
- label / script / language.

## Rationale

- **No combinatorial explosion of "denomination engines."** A single deterministic core serves every community by parameterization; variety lives in data, which is cheap to add, test, and audit.
- **Keeps the core small and the surface auditable.** A posek or community admin can inspect a parameter vector; they cannot inspect a code branch.
- **Data-driven testing.** Each parameter combination becomes a fixture that can be checked against the neutral oracle (ADR-0003) without touching code.
- Aligns with the market reality: time math is a commodity input, so investing in a configurable core (rather than bespoke per-stream logic) concentrates effort where the moat actually is — display, management, and visible sunrise.

## Alternatives considered

- **Per-denomination / per-nusach engines or code branches.** Rejected: duplication, drift between branches, an untestable matrix, and direct contradiction of the "no denomination engines" rule.
- **Hard-code a default opinion in the core.** Rejected: the core must resolve none. A default is itself a policy choice and belongs to the configuration/edge layer, where it can be overridden per site without a code change.

## Consequences

- The core advertises a complete knob set; an unset knob is a configuration error, not a silent default inside F1/F2/F3.
- A **parameter-vector schema** is required and is a first-class artifact (see Open questions).
- A configuration/resolution layer is needed to bind parameter vectors to sites/communities; this layer is where a shipped "default profile" may live.
- Refraction and horizon conventions are parameters of this same kind, even though the physics is first-order (ADR-0006).

## Open questions

- **The exact parameter-vector schema** — names, types, units, defaults, and which knobs are required vs optional. This is one of the three explicitly flagged open questions for the project and blocks F1/F2/F3 interface finalization. Owned here. **RESOLVED by ADR-0009** (2026-06-16): schema + F1/F2/F3 interface contract ratified; living contract in `docs/spec/parameter-vector-and-interface-contract.md`.

## Related

ADR-0001 (the functions that expose the knobs), ADR-0004 (terrain ON/OFF), ADR-0006 (refraction conventions as parameters).
