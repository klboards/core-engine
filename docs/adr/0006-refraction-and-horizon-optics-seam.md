# ADR-0006 — Refraction & horizon optics: first-order physics in F1, conventions as parameters

- **Status:** Accepted (Seam 1)
- **Date:** 2026-06-15
- **Scope:** Stack-agnostic domain model. Placement of a seam, not a runtime choice.

## Context

Refraction and horizon optics determine *where the sun appears to cross the horizon* — which is what fixes the instant of netz (sunrise), shkia (sunset), and several derived zmanim. This is genuinely part of solar geometry, so it belongs to F1 (ADR-0001). But the specific optical choices are **convention-bearing**, not universal constants:

- apparent vs geometric sun position,
- the refraction coefficient / model used,
- sea-level (mishor) vs terrain horizon (ties to ADR-0004).

These are halachic / convention decisions, not physics facts. The seam between "physics" and "convention" must be placed deliberately, or it gets blurred in one of two harmful ways.

## Decision

**The physics stays in F1; the conventions are parameters.**

- F1 computes the apparent horizon-crossing using a refraction model and a horizon definition that it receives **as parameters** (per ADR-0002). F1 resolves none of them.
- The visible-sunrise horizon profile (ADR-0004) and the refraction model **compose inside F1** to yield the apparent horizon-crossing instant. The provisioning-time ray-trace and the device-time composition must use the **same** refraction model, or the shipped profile will not compose correctly.

## Rationale

- **Refraction is intrinsic to F1's reads, not a downstream offset.** It determines the horizon-crossing instant itself; the many zmanim defined by horizon crossings depend on it directly. Pushing it outside F1 as a "correction" would sever it from the curve it actually shapes.
- **The choices are policy, so they must be knobs.** Hard-coding a single refraction model or a sea-level horizon would collapse a parameter into a constant and make terrain-corrected visible sunrise (ADR-0004) and alternative opinions impossible to serve.
- Placing the seam explicitly prevents both failure modes: (a) baking a convention into the physics, and (b) exiling the physics into a correction layer.

## Alternatives considered

- **Hard-code one refraction model + sea-level horizon.** Rejected: cannot serve visible sunrise or multiple opinions; collapses a required parameter into a constant.
- **Treat refraction as a post-hoc correction outside F1.** Rejected: refraction defines the crossing instant; it is intrinsic to F1's `altitude(t)` reads, not a downstream adjustment.

## Consequences

- F1's input parameter vector must carry the **refraction model selector** and the **horizon definition** (sea-level vs terrain-profile reference).
- The provisioning pipeline (ADR-0004) and the device runtime share a single refraction model; this is a coupling to test explicitly (profile composed on-device must match a full recompute to tolerance).
- Refraction/horizon knobs are part of the parameter-vector schema owned by ADR-0002.

## Open questions

- **Refraction model choice** — explicitly flagged open question. Candidates include a standard atmospheric model (Bennett / Saemundsson-class), the Meeus/NOAA default already used by the baseline engines, or a halachically motivated fixed coefficient. To be decided as: a single default plus whether/which alternatives are exposed as parameters. Must be settled jointly with the parameter-vector schema (ADR-0002) and the provisioning pipeline (ADR-0004). **RESOLVED by ADR-0009** (2026-06-16): default `standard-atmospheric` (Bennett/Saemundsson-class); `meeus-noaa` and `halachic-fixed-coefficient` exposed as selectable `refraction.model` alternatives; provisioning ↔ runtime must share one model.

## Related

ADR-0001 (F1 owns the physics), ADR-0002 (conventions as knobs), ADR-0004 (horizon profile composes with refraction).
