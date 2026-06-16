# ADR core-domain/0016 — The F1/F2/F3 couplings: day-roll, day-type, full Kiddush Levana, tal-u-matar

- **Status:** Accepted
- **Date:** 2026-06-17
- **Scope:** The four cross-function couplings declared by ADR core-domain/0001 and deferred by /0014
  (F3↔F1 day-roll) and /0015 (full Kiddush Levana). New `src/couplings.rs` (the only module importing
  both F1/F2 and F3) and `src/tekufa.rs` (arithmetic seasons); `DayKind`/`DayClass`/`classify_day` +
  a `chalakim_to_instant` extraction in `src/calendar.rs`; `Realm`/`TalUmatarBasis`/`TekufaMethod`
  knobs in `src/params.rs`; a `sun_effective_alt_deg` helper in `src/events.rs`. Stack: Rust/`no_std`
  (/0010). "Scope A" — couplings only; Omer / learning cycles / parsha strings / fixed-clock-minute
  tzeit read-specs / the cloud template layer are **out of scope**.

## Context

F1 (66/66), F2 (11/11) and F3 (27/27) were each oracle-green but uncoupled. The product's displayed
values are the *compositions*: which Hebrew day it is (rolls at sunset, not midnight), what kind of
day it is (gating which reads matter and diaspora Yom Tov Sheni), when Kiddush Levana is actually
sayable (window ∩ moon-up ∩ night), and when tal-u-matar begins (realm-selected). The halachic
content was researched and cross-verified (MyZmanim, WikiYeshiva, Wolfram, Hebcal, Chabad, the
Rabbinical Assembly, KosherJava, Mi Yodeya, OU). One prior framing was corrected: **tal-u-matar is
not F1 astronomy.**

## Decision

