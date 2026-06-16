# ADR core-domain/0014 — F3 core: Hebrew calendar (Dershowitz–Reingold fixed arithmetic)

- **Status:** Accepted
- **Date:** 2026-06-16
- **Scope:** F3 (Hebrew-calendar arithmetic) of ADR core-domain/0001 — **core-first** subset. New
  `src/calendar.rs`. Stack: Rust/no_std (core-domain/0010, /0012).

## Context

The post-Hillel-II Hebrew calendar is a **fixed arithmetic** system (Rambam, *Hilchot Kiddush
HaChodesh*) — fully deterministic, no observation. The molad is calendar arithmetic (F3), not the
observed moon (F2) (core-domain/0001).

## Decision

1. **Algorithm = Dershowitz–Reingold fixed arithmetic**, exact integer (RD day-number pivot;
   `(7y+1) mod 19` Metonic leap; molad-elapsed-days with the **four dechiyot** folded in; year
   length/type; month lengths incl. **Adar I=m12 / Adar II=m13**; Gregorian↔Hebrew). **No floats**
   ⇒ cross-target determinism is **structural** (the /0010 FP gate is N/A to F3; `calendar.rs` uses
   no `libm`).
2. **Month numbering** Nisan=1 … Tishrei=7 … Adar=12 (common) / Adar I=12, Adar II=13 (leap) —
   matches Wolfram and the fixture.
3. **Oracle = Wolfram "Jewish" calendar (primary) + Hebcal REST cross-check** (build/test only,
   never shipped — core-domain/0003/0008). All vectors validated to **triple agreement**
   (engine = Wolfram = Hebcal) on the Adar I/II cases.
4. **`AdarAnniversaryRule` knob** (params): a common-year Adar yahrzeit observed in a **leap** year
   → **Adar II default** (Rema / most poskim), `AdarI` / `Both` selectable. Core resolves none.
5. **Festivals/fasts/Rosh Chodesh** as structural **tokens** (`Festival` enum), not localized
   strings (labels downstream, core-domain/0007).

## Scope (core-first) & deferrals

In: Gregorian↔Hebrew conversion, leap/year-type/month-length, Rosh Hashanah/dechiyot, festival
date tokens, yahrzeit + Adar knob. **Deferred (additive follow-on):** parsha (Israel/diaspora) +
yom-tov-sheni, Omer, learning cycles, tekufa/tal-u-matar, and the precise **molad-moment** function
(landed in Phase 2 with Kiddush Levana, validated against Wolfram's lunation).

## Rationale

Fixed arithmetic is exact and authoritative (no precision/oracle-tolerance question as for F1/F2);
the dual oracle (neutral Wolfram + specialist Hebcal) guards the dechiyot/Adar edge cases; halachic
conventions are knobs, not code (core-domain/0002/0009).

## Consequences

- `tests/calendar.rs` validates **23/23** F3 vectors exactly (conversion across year-types,
  leap/length, festivals, Adar I/II, yahrzeit rule). F1 golden 66/66 unaffected.
- **F3↔F1 day-roll coupling** (the Hebrew day rolls at an F1 sunset, not civil midnight) is
  **Phase 3** — this pass is day-granular (civil date → Hebrew date).
- Determinism structural (integer); no_std-clean; clippy clean (`deny(missing_docs)`/`unsafe_code`).

## Related

core-domain/0001 (F3 in the three-function model), /0002 (knobs), /0003 + /0008 (oracle + Hebcal
cross-check), /0007 (tokens not labels), /0009 (parameter-vector; Adar knob), /0010 + /0012
(no_std/determinism). Meta: org/0006.
