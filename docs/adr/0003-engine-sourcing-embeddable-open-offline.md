# ADR-0003 — Engine sourcing: embeddable open-license library, on-device, oracle-validated

- **Status:** Accepted (Determination 1)
- **Date:** 2026-06-15
- **Scope:** Stack-agnostic domain model. Names specific libraries/datasets only where the determination itself selects them; no runtime, platform, or service is assumed.

## Context

F1/F2/F3 (ADR-0001) need a concrete implementation of the underlying time math. The market research across all four lanes is consistent: the **engine layer is a commodity**. Free, permissively licensed engines already provide candle-lighting, havdalah, multiple shitot, Daf Yomi, and yahrzeit logic at essentially zero licensing cost; the moat is in display, management, and visible sunrise — not in the time math.

The research also surfaces a concrete trap. In Israel, **B.A. Timing Solutions Ltd** owns the **Chazon Shamayim / Sky-View** engine that "serves most of the computerized and printed boards in Israel" **and** sells its own competing finished board (CleverSign). Depending on that engine would put a direct competitor astride our correctness. Abroad, the ecosystem standardizes on Hebcal / GeoNames / KosherJava and **no US or French vendor was found reselling the Israeli engine** — confirming the open stack is the default path.

## Decision

- **Compute on-device** using an **embeddable, open-license library** of the KosherJava (LGPL-2.1) / Hebcal (CC-BY-class) family — specifically the *embeddable code* form, not a hosted web API.
- The implementation must be **fully offline and autonomous** through at least a multi-day chag + Shabbat (see ADR-0005).
- **Validate against a neutral oracle** — Wolfram or a national observatory — as a build/test fixture.
- **Depend on no competitor engine** (B.A. Timing Solutions / Chazon Shamayim, or the ChaiTables/Sky-View calendar tooling) for **correctness or validation**.

## Rationale

- **Commodity input, near-zero cost.** Using the open engines captures the entire time-math capability for free and lets all investment flow to the moat.
- **Strategic independence.** Consuming a competitor's engine would hand a rival visibility into and control over our correctness, and is acutely unwise given the confirmed vertical-integration (engine-maker = board-competitor).
- **Offline by construction.** An *embeddable library* runs locally; a *hosted API* (e.g. the hebcal.com REST endpoint) is a network dependency and is therefore excluded by ADR-0005. The distinction is load-bearing: we adopt the open *code*, not the open *service*.
- **Non-circular validation.** A neutral oracle (independent of any zmanim vendor) avoids validating one engine against another vendor's engine.
- **Autonomy window.** Boards must keep working across the exact multi-day stretch when network is most likely down and least wanted; correctness cannot assume a refresh during that window.

## Alternatives considered

- **License/consume B.A. Timing (Chazon Shamayim) API.** Rejected: competitor dependency, network reliance, and strategic exposure. The negative finding abroad (nobody resells it) reinforces that this is not the market-default path.
- **Cloud-compute zmanim and push to the device.** Rejected: violates the offline-correctness keystone (ADR-0005); a network outage during chag would break the board at its single most important moment.
- **Build a proprietary engine from scratch.** Rejected: reinvents a commodity at high cost with no moat gain, and would still require oracle validation regardless.
- **Validate against another vendor's engine.** Rejected: circular trust and renewed competitor dependency.

## Consequences

- **LGPL-2.1 obligations** (KosherJava) must be honored — the LGPL component must remain replaceable/relinkable, with library and license notices preserved. Track in licensing/compliance. Hebcal web-API *content* is CC-BY, but we are adopting the embeddable `@hebcal/core`-class *code*, so the relevant obligation is the code license, not the API content terms.
- The **neutral oracle is a test-time dependency only** — never a runtime dependency. A fixture harness compares F1/F2/F3 instants to the oracle to the second.
- **Port/platform selection is deferred** (stack-agnostic). Note one concrete constraint from the research: KosherJava's official port set does **not** include Swift; an Apple-platform target would use KosherCocoa (Obj-C) or a community Swift port. This is recorded as a constraint, not a platform decision.
- Couples tightly to ADR-0005 (offline) and ADR-0001 (the library *is* the F1/F2/F3 implementation).

## Open questions

- Which specific port/library per eventual target platform — deferred until the platform is chosen (stack-agnostic by rule).
- Exact neutral-oracle source and tolerance (Wolfram vs a named national observatory; second-level vs sub-second tolerance).

## Related

ADR-0001 (functions implemented), ADR-0004 (visible-sunrise sits on top of this engine), ADR-0005 (offline keystone), ADR-0007 (labeling layered after correct instants).
