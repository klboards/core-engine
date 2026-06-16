# core-engine — Architecture Decision Records

> **Status: BINDING product-architecture decisions** for the klboards zmanim/calendar core
> engine. Migrated here from the `klboards/klboards` meta-repo (org/0004 fulfilled); this
> repo is now their home. Org/infra ADRs live in the meta-repo's `docs/adr/`. Keep the
> internal `0001–0007` numbering and cross-references intact.

Stack-agnostic ADRs for the klboards core domain. **Hard rule preserved throughout:** no language, framework, runtime, service, hardware, or workflow is assumed; unknowns are TODO, never a plausible default. Every ADR is `decision · rationale · alternatives · consequences`, with open questions marked explicitly.

ADR-0001 and ADR-0002 **ratify the already-settled first-order model** so the determination/seam ADRs can reference it; they are documented, not re-derived.

## Index

| ADR | Title | Maps to |
|---|---|---|
| [0001](0001-three-function-deterministic-core.md) | Three deterministic domain functions with absolute-instant outputs | Settled model |
| [0002](0002-variation-as-parameters.md) | Variation is parameters over first-order outputs, not code paths | Settled principle |
| [0003](0003-engine-sourcing-embeddable-open-offline.md) | Engine sourcing: embeddable open-license library, on-device, oracle-validated | Determination 1 |
| [0004](0004-visible-sunrise-terrain-horizon-profiles.md) | Visible sunrise: terrain-corrected horizon profiles computed at provisioning | Determination 2 |
| [0005](0005-offline-correctness-network-independent.md) | Offline correctness is network-independent | Determination 3 |
| [0006](0006-refraction-and-horizon-optics-seam.md) | Refraction & horizon optics: physics in F1, conventions as parameters | Seam 1 |
| [0007](0007-civil-time-dst-edge-axiom.md) | Civil time / DST: edge axiom outside the core, versioned | Seam 2 |
| [0008](0008-engine-posture-and-sourcing.md) | Engine posture: own primary behind a pluggable interface; zmanim-core as oracle + optional alternative | Posture (refines 0003) |
| [0009](0009-parameter-vector-schema-and-interface-contract.md) | Parameter-vector schema + F1/F2/F3 interface contract | Resolves 0002 schema; closes 0006 refraction default |

## Ratified decisions handoff (decision + rationale, for Claude Code)

