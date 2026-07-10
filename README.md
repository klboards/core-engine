# core-engine

Deterministic **zmanim / calendar core engine** for klboards — the correctness anchor of the product.

> **Status: early — architecture & decisions only, no implementation code yet.** The engine's
> implementation language is undecided (klboards open decision #8); there is no build yet, by design.

## What it is

The three deterministic domain functions — **F1 (solar) / F2 (lunar) / F3 (Hebrew-calendar)** —
emitting absolute instants, plus parameter resolution, the refraction model, and on-device
horizon-profile composition. Consumed by the device, cloud, and apps as an embeddable library.
It sits wholly on the **correctness** side of the klboards seam: compute correct, correctly-labeled
instants **fully offline**.

## Engine posture (ADR 0008)

We build our **own** oracle-validated engine behind a **pluggable engine interface**.
`zmanim-core` serves as a build/test validation oracle and an optional customer-selectable
alternative — **not** the foundation.

## Architecture decisions

All decisions live in [`docs/adr/`](docs/adr/) (ADRs 0001–0008; see the
[index](docs/adr/README.md)). They are **binding**.

## Part of klboards

A git submodule of the [`ybenjwho/klboards-klboards`](https://github.com/ybenjwho/klboards-klboards) meta-repo
at `engine/core-engine`. Org-wide conventions are inherited from the meta-repo and the
`klboards-org` plugin; agent context is in [`CLAUDE.md`](CLAUDE.md).

## License

TBD — see ADR 0003 / 0008 (the engine is owned; license diligence on any shipped *alternative*
engine is tracked there).
