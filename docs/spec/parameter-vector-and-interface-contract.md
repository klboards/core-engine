# Spec — Parameter-vector schema + F1/F2/F3 interface contract

- **Status:** Ratified contract (the repo's living spec). Decision recorded in **ADR-0009**;
  resolves ADR-0002's schema question and closes ADR-0006's refraction-default sub-question.
- **Scope:** Stack-agnostic logical contract. Types are abstract kinds. No language, framework,
  runtime, or serialization format is named or implied. The core **exposes** every knob and
  **resolves none** (ADR-0002): an unset *required* knob is a **configuration error surfaced by the
  core**, never a silent default. No halachic value is fixed here; angles, minutes, fractions, and
  coefficients are **community-supplied placeholders**.
- **Sources:** ADR-0001 (three functions + typed couplings), 0002 (variation = parameters; core
  resolves none), 0004 (terrain horizon profile; terrain ON/OFF), 0006 (refraction & horizon optics
  as parameters), 0007 (civil time / labeling outside the core), 0008 (engine-selection is
  correctness-bearing; serialization is an open cross-repo contract).

## 0. Conventions used in this document

**Abstract type kinds** (no language type system): `integer`, `rational`, `boolean`,
`enum{…}`, `angle-degrees`, `duration-minutes`, `duration-seconds`, `instant` (absolute / UTC),
`identifier` (opaque key), `reference` (a key naming another read in this same vector),
`locale-string`, `list<T>`, `map<K,V>`, and `one-of{…}` (a tagged union / discriminated variant).

**Givens vs knobs.** `(φ latitude, λ longitude, h elevation)` and the query instant `t` are the
**natural givens** of ADR-0001, not knobs — they are per-site context and the query moment. The
**parameter vector** is everything else: the convention/stringency/labeling choices. F1/F2/F3
take `(φ, λ, h, t)` **plus** the parameter sub-vector each consumes.

**The read-spec is the heart of the design.** Rather than enumerate a fixed list of zmanim with
baked-in values (which would smuggle policy into the core), the schema defines each zman as a
**typed read** off F1's continuous `altitude(t)` curve. Adding a community's opinion = adding a
data row, never code (ADR-0002). The read-spec variants are defined in §1.C.

---

## 1. KNOB CATALOG

Notation in **Default policy**: "**none — required**" means the core errors if the knob is
absent for a requested output (ADR-0002). "**config/preset layer**" means *no core default
exists*; the shipped base preset (outside the core) supplies a starting value, overridable per
tenant/site (§2). "**feature-gated**" means absence simply means the corresponding output is
**not emitted** — never silently defaulted.

### 1.A — Contract / engine block (correctness-bearing meta)

| Name | Meaning | Type | Unit | Domain / range | Req? | Default policy | Consumer | ADR |
|---|---|---|---|---|---|---|---|---|
| `schema.version` | Version of this logical contract the vector targets | identifier | — | known contract versions | **Req** | none — required | all | 0002 |
| `engine.selection` | Which correctness engine implementation evaluates F1/F2/F3 | `enum{owned-primary, alternative:<id>}` | — | owned-primary or a registered alternative id | **Req** | config/preset layer | F1/F2/F3 | **0008** |
| `parameter_set.id` | Provenance/audit handle for the resolved vector | identifier | — | opaque | Opt | feature-gated (audit only) | none (metadata) | 0002 |

> **`engine.selection` is correctness-bearing — same class as the day-boundary knobs, NOT
> cosmetic** (0008). Two engines = two correctness surfaces; the "device and apps share one core
> so they cannot drift" guarantee holds **only within a single engine choice**. An alternative
> engine MAY be flagged feature-incomplete (e.g. Israel/diaspora, Adar I/II) until parity is
> oracle-confirmed; our owned engine must honor every knob in this schema (that is our F3
> completeness bar).

### 1.B — Solar geometry / optics block (consumed by F1; 0004/0006)

| Name | Meaning | Type | Unit | Domain / range | Req? | Default policy | Consumer | ADR |
|---|---|---|---|---|---|---|---|---|
| `horizon.mode` | **Terrain/elevation ON/OFF**: idealized sea-level (mishor) vs terrain skyline | `enum{sea-level, terrain-profile}` | — | the two values | **Req** | config/preset layer | F1 | 0002, 0004 |
| `horizon.profile_ref` | Binding to the per-site `(azimuth→horizon-angle)` provisioning artifact | reference (artifact handle) | — | a provisioned profile bound to `(φ,λ,h)` | **Req if** `horizon.mode = terrain-profile` | feature-gated | F1 | 0004 |
| `refraction.model` | Refraction model selector composed into F1's horizon crossing. **Ratified default (base preset): `standard-atmospheric` (ADR-0009).** | `enum{standard-atmospheric, meeus-noaa, halachic-fixed-coefficient}` | — | the registered models | **Req** | config/preset layer (default `standard-atmospheric`) | F1 | **0006, 0009** |
| `refraction.coefficient` | Fixed horizon refraction value when model demands one (or an explicit override) | rational | arc-minutes | community-supplied | **Req if** `refraction.model = halachic-fixed-coefficient` | feature-gated | F1 | 0006 |
| `solar.position_reference` | Apparent (refraction-included) vs geometric sun position | `enum{apparent, geometric}` | — | the two values | **Req** | config/preset layer | F1 | 0006 |
| `solar.limb_reference` | Which part of the disc defines a horizon crossing | `enum{upper-limb, center, lower-limb}` | — | the three values | **Req** | config/preset layer | F1 | 0006 |

> **Provisioning↔runtime coupling (0004/0006):** the model in `refraction.model` (and any
> `refraction.coefficient`) is the **same** model the provisioning ray-trace used to emit
> `horizon.profile_ref`. Mismatch means the shipped profile does not compose correctly. This is a
> tested invariant (on-device composition matches a full recompute to tolerance), not a free knob
> per device.

### 1.C — Zman-definition block (the read-spec mechanism; consumed by F1)

A single structural collection plus the day-bound knob expresses "per-zman depression angle /
fixed-minute offset", "GRA vs Magen Avraham", and "day definition" — all as **data**.

| Name | Meaning | Type | Unit | Domain / range | Req? | Default policy | Consumer | ADR |
|---|---|---|---|---|---|---|---|---|
| `zman_definitions` | Map of zman-key → **read-spec** (see variants below); the requested outputs | `map<identifier, read-spec>` | — | one entry per requested zman | **Req** (for any requested zman) | config/preset layer | F1 | 0001, 0002 |
| `proportional_day_bounds` | **GRA vs MA / day definition**: the ordered pair of reads bounding the seasonal-hour ("halachic hour") day | pair of `reference` (start-read, end-read) | — | references resolving within `zman_definitions` | **Req if** any proportional read exists | config/preset layer | F1 | 0002 |

> **GRA vs MA *is* a setting of `proportional_day_bounds`**, not a separate code path:
> GRA ≈ `(sunrise, sunset)`; Magen-Avraham ≈ `(dawn, nightfall)`. Likewise "day definition
> netz→shkia vs alot→tzeit" is the same axis. A read MAY carry a local `bounds_override` for
> shitot that bound a specific zman differently from the canonical day.

**Read-spec variants** (`one-of`): every zman is exactly one of these. This is what makes
opinions additive-without-code (0002).

| Variant | Fields | Type(s) | Unit | Semantics |
|---|---|---|---|---|
| `depression-angle` | `angle`; `obligation_sense` | angle-degrees; enum (below) | degrees below horizon | Curve crossing at a depression angle (e.g. dawn/nightfall opinions). Community-supplied angle. |
| `horizon-crossing` | `which` ∈ {sunrise, sunset}; `obligation_sense` | enum; enum | — | The apparent horizon crossing — composes `horizon.mode` + `refraction.*` + `solar.*`. |
| `fixed-minute-offset` | `base` (reference); `offset`; `seasonal` (boolean); `obligation_sense` | reference; duration-minutes; boolean; enum | minutes | A clock-minute offset from another read (fixed-minute opinions, **candle-lighting**, havdalah). `seasonal=true` scales the minutes by season if the community so defines. |
| `proportional` | `fraction`; `bounds` (optional reference pair, else `proportional_day_bounds`); `obligation_sense` | rational; pair<reference>; enum | fraction of day | A fraction of the seasonal-hour day between two bounding reads (most "X seasonal hours" zmanim). |
| `extremum` | `kind` ∈ {solar-noon, solar-midnight} | enum | — | Chatzot — the curve's extremum/midpoint; needs no angle. |

> `obligation_sense ∈ {opens, closes, neutral}` feeds the rounding policy (§1.E): a read that
> *closes* an obligation rounds toward earlier to be stringent; one that *opens* rounds toward
> later. The core does not decide stringency direction per zman — it derives direction from
> `obligation_sense` + the global rounding policy.

> **High-latitude fallback (deferral — ADR core-domain/0009):** when a primary read returns
> `does-not-occur` (e.g. Alot/Tzeit R"T at −16.1° at high latitude in summer), the substitute to
> display is a community-supplied alternate read selected at the preset/edge layer — expressed with
> the existing `fixed-minute-offset` read-spec — not a new core knob. The core resolves none: it
> returns the typed `does-not-occur`; choosing what to show instead is a preset/edge policy
> (consistent with §1.F / ADR-0007).

### 1.D — Hebrew-calendar block (consumed by F3; 0001)

| Name | Meaning | Type | Unit | Domain / range | Req? | Default policy | Consumer | ADR |
|---|---|---|---|---|---|---|---|---|
| `locale.realm` | **Israel vs diaspora** — drives parsha divergence, yom-tov-sheni, tal-u-matar basis | `enum{eretz-yisrael, diaspora}` | — | the two values | **Req** | config/preset layer | F3 | 0001, 0008 |
| `nusach.customs` | Bundle of nusach calendar customs (Omer wording, Rosh-Chodesh/molad announcement, tachanun set, etc.) | `identifier` or `map<identifier, value>` | — | community-supplied | Opt | feature-gated | F3 | 0002 |
| `tal_umatar.basis` | Which rule starts tal-u-matar | `enum{tekufa-based, fixed-7-cheshvan}` | — | the two values | **Req if** tal-u-matar emitted | config/preset layer | F3 (gated by F1-class tekufa) | 0001, 0002 |
| `tekufa.method` | Tekufa computation method (for seasons + tekufa-based tal-u-matar) | `enum{shmuel, rav-ada}` | — | the two values | **Req if** `tal_umatar.basis = tekufa-based` | config/preset layer | F3 (consumes F1-class read) | 0001 |
| `adar.anniversary_rule` | How an Adar yahrzeit/anniversary maps across leap vs non-leap years | `enum` (community-supplied policy set) | — | registered policies | **Req if** yahrzeit emitted | config/preset layer | F3 | 0001, 0008 |
| `learning_cycles` | Which learning cycles to compute (daf yomi, etc.) | `list<identifier>` | — | registered cycles | Opt | feature-gated | F3 | 0001 |
| `omer.custom` | Omer counting/labeling custom | `identifier` | — | community-supplied | Opt | feature-gated | F3 | 0001, 0002 |

### 1.E — Stringency / rounding block (cross-cutting; applied to F1/F3 instant outputs)

| Name | Meaning | Type | Unit | Domain / range | Req? | Default policy | Consumer | ADR |
|---|---|---|---|---|---|---|---|---|
| `rounding.stringency` | Direction policy: lenient / stringent / nearest / truncate | `enum{lehakel, lehachmir, nearest, truncate}` | — | the four values | **Req if** any rounded output | config/preset layer | F1/F3 | 0002 |
| `rounding.granularity` | Rounding grain for emitted instants | `enum{second, minute}` or integer | seconds | community-supplied grain | **Req if** any rounded output | config/preset layer | F1/F3 | 0002 |

> The core composes `rounding.stringency` with each read's `obligation_sense` to choose a
> direction **per zman** — it never hard-codes "round down". The unrounded absolute instant is
> always available; rounding is an emission policy, not a change to the stored instant (0007).

### 1.F — Labeling / presentation block (part of the vector; **NOT consumed by F1/F2/F3**)

Per **0007**, the core emits **absolute instants and structural tokens only**; *all* language,
script, locale, and wall-clock rendering is **downstream at the display/input boundary**. These
knobs travel in the vector for completeness but are consumed by the edge/rendering layer, never by
F1/F2/F3.

| Name | Meaning | Type | Unit | Domain / range | Req? | Default policy | Consumer | ADR |
|---|---|---|---|---|---|---|---|---|
| `label.language` | Display language for resolved labels | locale-string | — | community-supplied | Opt | feature-gated (edge) | **edge layer (not core)** | 0002, 0007 |
| `label.script` | Script/orthography (e.g. Hebrew vs transliteration) | `enum`/identifier | — | community-supplied | Opt | feature-gated (edge) | **edge layer (not core)** | 0002, 0007 |
| `label.name_overrides` | Per-zman / per-event display-name overrides | `map<identifier, locale-string>` | — | community-supplied | Opt | feature-gated (edge) | **edge layer (not core)** | 0002, 0007 |

> Civil-time / DST labeling and `tzdata` are likewise outside this schema and outside the core
> (0007): the vector carries no time-zone knob; absolute→wall-clock is a versioned boundary
> concern handled where instants are displayed.

---

## 2. PRESET / "STREAM" MECHANISM (structure only — resolves no values)

A **preset** ("stream") is a **named bundle of knob values expressed purely as data** — a partial
parameter vector. Adding or changing a stream is therefore **additive without code** (0002): no
F1/F2/F3 branch, no new function. A preset MAY name a `base_preset` to inherit from (composition by
data, optional).

**Resolution happens entirely in the configuration layer, outside the core.** The layer flattens
the layers below into one fully-resolved vector and hands *only that* to the core; the core
performs **no layering and supplies no defaults** — it validates that every **required** knob is
present and rejects the vector with a configuration error otherwise (0002).

**Precedence (ratified — ADR-0009 Choice C; most-general → most-specific; later layers win per
knob-key):**

```
base preset  →  per-tenant override  →  per-site override
```

- Merge is **per-knob-key last-writer-wins**, with a **deep merge of the `zman_definitions` map
  by zman-key** (a site may override a single zman's read-spec without restating the bundle) and a
  deep merge of `nusach.customs` / `label.name_overrides`.
- `horizon.profile_ref` is intrinsically per-site (bound to `(φ,λ,h)`, 0004) and is therefore
  expected to be set at the **site** layer.
- The flattened result must be **complete** (all required knobs resolved) before the core is
  called; the core's first act is a completeness/closure check.

---

## 3. F1 / F2 / F3 INTERFACE CONTRACT (language-neutral)

Each function takes the givens `(φ, λ, h, t)` plus the **parameter sub-vector** it consumes, and
returns **absolute instants** (or typed windows / structural tokens). All outputs are timezone-free
(0001, 0007). Outputs that **cannot occur** at a given latitude/season (e.g. a depression angle
never reached) are returned as a **typed `does-not-occur`**, never a wrong instant.

### F1 — Solar geometry
- **Inputs:** `(φ, λ, h, t)`; sub-vector = `{horizon.mode, horizon.profile_ref?, refraction.model,
  refraction.coefficient?, solar.position_reference, solar.limb_reference, zman_definitions
  (solar reads), proportional_day_bounds, rounding.*}`.
- **Outputs:** continuous `altitude(t)`; per requested read-key → `instant | does-not-occur`;
  derived seasonal-hour length. Refraction + horizon profile compose **inside** F1 (0006).

### F2 — Lunar geometry
- **Inputs:** `(φ, λ, h, t)`; sub-vector = `{lunar visibility criteria for Kiddush Levana}`.
- **Outputs:** moon position / phase; **visibility predicate** `visible(t)` and visibility
  intervals. **No molad here** — molad is F3 arithmetic (0001).

### F3 — Hebrew-calendar arithmetic
- **Inputs:** `t`; sub-vector = `{locale.realm, nusach.customs?, tal_umatar.basis?, tekufa.method?,
  adar.anniversary_rule?, learning_cycles?, omer.custom?, rounding.*}`; **plus typed dependencies
  on F1 and F2** (below).
- **Outputs:** Hebrew date; day-type token; festival/fast/Rosh-Chodesh/Omer tokens; learning-cycle
  positions; tekufa & tal-u-matar instants/flags; yahrzeit mappings; Kiddush-Levana window. All as
  **instants + structural identifiers** (labels are downstream, 0007).

### Cross-function couplings — declared as TYPED dependencies (0001)

1. **F3 day-roll consumes an F1 read.** The Hebrew day rolls at a sun-defined instant, not civil
   midnight. F3 receives an injected boundary function `solar_day_boundary: (civil-date) → instant`
   supplied by **F1** under the *same* parameter sub-vector (the day-end read named by the
   `day_definition` setting of `proportional_day_bounds`, e.g. sunset/nightfall). Type:
   `F3.dayOf(t)` depends on `F1.read(boundary, φ, λ, h, ·)`.
2. **F3 day-type selects which F1 reads apply.** F3 emits a `day_type` token
   (weekday / shabbat / yom-tov / erev / fast / …). That token **selects** which solar reads are
   meaningful (candle-lighting only on erev; fast-start/end only on fast days). Type:
   `F1.requested_reads = select(zman_definitions, F3.day_type(t))`.
3. **tal-u-matar = tekufa gated by F3 date + realm.** The tekufa instant is an **F1-class**
   astronomical computation; whether/when tal-u-matar begins is **F3(date rule) + `locale.realm`**
   gating that instant, per `tal_umatar.basis`/`tekufa.method`. Type:
   `F3.talUmatarStart = gate(F1class.tekufa(tekufa.method), F3.date_rule, locale.realm)`.
4. **Kiddush Levana = F3 window confirmed by F2 visibility.** F3 computes the permissible window
   `[start, end]` from molad arithmetic; **F2** confirms moon visibility within it. Type:
   `result = confirm(F3.kidushLevanaWindow(t), F2.visible(·))` → the usable sub-interval.

---

## 4. SERIALIZATION (logical shape fixed here; on-wire encoding remains OPEN)

This spec fixes the **LOGICAL shape** of the parameter vector (knob names, abstract types, the
read-spec union, the layering semantics) and of its relationship to the horizon profile. It does
**not** choose an on-wire encoding.

Per **0008**, the **byte encoding of the parameter vector + horizon profile is an OPEN cross-repo
contract**: provisioning *writes* it, the device/apps *read* it, so it is a shared cross-language
format — explicitly **not** to be pre-resolved by any in-ecosystem default. Candidate families were
noted in 0008 (a language-native format, or CBOR / protobuf / flatbuffers-class). **The encoding
stays TODO**; only the logical shape is settled here.

---

## 5. Still-open, by design

- **On-wire serialization** of the parameter vector + horizon profile — cross-repo encoding
  contract (0008). This spec fixes only the logical shape.
- **Israel high-resolution DTM source** (0004) — untouched here; Israel sites fall back to the
  global baseline until resolved.
