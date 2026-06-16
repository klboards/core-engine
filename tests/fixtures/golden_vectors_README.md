# klboards — Oracle golden-vector fixture (zmanim correctness)

*Generated 2026-06-16 against the Wolfram Language oracle (the neutral reference named in
core-domain/0003). Companion data: `golden_vectors.csv` (58 rows). This is a decision/research-track
artifact intended to seed the `core-engine` test fixture required by core-domain/0003 and
open-decision #9 (CI: neutral-oracle validation).*

## What this is and why it exists

A small, oracle-verified table of `(location × date × zman)` → expected instant, to be checked
against the klboards engine to a stated tolerance. It does **two** jobs:

1. **Oracle fixture** — the values are Wolfram outputs, so they *are* golden by construction
   (core-domain/0003: validate against a neutral oracle, never against a competitor engine).
2. **Schema pressure-test** — generating it surfaced concrete requirements the parameter-vector
   schema (core-domain/0002) and the F1/F2/F3 output types (core-domain/0001) must satisfy. See
   "Findings that constrain the schema" below.

**Validation anchor:** the Jerusalem `(31.78, 35.23, 754 m)` 2026-06-15 row reproduces the
hand-validated table in `klboards_domain_determinations.md` **to the minute** for all nine values
(Alot 04:05 · Misheyakir 04:32 · Netz 05:26 · Chatzot 12:39 · Shkia 19:52 · Tzeit 20:29 ·
Tzeit R"T 21:14 · sha'ah zmanit 72.2 min · 30 Sivan 5786). The method is therefore trusted.

## The zman taxonomy this fixture exercises (maps 1:1 to core-domain/0001)

Every zman is one of four reads off the same machinery — exactly the decomposition in ADR-0001:

| def_type | examples | depends on |
|---|---|---|
| `depression_angle` | alot −16.1°, misheyakir −11.5°, tzeit −8.5°, tzeit R"T −16.1° | **pure solar geometry** — refraction-independent (sun is far below the horizon, refraction ≈ 0) |
| `horizon_crossing` | netz, shkia | **refraction + horizon/elevation** (the convention-bearing seam, ADR-0006) |
| `extremum_midpoint` | chatzot | netz/shkia (or meridian transit) |
| `proportional` | sha'ah zmanit (GRA shown) | a duration `(shkia−netz)/12`, **not an instant** |

This split is not cosmetic: it tells you which knobs touch which zman. The depression-angle zmanim
were identical regardless of refraction model in every computation; only the horizon-crossing
zmanim moved. That is ADR-0006's seam, observed empirically.

## Findings that constrain the schema (the real value of this exercise)

1. **A zman result is NOT "a time." It is an optional instant with a civil-day label.** Two edge
   cases force this, both at high latitude in June:
   - **London (51.5°N), 2026-06-15:** deepest sun depression of the night = **−15.1°**, so
     Alot and Tzeit R"T at −16.1° **do not exist**. The output type must allow `NONE`.
   - **Paris (48.9°N), 2026-06-15:** deepest depression = **−17.7°**, so Tzeit R"T at −16.1°
     *does* exist — but it lands at **~00:40 the next civil day**. The output type must carry a
     day label; "which civil day is this zman on" is not assumable.
   → **Schema/interface requirement:** F1 zman outputs are `Optional<AbsoluteInstant>` plus an
     explicit civil-day tag. The device storing absolute instants (ADR-0001/0007) handles both
     cleanly; any wall-clock/"today's row" projection must not assume one-zman-per-civil-day.
   → This also means the parameter schema needs a **high-latitude fallback policy** knob (what to
     show when a depression-angle zman is absent — e.g. fixed-minute alternative, or "n/a"). That
     is a real community-facing parameter, not an engine internal.

2. **Horizon/elevation dominates refraction-model choice by ~10×.** At Jerusalem:
   - Sea-level vs 754 m terrain horizon moves **netz by 6 min 15 s** (05:32:53 → 05:26:38).
   - Bennett vs Saemundsson refraction at the horizon differs by ~5.5′ ≈ **~tens of seconds**
     (and part of that is an input-convention artifact between the two formulas).
   → **Implication for ADR-0006 (refraction model — open):** the formula default is nearly a
     rounding choice; the **horizon/elevation profile (ADR-0004) is where the real time-shift and
     the moat live.** Recommendation: default to the **NOAA/Meeus** model already used by
     Hebcal/KosherJava (guarantees oracle agreement, zero surprise), expose the refraction
     coefficient + a "halachic fixed-coefficient" option as parameters, and put engineering effort
     into the terrain horizon, not the refraction formula. The binding requirement from ADR-0006
     stands regardless: provisioning and runtime must use the **same** model.

3. **`sha'ah zmanit` is a duration, not an instant** (72.2 min summer vs 51.4 min winter at
   Jerusalem). The schema/output type must distinguish duration-valued outputs from instant-valued
   ones; GRA vs MGA is just *which two crossings* bound it (ADR-0002).

## Precision / tolerance caveat (open, owned by core-domain/0003)

The CSV is rounded to the **minute** — enough to seed the fixture and to validate the method, not
to certify the engine. core-domain/0003 leaves the oracle **tolerance** open (second vs sub-second).
Regenerate at full precision once that tolerance is ratified; the generator below already computes
to sub-second internally. Store the absolute UTC instant alongside the local label in the certified
fixture (the local label is a projection; the instant is the truth — ADR-0007).

## How to regenerate (validated Wolfram Language generator)

Stateless kernel; `SunPosition` returns `{azimuth, altitude}` (altitude is element **2**). Build
day-granularity-free `DateObject`s so second-shifts apply. Display via `TimeZoneConvert` (the
headless kernel's `$TimeZone` is not the site's).

```wolfram
solveDay[name_, geo_, tzS_, ymd_] := Module[
  {doS, mid, hh, altS, sr, ss, nSec, sSec, chatz, shaah, scanWin, pick},
  doS[s_] := DateObject[Join[ymd, {0, 0, s}], TimeZone -> tzS]; mid = doS[0];
  hh[t_] := DateString[TimeZoneConvert[t, tzS], {"Hour24", ":", "Minute", ":", "Second"}];
  altS[s_?NumericQ] := QuantityMagnitude[SunPosition[geo, doS[s]][[2]]];  (* altitude = [[2]] *)
  sr = Sunrise[geo, mid]; ss = Sunset[geo, mid];                          (* elevation-aware *)
  nSec = QuantityMagnitude[UnitConvert[DateDifference[mid, sr, "Second"], "Seconds"]];
  sSec = QuantityMagnitude[UnitConvert[DateDifference[mid, ss, "Second"], "Seconds"]];
  chatz = (nSec + sSec)/2; shaah = (sSec - nSec)/12.;
  scanWin[depr_, sLo_, sHi_] := Module[{ts, pts, br}, ts = Range[sLo, sHi, 300];
    pts = ({#, altS[#] - depr} &) /@ ts;
    br = Select[Partition[pts, 2, 1], (#[[1, 2]] #[[2, 2]] <= 0) &];
    Quiet[((s /. FindRoot[altS[s] == depr, {s, #[[1, 1]], #[[2, 1]]}]) &) /@ br]];
  pick[depr_, morning_] := Module[{cs},
    cs = Select[If[morning, scanWin[depr, Max[0, nSec - 14400], nSec],
                            scanWin[depr, sSec, Min[86400, sSec + 14400]]], NumericQ];
    If[cs === {}, "ABSENT-or-rollover", hh[doS[If[morning, Max[cs], Min[cs]]]]]];
  {name, hh[sr], hh[ss], hh[doS[chatz]], Round[shaah/60., 0.01],
   pick[-16.1, True], pick[-11.5, True], pick[-8.5, False], pick[-16.1, False]}];

(* For ABSENT vs ROLLOVER: deepest depression of the night classifies it.        *)
(* If min night altitude > target depression  => ABSENT (event never occurs).     *)
(* Else the event exists; extend the evening window past 86400 s to catch a       *)
(* post-civil-midnight crossing and tag it +1d.                                   *)
```

`CalendarConvert[DateObject[{y,m,d}], "Jewish"]` supplies the F3 calendar half (validated:
2026-06-15 → `{5786, 3, 30}` = 30 Sivan 5786).

## Coverage in this seed set

4 locations (Jerusalem 754 m · Brooklyn · Paris · London) × 2 dates (summer solstice-adjacent
2026-06-15 · winter solstice 2026-12-21). Deliberately spans: an elevation case (Jerusalem 754 m),
mid-latitude diaspora (Brooklyn), a high-latitude rollover (Paris June), and a true-absence edge
(London June). **Still not covered** (next expansion): a Southern-Hemisphere site (seasonal
inversion) and a per-tenant/site preset-override round-trip.

## Schema-validation pass (post-ratification of A/B/C, 2026-06-16)

Three axes were added (rows 59–69) specifically to stress the just-ratified parameter-vector
schema (core-domain/0002) *before* it is recorded as an ADR. **The schema survived all three —
no new gap surfaced; each axis resolved cleanly to an already-ratified knob:**

1. **GRA vs MGA proportional day** → resolves entirely to `proportional_day_bounds` (netz→shkia
   vs alot→tzeit). One knob produces a ~40-min spread in `sof_zman_shma` (Jerusalem summer
   GRA 09:03 vs MGA 08:22). Confirms 0002's "GRA-vs-MA is a setting, not a code path."
2. **DST-transition labeling** (Brooklyn netz across spring-forward) → the absolute instant moved
   −1 min (11:18Z→11:17Z) while the wall-clock label jumped +59 min (06:18→07:17). Confirms
   §1.F / ADR-0007: the instant is the truth, the label is a versioned edge projection; tzdata
   stays out of the core.
3. **Adar I/II yahrzeit** → a 15-Adar yahrzeit recurs in leap year 5787 as either 22 Feb 2027
   (Adar I) or 24 Mar 2027 (Adar II), one month apart. Empirically justifies Choice B making
   `adar.anniversary_rule` *required-if-yahrzeit-emitted*.

**One thin spot persists (not a contradiction):** the schema represents a non-occurring zman as a
typed `does-not-occur` (London Alot/Tzeit R"T in June) but has **no named high-latitude-fallback
knob** — "show *what* instead." A community must currently express that as a separate fixed-minute
read. Flag for the follow-up recording run: decide whether this is a deliberate deferral (policy
stays at the edge / preset layer) or a knob worth naming in 0002 before it is frozen.
