# ADR 0008 — Engine posture: own primary engine behind a pluggable interface; zmanim-core as oracle + optional alternative

- **Status:** Accepted
- **Date:** 2026-06-15
- **Scope:** Engine sourcing/posture for the core-engine repo. Refines ADR-0003 (engine sourcing). **Does NOT pick the implementation language** — that is the still-open #8 language decision, made separately. Cross-references the meta-repo edge envelope `org/0006`.

## Context

A decision thread initially proposed adopting `zmanim-core` (a Rust KosherJava-conformance port) *as* the engine. That is **not** our posture. Our correctness foundation must be a thing we own, ship, and stand behind — not a dependency on an external derivative-licensed port. This ADR records the corrected posture and the roles external engines may play.

## Decision

**1. Our own engine is the primary (#8 primary).** The F1/F2/F3 implementation we ship and stand behind is **built and owned by us**, oracle-validated (ADR-0003). Its **implementation language is TODO** (the open #8 decision) — *not* selected here.

**2. Pluggable engine interface.** The core exposes a **pluggable engine interface** so the correctness engine behind it is a selectable implementation. Our owned engine is the default/primary; alternatives can sit behind the same interface.

**3. `zmanim-core` has two distinct roles — never the foundation:**
- **(a) Validation oracle (build/test only).** `zmanim-core` is adopted as an *additional* build/test cross-check oracle alongside the existing neutral oracles (Wolfram / observatory; ADR-0003). **It never ships to the device.** License/F3/bindings concerns are minimal in this role — internal validation only. Recorded in ADR-0003's testing/validation section.
- **(b) Optional customer-selectable alternative engine.** Offered for convenience behind the pluggable interface. This is the "shipped-switch path" and carries the requirements below.

## Consequences

### Relinkability is satisfied by architecture, not a license claim
`org/0006`'s LGPL-relinkability requirement was protection against being locked to a derivative-licensed foundation. The **pluggable-engine design with our owned engine as primary is that protection, made concrete** — we depend on **no external engine for correctness**. The MIT-vs-LGPL provenance question on `zmanim-core` therefore attaches **only to the shipped-switch path**, and is **downgraded from a foundation blocker to a counsel/revisit item**. **Revisit trigger:** *before `zmanim-core` ships as a customer-selectable option.*

### Two roles, opposite requirements — do not let the benign benchmark framing license the shipped path
- As an **oracle** (build/test): never ships; minimal concern; pure upside.
- As a **shipped alternative**: it runs in the product, so **"which engine" becomes a correctness-bearing selection, not a cosmetic preference.** Two engines = two correctness surfaces. The "device and PWA share one core so they can't drift" guarantee holds **only within a single engine choice.**
  - **Handoff:** *engine-selection* is a **calendar-correctness-bearing knob** → handed to the parameter-vector schema thread (`core-domain/0002`), in the **same class as the day-boundary parameters**, NOT the cosmetic-preference class.

### F3 parity is scoped to the alternative path, not our correctness
Because our engine is the validated primary, `zmanim-core`'s incompleteness on Israel/diaspora and Adar I/II is a **feature-parity limitation on the optional switch**, not a wrong-date risk in what we ship. Record: the customer switch **may be unavailable or flagged-incomplete** for communities needing those features until parity is confirmed. **Convergence note:** Israel/diaspora and Adar I/II disambiguation are exactly the knobs the schema defines and **our engine must honor** — that is **our F3 completeness bar, measured against the oracle.**

### Bindings: buildable, not confirmed-shipped
Do not claim shipped Python/JS bindings. The no-drift property comes from **a single core with single-source-generated bindings that are buildable** (e.g. pyo3/maturin, wasm-bindgen if the engine is a Rust-class language), **not** from shipped bindings. Recorded as *buildable, not confirmed-shipped*.

### org/0006 memory + serialization
- **Runtime term → Profile A** *if our engine lands freestanding/no-GC* (the design intent), which lets `org/0006` size the emulator. **Only the runtime term closes** — the parameter-vector term still waits on the schema (`core-domain/0002`) and the display-buffer on the #3 display profile. **Memory is NOT "closed."**
- **Profile A scope:** closes for the **core engine** if freestanding; it holds **device-wide only if `device-runtime`'s app layer** (state machine, OTA, display glue) **is also freestanding.** A managed-runtime host for `device-runtime` would re-add a B-term. Stated so the emulator sizes the right thing.
- **Serialization stays OPEN** as a deliberate **cross-repo encoding sub-decision**: the horizon-profile + parameter-vector format is a **shared contract** that provisioning writes and apps read. A language-only format (e.g. Rust `postcard`) is frictionless on-device but thin cross-language; CBOR / protobuf / flatbuffers may be right. Do **not** let an in-ecosystem default pre-resolve it.

## Open items (record now)

| Item | Path | Status |
|---|---|---|
| Our engine's implementation **language** | #8 | OPEN — separate gated decision; not picked here |
| `zmanim-core` shipped-switch license (MIT vs LGPL provenance) | shipped path | Counsel/revisit — trigger: before it ships as a customer option |
| `zmanim-core` F3 parity (Israel/diaspora, Adar I/II) | shipped path | Feature-parity limit; switch flagged-incomplete until confirmed |
| `engine-selection` as a correctness-bearing knob | core-domain/0002 | Handed to the schema thread |
| Parameter-vector + horizon-profile **serialization** | cross-repo contract | OPEN — deliberate sub-decision (cross-language) |
| Device-wide Profile A (app layer freestanding?) | org/0006 / device-runtime | Intent A; confirm at device-runtime |

## Related

ADR-0003 (engine sourcing — this refines it; oracle list extended with `zmanim-core`), ADR-0001 (F1/F2/F3), ADR-0002 (parameter-vector schema — owns engine-selection knob), ADR-0005 (offline keystone), ADR-0007 (civil-time edge axiom). Meta: `org/0006` (edge-target envelope — runtime Profile A term), #8 (engine language, open).
