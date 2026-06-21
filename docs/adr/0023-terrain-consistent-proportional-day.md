# core-domain/0023 — Terrain-consistent proportional day (`horizon_mode` governs sha'os-zmaniyos bounds)

Status: accepted · Date: 2026-06-21 · Extends: /0004 (terrain), /0018 (TerrainProfile reader),
/0019 (poskim spectrum is second-order), /0017 §Open (the flagged horizon_mode↔proportional gap)

## Context

The visible-sunrise moat (/0004, /0018) was reaching only the **displayed event** (`hanetz`/`shkia` via
`terrain_horizon_crossing`), not the **deadlines derived from it**. A live cross-check of the Jerusalem
board (`jerusalem-gra-visible`, `horizon_mode = TerrainProfile`) against MyZmanim made this concrete:

- `hanetz` displayed = **05:51** (terrain, over Har HaZeisim), but `sof-zman-shma-gra` was computed from
  the **scalar elevation-dip (Visible)** netz ≈ 05:27 → **09:04**, while the **Mishor** basis gives 09:07
  (MyZmanim) and the **terrain** basis would give a later value. The board showed a terrain *event* and a
  Visible-basis *deadline* — an internally inconsistent state that is **nobody's halachic position**.

**Root cause (engine):** the proportional chain `Proportional{start,end}` → `proportional_span_days` →
`bound_jd` → `prim_bound_jd` → `read_jd(HorizonCrossing)` resolved Netz/Shkia via the scalar
`find_crossing`; **no `HorizonProfile` was threaded**, so a `TerrainProfile` mode could not reach a
proportional bound. The cloud/device dispatchers called `terrain_horizon_crossing` *only* for direct
`HorizonCrossing` reads.

**Halachic finding (Sefaria / MyZmanim, recorded as the basis):** SA O.C. 58:1 + MB 58:4 define the GRA
day as `הנץ החמה → שקיעה` reckoned to `רביע היום`, but the sources say only **"הנץ החמה"** — they do
**not** disambiguate *mishor* (sea-level) vs *nireh* (visible). A targeted search for that distinction
returns **0 classical hits**: it is a **modern computational question**, arising only once terrain can be
computed. A literal read favours *visible*; standard luchos / KosherJava / MyZmanim compute proportional
from *mishor*. ⇒ A genuine unresolved machlokes — it must be a **parameter applied consistently**, not a
hard-coded choice (/0019: variation is second-order; the core resolves no posek dispute).

## Decision

The existing **`horizon_mode` knob (Mishor / Visible / TerrainProfile)** is the single basis parameter and
now governs netz/shkia **consistently** — for the displayed event AND every read derived from it (the
`Proportional` day-bounds and the seasonal `FixedMinuteOffset` span). The same netz reckons the event and
the deadline. **No new knob and no wire change** (`horizon_mode` is already in the parameter-vector; the
profile already ships). The basis is a per-preset choice; a Beis-Din run may ratify the default per preset.

### Engine (`src/events.rs`)
- `read_jd_h(... horizon: Option<&HorizonProfile>)` is the horizon-aware core; `read_jd` is the
  `None` wrapper. The **one behavioural change** is the `HorizonCrossing` arm: when
  `horizon_mode == TerrainProfile` **and** a profile is bound, it resolves via the new JD-returning core
  `terrain_crossing_jd` (extracted byte-for-byte from `terrain_horizon_crossing` — no lossy
  JD→instant→JD round-trip). `horizon` is threaded through `bound_jd`/`prim_bound_jd`/
  `proportional_span_days_h`. New public entry `read_instant_with_horizon`.
- **Determinism discipline:** the byte-frozen scalar `find_crossing` is untouched; Mishor/Visible and any
  `None` path are **bit-identical** → the oracle/golden/regression suite and **659/659 native==wasm** stay
  green unchanged. The new terrain-proportional path is deterministic by composition (identical float ops
  to the already-validated terrain crossing + the already-validated proportional arithmetic).

### Dispatchers (compose==recompute, org/0006)
control-plane `api.rs::compute_luach` and device-runtime `lib.rs::compute` both call
`read_instant_with_horizon`, passing the bound profile for **all** reads — an identical change, so
cloud == device.

## Consequences

A `TerrainProfile` preset is now internally consistent end-to-end (the moat reaches the deadlines).
The mishor-vs-visible-vs-terrain basis is an explicit, documented, per-preset parameter rather than an
accident of read-spec routing. Verified by `tests/terrain.rs::terrain_proportional_uses_terrain_bounds`
(engine), the device acceptance suite (cloud==device), and `control-plane/tests/terrain_board.rs` (the
real Jerusalem profile: GRA sof-zman-shma sits ¼ into the terrain day and differs from sea-level).
**Ratified (documented, 2026-06-21):** a Beis-Din psak record now ratifies the basis as a *per-preset
parameter* — `verdict=disputed, tier=convention, correctness=contested` (a genuine machlokes: SA O.C.
58:1 + MB 58:4 say only "הנץ החמה", with **0 classical hits** disambiguating mishor vs nireh/visible).
Recorded honestly (source research via knowledge MCPs, not a swarm run) at
`knowledge/halacha-model/docs/model/verification-log/beis-din-terrain-proportional-20260621.ndjson`;
`psak-log audit` clean. Standing per-preset defaults: standard presets keep their existing Mishor/Visible
basis; `jerusalem-gra-visible` = consistent terrain. The Israel high-res DTM (/0004) is unchanged.
