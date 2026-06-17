# ADR core-domain/0020 — Read-spec vocabulary is complete (limb reference + fixed-minute-offset + read-spec decode)

- **Status:** Accepted
- **Date:** 2026-06-17
- **Scope:** Closes the **three** first-order mechanism gaps flagged by core-domain/0019 §4 — the
  netz-definition `limb_reference` axis, the `fixed-minute-offset` read, and CBOR-decoding of a single
  read-spec — so the first-order read vocabulary is **total**: any posek's parameter vector now composes
  from existing primitives, with no remaining expressiveness gap that would force a core change. The
  **mechanism-completeness** counterpart to /0019's **policy** ruling. Touches `params.rs` (limb knob),
  `optics.rs`/`events.rs` (limb-shifted target, incl. the terrain path), `events.rs` (the new read
  variant), `wire.rs` (`decode_read_spec` + the wire types). The `zman_definitions` catalog stays
  second-order (management side); **not** built here.

## Context

core-domain/0019 ruled the entire poskim spectrum **second-order** — halachic variation is
parameters/presets over the deterministic first-order core (ADR-0002), not core code paths — and showed
most axes need **zero** engine change. But /0019 §4 also isolated exactly **three genuine first-order
gaps**: things a posek could need that the engine's primitives could not yet *express*, which would
otherwise force a core edit. Until those close, the first-order surface is incomplete and "can the core
express any posek?" stays an open risk. core-domain/0018 §Open carried the same three under Phase 4b.
This ADR closes all three — and **only** those three — leaving the core lean.

## Decision

1. **Netz-definition axis `LimbReference {Upper, Center, Lower}`** (spec field `solar.limb_reference`).
   The beginning-vs-end-of-ascent dispute: **upper limb** = the sun's first appearance (majority; e.g.
   Biur Halacha 58); **lower limb** = the whole disc has risen / "end of ascent" (Ish Matzliach; Yalkut
   Yosef 89:3); **center** = disc centre. Realized as a `±semidiameter` shift on the horizon target —
   `target = −(limb_sign·semidiameter + dip)`, `limb_sign ∈ {+1 Upper, 0 Center, −1 Lower}`. It is
   convention-free **mechanism**: the core resolves no shita. **Default `Upper` reproduces the pre-0020
   behaviour byte-for-byte** (the regression gate). Lives on the `Optics` knob struct (`params.rs`),
   threaded through the horizon target in `optics.rs`/`events.rs`, **including the terrain-skyline path**
   (`terrain_horizon_crossing`).
2. **`ReadSpec::FixedMinuteOffset { base, offset_min, seasonal }`** — completes the spec §1.C read-spec
   union (previously `DepressionAngle`, `HorizonCrossing`, `ExtremumMidpoint`, `Proportional`; this is
   the missing fifth). `seasonal = None` ⇒ literal fixed clock minutes (`base ± offset_min/1440` of a
   day); `seasonal = Some((start, end))` ⇒ **zmaniyos** (seasonal) minutes scaled by that day's sha'ah
   zmanit (`offset_min/60 · span/12`, `span` = the proportional-day length between the two bounds,
   **reusing the existing `Proportional` machinery**). Expresses fixed-minute shitot: Magen Avraham alot
   = netz − 72, Rabbeinu Tam tzeit = shkia + 72, and their seasonal variants.
3. **Read-spec is decodable from CBOR** — `wire::decode_read_spec(bytes) -> Result<ReadSpec, DecodeError>`
   plus integer-keyed `ReadSpecWire` / `BoundWire` minicbor types covering all five variants, mapping to
   `events::ReadSpec`. **Scope cut, consistent with /0019:** the engine decodes **one** read (the
   mechanism); the **`zman_definitions` map/catalog** — which named zmanim a tenant wants, ids→reads —
   stays **second-order** in the management/provisioning layer, which iterates the catalog and calls the
   engine per read. Deliberately narrower than the /0018 Phase-4b flag implied, and exactly what
   permanently closes the *core* gap without bloating the core.
4. **Out of scope, flagged not faked:** `solar_position_reference` (apparent-everywhere vs
   geometric-everywhere) stays a **baked** behaviour — the wire layer returns `Unimplemented` for
   non-default values (the /0018 discipline), a separate, rarely-needed axis. The zman catalog,
   `obligation_sense`, and rounding remain management concerns.

## Rationale

Completing the read vocabulary is the dual of /0019: /0019 proved the *policy* lives second-order; this
proves the *mechanism* is sufficient to host it. With these three additions the engine spans every named
position by substitution, so no future posek forces a core branch (ADR-0002/0008) — the value of one
oracle-validated, bit-reproducible, offline engine is preserved while the core does **not** grow beyond
mechanism. Keeping the catalog out (decode one read, not a map) holds the seam (org/0004) and matches
/0009's preset layering, which already iterates per read.

## Consequences

- **First-order surface closed.** Risk on "can the core express any posek?" is retired; remaining risk is
  integration/systems — the provisioning **writer** (Phase B), device-runtime, COSE verification, the
  management repos.
- **Regression-safe.** `LimbReference` default `Upper` + the additive read variant leave golden 66/66,
  F2 11/11, the existing 536 FP rows, and the regression snapshot **byte-unchanged** (golden + regression
  pass with no re-bless); the new limb/offset paths are deliberately *added*, not silently mutated.
- **Determinism preserved.** The float-free wire stays CDE (/0018 sub-items 1 & 2); the new float paths
  are `libm`-only and proven exact native==wasm — the FP grid grows **536 → 659** (the /0010
  one-core-no-drift gate, now covering lower-limb netz + fixed/seasonal minute offsets), wire 5 → 7.

## Open / flagged

Whether a `seasonal` offset should reference the read's own explicit day bounds vs a global day
definition — **resolved here** by using the read's explicit `(start, end)` bounds, matching the existing
`Proportional` pattern. `solar_position_reference` (apparent/geometric-everywhere) remains a baked axis,
flagged `Unimplemented` not faked. Device-side **COSE verification** remains open, coupled to org/0006
§7's root-of-trust (HW secure-boot vs pinned-key). The `zman_definitions` catalog stays second-order
(management side). Israel high-res DTM (/0004) is unaffected — the sole open top-level hard-TODO.

## Related

core-domain/0019 (the policy ruling whose three §4 gaps this closes; the mechanism-completeness dual),
/0018 (CBOR intake reader + the §Open Phase-4b items this resolves, narrowed), /0013 (optics seam — the
apparent-limb horizon target the `±semidiameter` shift extends; baked `solar_position_reference`),
/0002 (parameters over outputs, no denomination engines), /0009 (parameter-vector schema; read-spec
union; base→tenant→site presets that iterate per read), /0011 (CBOR/CDDL/COSE wire), /0010 (no_std/FP
determinism gate), /0017 (no-panic decoder invariant, extended to `decode_read_spec`). Meta: org/0004
(correctness ↔ management seam), org/0006 (§7 root-of-trust, COSE).