- **ADR-0001** — *Decision:* model the domain as exactly three pure deterministic functions (F1 solar, F2 lunar, F3 Hebrew-calendar) emitting absolute instants, with `(φ, λ, h, t)` as the only natural inputs and couplings declared explicitly. *Rationale:* determinism + absolute outputs make the core oracle-testable, timezone-free, and offline-capable; elevation must be first-class for terrain horizons; molad (F3) is arithmetic and separate from observed moon (F2).
- **ADR-0002** — *Decision:* every stream/custom/stringency is a parameter over first-order outputs; the core exposes knobs and resolves none. *Rationale:* avoids a combinatorial explosion of "denomination engines," keeps the core small and auditable, and makes variation data-driven and testable — matching a market where the math is commodity.
- **ADR-0003** — *Decision:* compute on-device with an embeddable open-license library (KosherJava LGPL / Hebcal CC-BY *code*), fully offline through a multi-day chag+Shabbat, validated against a neutral oracle (Wolfram / observatory), depending on no competitor engine. *Rationale:* the engine is a commodity, so capture it for free and spend effort on the moat; depending on B.A. Timing/Chazon Shamayim (an engine-maker that also sells a competing board) would be a network and strategic liability; validate non-circularly.
- **ADR-0004** — *Decision:* deliver terrain-corrected visible sunrise via per-site `(azimuth → horizon-angle)` profiles computed at provisioning from Copernicus GLO-30 baseline + 1 m lidar overrides (USGS 3DEP / IGN RGE ALTI; Israel TODO; FABDEM commercial-only), composed offline on-device with F1 + refraction. *Rationale:* this is the one scarce, defensible capability; heavy DEM work belongs at provisioning, leaving the device offline-correct and cheap; build it rather than depend on ChaiTables/Sky-View.
- **ADR-0005** — *Decision:* nothing about correctness may depend on the network; the network is for management/content/ruleset updates only, and the device stays correct with zero connectivity across a full multi-day chag+Shabbat. *Rationale:* the board matters most when the network is down or unwanted; correctness must fail-safe while freshness/management may degrade gracefully; staleness is handled by versioning with a bounded (labeling-only) failure mode.
- **ADR-0006** — *Decision:* refraction and horizon optics are first-order physics inside F1, but the conventions (apparent vs geometric, refraction coefficient, sea-level vs terrain) are parameters; provisioning and runtime share one refraction model. *Rationale:* refraction defines the horizon-crossing instant so it must live in F1, but the choices are policy so they must be knobs — preventing both baking a convention into physics and exiling physics into a correction layer.
- **ADR-0007** — *Decision:* civil time/DST live outside the core; the core stores absolute instants, and wall-clock is a boundary-only label from a versioned IANA/DST ruleset that is never trusted as fixed. *Rationale:* civil time is a mutable, decree-driven labeling convention, not a domain function; keeping it out preserves determinism and bounds the failure mode — a stale ruleset can mis-label but never make a zman wrong.
- **ADR-0008** — *Decision:* build and own the primary F1/F2/F3 engine behind a pluggable engine interface; `zmanim-core` serves only as a build/test oracle and an optional customer-selectable alternative, never the foundation; engine-selection is a correctness-bearing knob handed to ADR-0002. *Rationale:* the correctness foundation must be ours, not a derivative-licensed dependency; relinkability is satisfied by architecture, not a license claim; two engines mean two correctness surfaces, so the no-drift guarantee holds only within one engine choice.
- **ADR-0009** — *Decision:* ratify the parameter-vector schema + F1/F2/F3 interface contract (`docs/spec/parameter-vector-and-interface-contract.md`): zmanim as a typed read-spec union (data, not code), terrain/refraction/engine-selection as knobs, the four 0001 couplings as typed dependencies; default `refraction.model = standard-atmospheric` with `meeus-noaa`/`halachic-fixed-coefficient` as alternatives; required-vs-optional cut and `base → tenant → site` preset precedence fixed. *Rationale:* a single data-parameterized core stays oracle-testable and auditable (0002/0003); the standard-atmospheric model composes with terrain horizon profiles where a fixed coefficient degrades; layering and defaults stay outside the core so it resolves none. Resolves 0002's schema question and closes 0006's refraction default; serialization (0008) and Israel DTM (0004) stay open.

## Explicitly flagged open questions

| Open question | Owned by | Note |
|---|---|---|
| ~~Exact **parameter-vector schema**~~ | ADR-0002 | **RESOLVED by ADR-0009** — schema + F1/F2/F3 interface contract ratified; living contract in `docs/spec/parameter-vector-and-interface-contract.md`. |
| **Israel high-resolution DTM source** | ADR-0004 | **OPEN** — until resolved, Israel sites fall back to Copernicus GLO-30 (30 m), a precision gap in the market that values visible sunrise most. |
| ~~**Refraction model choice**~~ | ADR-0006 | **RESOLVED by ADR-0009** — default `standard-atmospheric` (Bennett/Saemundsson-class); `meeus-noaa`/`halachic-fixed-coefficient` exposed as selectable alternatives; provisioning ↔ runtime share one model. |

> Of the three top-level flags, two are now resolved (ADR-0009); **Israel high-resolution DTM
> source** remains the sole open top-level question.

Secondary unknowns recorded in-ADR (not among the three top-level flags): per-platform port/library and oracle source/tolerance (0003); certified offline-autonomy window `N` (0005); edge staleness-signaling policy and tzdata update cadence (0005/0007).
