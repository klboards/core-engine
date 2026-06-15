# core-engine

The **deterministic zmanim/calendar core engine** for klboards — the anchor of the
correctness side of the product (org/0004). A git submodule of the `klboards/klboards`
meta-repo at `engine/core-engine`.

## Responsibility

- The three deterministic domain functions **F1 (solar) / F2 (lunar) / F3 (Hebrew-calendar)**
  emitting **absolute instants** (see `docs/adr/0001`).
- **Parameter resolution** — the engine exposes knobs and resolves none; it **owns the
  parameter-vector schema** (`docs/adr/0002`).
- **Refraction model** and **on-device horizon-profile composition** (`docs/adr/0004`, `0006`).
- Consumed by device, cloud, and apps as an embeddable library.

## Seam: correctness — inherited constraints (do not violate)

- **Fully offline** through a multi-day chag + Shabbat; the network is never on the
  correctness path (`docs/adr/0005`).
- **Own oracle-validated engine** behind a **pluggable engine interface**; **no external
  engine on the correctness path** (`docs/adr/0003`, `0008`). `zmanim-core` is a build/test
  oracle + optional customer-selectable alternative — **not** the foundation.
- **Oracle-validated** (Wolfram / observatory) as a test fixture, never a runtime dependency.
- Civil time / DST is an **edge axiom outside the core** (`docs/adr/0007`).

## Engine posture (ADR-0008)

We **build our own** primary engine (owned, oracle-validated, shipped), exposed behind a
**pluggable engine interface**. `zmanim-core` plays two distinct roles — a build/test
validation oracle (never ships) and an optional customer-selectable alternative engine behind
the interface. Relinkability (org/0006) is satisfied by this architecture, not a license claim.
**Engine-selection, once ≥2 engines ship, is a calendar-correctness-bearing knob** (handed to
the parameter-vector schema, `docs/adr/0002`) — the no-drift guarantee holds only within one
engine choice.

## Stack

**TODO** — the engine's implementation **language is undecided** (klboards open decision #8) and
is **not** picked by ADR-0008. Design intent: a **freestanding / no-GC** engine (org/0006
runtime **Profile A**). Do not assume a language, framework, or build tool; mark unknowns TODO.

## Conventions

Org-wide conventions (stack-agnostic rule, ADR cross-ref prefix, seam vocabulary,
decision+rationale → ADR) are inherited from the `klboards-org` plugin and the meta-repo
root `CLAUDE.md` — not duplicated here. Architecture decisions live in `docs/adr/`
(migrated from the meta-repo; org/0004).