1. **Day-roll (coupling #1)** — `couplings::hebrew_date_at_instant(t, site, boundary, optics) ->
   DayRoll`. The Hebrew date labelling civil day `D` covers `[boundary(D−1), boundary(D))`; the result
   is `hebrew_from_fixed(D)` for the least civil day whose boundary instant is strictly after `t`. The
   **boundary read is a caller-supplied `ReadSpec`** (default shkia = `HorizonCrossing{Setting}`); the
   shkia-vs-tzeit **bein-hashmashot safek is not resolved by the core**. The civil candidate is seeded
   from `t`'s UTC date (tz-free, /0007) but the decision is made by comparing `t` against each day's
   boundary *instant* over a ±1-day window, so far-longitude instants near 00:00 UT do not go
   off-by-one.
2. **Day-type (coupling #2)** — `calendar::classify_day(date, realm) -> DayClass` (co-holding flags:
   `shabbat`, `yom_tov`, `chol_hamoed`, `erev`, `rosh_chodesh`, `fast_day`; plus a `DayKind::primary`
   convenience). Pure F3 integer arithmetic. `Realm` gates **Yom Tov Sheni** (the diaspora second
   festival day and the shifted Chol HaMoed boundary) and is the *only* realm-conditional branch — a
   parameter, not a per-stream code path (/0002). Fasts carry the standard Shabbat-deferral (nidcheh:
   forward to Sunday, except Ta'anit Esther back to Thursday). "Which reads apply" stays at the edge.
3. **Full Kiddush Levana (coupling #3)** — `couplings::kiddush_levana_sayable_at(...) -> bool` =
   molad window ∩ F2 `moon_visible` ∩ F1 night (Sun below a **caller-supplied** tzeit depression, via
   the new `events::sun_effective_alt_deg`). `kiddush_levana_interval_on_night(...)` returns the
   window∩night bracket (moon-up confirmed per-instant by `_sayable_at`, since the moon can rise
   mid-night). **Day-type exclusions (no KL on Shabbat / Yom Tov, motzaei-Shabbat-only customs,
   post-Tisha-b'Av) are EDGE policy, not core** — they are minhag-contested, and baking them into
   correctness would violate /0002; the edge intersects with `classify_day`.
4. **tal-u-matar (coupling #4)** — **realm-selected calendar arithmetic, both branches F3-class**:
   `tal_umatar_start_date(year, basis, method) -> HebrewDate` (always computable) and
   `tal_umatar_start_instant(...)` (the maariv-entering boundary instant, or does-not-occur). EY /
   `Fixed7Cheshvan` → 7 Cheshvan; diaspora / `TekufaBased` → the **60th day after Shmuel's Tekufat
   Tishrei** (tekufa day = day 1; counting-day rolls at 18:00 — the Dec-5-vs-Dec-6 mechanism). The new
   `src/tekufa.rs` computes the arithmetic tekufa exactly in **regaim** (1/76 chalakim, so Rav Ada's
   235-months/19 year is exact), reusing the molad's UT projection.
5. **`molad_instant` → `chalakim_to_instant` extraction** (the one non-additive edit), so the tekufa
   shares the single flagged meridian projection and introduces **no new** geo assumption. Verified
   byte-identical by the existing molad FP probe (kind 10) + `f3_molad_and_kiddush_levana`.
6. **New knobs** `Realm{EretzYisrael,Diaspora}`, `TalUmatarBasis{TekufaBased,Fixed7Cheshvan}`,
   `TekufaMethod{Shmuel,RavAda}` (spec §1.D; plain CBOR-ready enums, no `Default` — all Req).

## Findings / refinements (surfaced, not worked around)

- **tal-u-matar is F3-class arithmetic, not F1 astronomy.** Shmuel's tekufa is the Julian 365¼-day
  construct, Julian-locked and deliberately *not* the true equinox; Rav Ada's is the Metonic mean.
  So spec §3 coupling-3's "`F1class.tekufa`" label is **imprecise** — corrected in the spec. A true
  astronomical-equinox method (if ever added) would be the only F1-class one.
- **Signature refinements from the plan:** tal-u-matar takes `basis`+`method` directly (the spec
  knobs; realm selects basis upstream) rather than threading `realm`; the start is split into an
  always-computable date and a fallible instant, rather than overloading `DayRoll`.

## Open gates — explicitly NOT decided here

(a) **Molad meridian** (`MOLAD_MERIDIAN_DEG_EAST`): reused/flagged; tal-u-matar's *date* is exact RD
arithmetic, so the meridian never decides a date. (b) **Day-roll / bein-hashmashot sanctity default**:
caller supplies the boundary read. (c) **Realm / Eretz-Yisrael geography**: `Realm` is a provisioned
input — the core never decides which sites are in Israel (Bamidbar-34 boundary; Eilat/Aleppo edges).
(d) **High-latitude fallback**: a typed does-not-occur (`DayRoll::BoundaryDoesNotOccur` / `Option` /
`false`) propagates at every boundary; what the edge displays is open (and whether KL needs a
tri-state vs `false`). (e) **/0003 validation tolerance**. (f) **Non-circularity line**: Wolfram stays
the sole raw-astronomy authority; MyZmanim is a composed-output cross-check only; integer couplings
compared exact against independent published tables.
- **Rav-Ada tekufa anchor:** shares the Shmuel creation anchor with the exact Metonic year length;
  the Shmuel branch is oracle-validated, the **Rav-Ada branch is not yet independently validated**.

## Rationale

The couplings share one shape — F3 integer context selects/​rolls; F1/F2 floats confirm/​bound — so
one module owns the only F1↔F3 dependency and the DAG stays acyclic (`events` never imports F3;
`calendar` never imports F1). The tekufa anchor is **calibrated**, not fitted: the Julian-locked
season length is exact, so only the integer phase is pinned (to the universally-published Oct-7 /
Dec-4-5-6 dates), cross-checked against the Birkat-Hachama-2009 anchor. Contested conventions stay
knobs or edge policy (/0002); does-not-occur stays a typed value, never a guessed instant (org/0006).

## Consequences

- New tests: `tests/couplings.rs` (3 — day-roll at shkia incl. the MyZmanim 24→25 Kislev anchor,
  polar does-not-occur, KL clause-by-clause), `tests/tekufa.rs` (10 — Tekufat Tishrei Oct-7 at
  3am/9am/3pm, tal-u-matar **Dec 5 / Dec 5 / Dec 6** across the 4-year cycle incl. the year-before-leap
  push, 7 Cheshvan). `tests/calendar.rs` grows to **38/38** (+11: the Yom-Tov-Sheni realm divergence
  16 Tishrei = CHM in EY / Yom Tov in diaspora, oracle-confirmed via MyZmanim; YK; Rosh Chodesh; the
  Shabbat-Chanukah and erev-Shabbat day-rolls; weekdays).
- **FP-determinism extended to the couplings: 20/20 exact native==wasm** (new kinds 11 tekufa-Shmuel,
  12 tekufa-Rav-Ada, 13 day-roll resolved RD incl. a far-longitude near-00:00-UT case, 14 night-
  predicate Sun altitude).
- **Regressions byte-unchanged:** F1 66/66, F2 11/11, F3 molad/KL, knobs 3/3, FP kind 10 — all green;
  the `molad_instant` extraction is byte-identical. `no_std`-clean; clippy clean (`deny(missing_docs)`
  / `deny(unsafe_code)`).

## Related

core-domain/0001 (the four couplings; molad ≠ observed moon), /0002 (parameters-over-core; realm and
KL exclusions), /0006 + /0013 (the optics seam reused for the day-roll boundary and the night
predicate), /0007 (tz-free; the UTC-civil-date subtlety), /0009 (the spec knobs + interface contract;
the "F1class.tekufa" correction), /0010 (`no_std`/FP determinism, extended), /0014 (F3; day-roll
deferral fulfilled), /0015 (F2 + molad; full-KL deferral fulfilled; molad meridian flag inherited).
Meta: org/0006.
