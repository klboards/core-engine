# ADR core-domain/0022 — Daf Yomi + Israeli national days (offline calendar facts)

- **Status:** Accepted
- **Date:** 2026-06-20
- **Scope:** Adds two offline, deterministic board **calendar facts** the device shows: (1) **daf-yomi**
  (daily Talmud-Bavli page) — new `src/daf_yomi.rs` (`daf_yomi(RataDie) -> DafYomi{masechta,daf}`); and
  (2) the **modern Israeli national days** with the Knesset Shabbat-shift — added to `src/calendar.rs`
  (`israeli_national_day(HebrewDate) -> Option<IsraeliDay>`). Both F3-family calendar arithmetic, both
  return **indices/enums, not names** (names are localizable content), both Hebcal-oracle-validated.

## Context

A synagogue board commonly shows the day's daf. It was the one calendar object with **no offline
representation** (core-domain/0014 deferred it as "Scope B, not in the 8-stage spine"). The device
computes everything offline, so daf-yomi belongs in the engine, not a shipped-and-stale table — it is a
fixed **civil-day** cycle (one daf/day) and thus pure integer arithmetic, the cleanest possible engine
addition (no FP, native==wasm exact by construction).

## Decision

A periodic cycle of `DAF_YOMI_CYCLE_DAYS = 2711` days anchored at the **14th-cycle epoch 2020-01-05 =
Berachos 2**. Day-in-cycle `= (rd − epoch).rem_euclid(2711)`, walked through the modern masechta table
(`MASECHTA_LAST_DAF`, = KosherJava's `blattPerMasechta`; each masechta learned daf 2..=last, occupying
`last−1` days; ∑ = 2751 − 40 = 2711). The Meilah-block masechtos (Kinnim/Tamid/Middos) continue Meilah's
Vilna-Shas pagination — applied as **display offsets** (+21 / +24 / +32). Returns `{masechta: u8 (0..39),
daf: u16}`.

**Scope = the modern table** (Yerushalmi Shekalim = 22 daf, in force since the 8th cycle, 1975-06-24).
`rem_euclid` makes it correct for any date from 1975-06-24 onward — all the device ever shows (present +
future). Pre-1975 cycles (13-daf Shekalim) are **out of scope** and would compute wrong; the device never
displays them. (KosherJava's 1923 epoch + the pre-1975 branch are deliberately *not* ported — unneeded
complexity for a present/future board.)

## Validation

- **Hebcal differential** (`tests/daf_yomi_oracle.rs` + `tests/fixtures/daf_yomi_vectors.csv`): 14 dates,
  **exact**, covering Berachos→Shabbos boundary, Shekalim, mid-cycle (Yoma/Nedarim/Bava Basra/Chullin/
  Temurah), and the whole tricky end-block — Meilah, **Kinnim (+21), Tamid (+24), Midos (+32)**, Niddah.
  Hebcal is an independent implementation (oracle-as-fixture, never shipped).
- Unit tests: epoch = Berachos 2, ∑ day-counts = 2711, the Berachos→Shabbos boundary, the Meilah-block
  offsets. `cargo test` + clippy `-D warnings` + fmt + the `no_std` wasm build all green. No FP path ⇒ the
  659/659 native==wasm gate is unaffected (integer-exact by construction).

## Israeli national days

`israeli_national_day(HebrewDate) -> Option<IsraeliDay>` returns the day observed on a Hebrew date with
the Knesset Shabbat-shift applied (verified weekday encoding: `weekday_from_fixed` = `amod(rd,7)`, Sun=0):
- **Yom HaShoah** (27 Nisan): Fri→−1 (Thu), Sun→+1 (Mon).
- **Yom HaZikaron** (4 Iyar): Thu→−1, Fri→−2 (both Wed), Sun→+1 (Mon); **Yom HaAtzmaut = +1 day**.
- **Yom Yerushalayim** (28 Iyar): unshifted.

Realm/community **opt-in** (a board surfaces them only where observed); the arithmetic is realm-free.
Validated by `tests/israeli_days_oracle.rs` against Hebcal over **2025–2033 (36/36)** — a span hitting
every shift branch (Fri/Sun for Shoah; Thu/Fri/Sun for Zikaron; unshifted years). Integer-only ⇒ no FP,
659/659 native==wasm retained.

## Consequences

- Daf-yomi is now an offline, oracle-validated engine fact; halacha-model adds a `daf-yomi` **calendar
  object** that calls it, and the content layer localizes the masechta name (he/en/yi/…).
- The cycle table is a one-time constant; a future need for pre-1975 dates (none foreseen) would add the
  historical Shekalim branch + the 1923 epoch behind the same API.

## Open / not addressed

- Pre-cycle-8 (pre-1975) dates compute wrong by design (out of scope).
- Daf-Yomi *Yerushalmi* and other learning cycles (Mishna Yomi, Nach Yomi) — not built; same pattern if
  ever needed.
