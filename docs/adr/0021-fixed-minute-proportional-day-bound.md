# ADR core-domain/0021 — Fixed-minute proportional-day bound (literal-72-minute MGA day)

- **Status:** Accepted
- **Date:** 2026-06-19
- **Scope:** Adds **one** read-vocabulary primitive — a proportional-day [`Bound`] that is a *primitive
  bound shifted by fixed clock minutes* — so the **literal-72-fixed-minute Magen Avraham** proportional
  day (alos = netz−72 min, tzeis = shkia+72 min) becomes expressible. Touches `events.rs` (`PrimBound` +
  `Bound::OffsetMinutes` + `bound_jd`), `wire.rs` (`PrimBoundWire` + `BoundWire::OffsetMinutes` tag 3 +
  decoder), `docs/spec/read-spec.cddl`. The `zman_definitions` catalog stays second-order (the
  `sof-zman-shma-ma72-min` *entry* lives in halacha-model); **not** built here.

## Context

core-domain/0020 declared the first-order read vocabulary "total," but a real, common posek vector turned
out to be inexpressible: the **Magen Avraham sof-zman-shma computed from a literal 72 fixed clock minutes**
(KosherJava `getSofZmanShmaMGA72Minutes`, MyZmanim `ShemaMA72fix`). An external-source check (2026-06-19)
found this — not the 16.1°-degree approximation — is the **standard / prevalent** MGA basis for the
US-yeshivish (Lakewood) target base (the 72-min "4 mil × 18 min" method; GRA/MGA ≈36 min apart). The
degree variant (`sof-zman-shma-ma72-deg`, halacha-model migration 0013) only coincides with it at the
Israeli equinox.

The gap: a `Proportional{ fraction, start: Bound, end: Bound }` read needs its day-bounds to be `Bound`s,
and `Bound` admitted only `Netz | Shkia | Depression{angle}`. The MGA-72 day's bounds (netz−72, shkia+72
fixed minutes) are `FixedMinuteOffset` *reads*, not `Bound`s; and `FixedMinuteOffset.seasonal` cannot
anchor it either (its span bounds are also `Bound`s). Verified by reading `events.rs`. So the value forced
a core change — exactly the kind /0019/0020 aim to avoid, here genuinely unavoidable for a fixed-minute day.

## Decision

Add a **non-recursive** offset bound:

```rust
pub enum PrimBound { Netz, Shkia, Depression { angle_deg, dir } }   // the existing primitives
pub enum Bound {
    Netz, Shkia, Depression { angle_deg, dir },
    OffsetMinutes { base: PrimBound, offset_min: f64 },             // base shifted by fixed clock minutes
}
```

`Proportional` is unchanged — it already takes `Bound`. The literal-72-min MGA day is
`Proportional{ 0.25, OffsetMinutes{Netz,−72}, OffsetMinutes{Shkia,+72} }`. `bound_jd` resolves
`OffsetMinutes` as `prim_bound_jd(base) + offset_min/1440`; a does-not-occur base propagates (`?`).

**Why this shape (Design A, chosen over a new `ProportionalOffsetDay` ReadSpec variant):**
- **Composable** — an offset bound is reusable anywhere a `Bound` is (Proportional, FixedMinuteOffset
  base/span), matching the `proportional_day_bounds` knob and /0020's "compose from primitives."
- **`no_std`/no-alloc** — `PrimBound` is a fixed-size sub-enum, so `Bound` stays `Copy` with no `Box`/
  recursion (no offset-of-offset).
- **Wire-additive, no drift** — new `BoundWire` tag `3` only; tags `0/1/2` are byte-identical to /0020, so
  existing signed artifacts + the byte-stability snapshot are unaffected. New float path is `/1440` (a
  plain divide already used by `FixedMinuteOffset`) → native==wasm determinism is preserved by construction.

## Validation (oracle-first, every /0020 gate)

- **Cross-engine oracle (non-circular):** `tools/KosherDiff.java` extended to emit
  `getSofZmanShmaMGA72Minutes()`; `tests/cross_engine.rs` compares our `OffsetMinutes`-bounded read
  (based on `Depression{0.8333°}` = KJ's sea-level sunrise basis) — **24/24 within ±30 s, max residual
  ≈2.0 s** (the same few-second class as the sunrise/sunset differential). The LGPL jar is a build-tool,
  never vendored; regenerated in-agent via the JEP-330 single-file launch (no `javac` needed).
- **Wire:** `tests/wire.rs` round-trips the OffsetMinutes-bounded Proportional + a malformed prim-base
  direction → `Range` (no panic). **Fuzz:** `decode_read_spec` added to the 100k no-panic decoder fuzz.
- **Regression:** `shma_mga72min` added to the snapshot grid (blessed). **Determinism:** 659/659 exact
  native==wasm retained. **Mechanism:** `tests/read_vocab.rs` proves the identity (= alos72 + ¼·span) and
  that it precedes the GRA value. clippy `-D warnings` + fmt clean.

## Consequences

- The MGA-72-minutes value — the flagship's actual basis — is now expressible and oracle-validated. The
  shipped 16.1° variant remains valid (high-latitude / those who follow degrees).
- halacha-model mirrors `Bound::OffsetMinutes` + adds a `DayStream::MaFixedMinutes` and the
  `sof-zman-shma-ma72-min` catalog entry (migration 0014), routed through Beis Din before shipping.
- The read vocabulary is again total for the known posek set; the offset bound is the general fixed-minute
  day-bound primitive, not a one-off.

## Open / not addressed

- Offset bounds compose one level only (no offset-of-offset); no known posek vector needs more.
- Unchanged from /0020: COSE verification, one-vs-two channels, the `zman_definitions` catalog (management),
  Israel high-res DTM.
