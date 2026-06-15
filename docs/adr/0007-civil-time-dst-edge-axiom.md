# ADR-0007 — Civil time / DST: an edge axiom outside the core, versioned and never trusted as fixed

- **Status:** Accepted (Seam 2; subsumes the "civil time is not a domain function" axiom)
- **Date:** 2026-06-15
- **Scope:** Stack-agnostic domain model and edge boundary.

## Context

Wall-clock time is what a worshipper reads off the board, but it is **not** a domain function. It is a labeling convention layered on physical instants, determined by government decree (time zones, DST start/end, occasional one-off changes), and it can change at any time. The core (ADR-0001) computes and stores everything in absolute time — UTC instants and sun-angle events — and is timezone-free by construction.

## Decision

Civil time and DST are **outside the domain core entirely.**

- The core computes and stores in **absolute time** (UTC instants / sun-angle events), timezone-free.
- `wall-clock = absolute instant → label via IANA time zone / DST rules`, applied **only at the display and input boundaries**.
- The civil-time ruleset is **human-mutable** (changes by decree), so it must be **versioned** and **never trusted as fixed**. IANA `tzdata` is the standard carrier and is treated as versioned, cached data updated through the management/content channel — **not** a correctness dependency.

## Rationale

- **Determinism and oracle-comparability are preserved** by keeping a decree-driven, mutable convention out of the core. Absolute instants are stable and comparable to an observatory; wall-clock strings are neither.
- **Bounded failure mode.** Because the core stores absolute instants, a stale ruleset can **mis-label** a correct instant (e.g. show the wrong wall-clock across a DST boundary that was changed by decree) but can **never make the underlying zman wrong.** This is exactly what lets correctness be fully offline (ADR-0005) while labeling tolerates versioned staleness.
- **Standard carrier, clean update path.** Treating `tzdata` as versioned cached data means DST changes ride the normal update channel; offline devices keep last-known-good and recover on the next update.

## Alternatives considered

- **Compute and store in wall-clock / local time.** Rejected: bakes a mutable, decree-driven convention into the core; a single tz-rule change would corrupt previously "correct" stored times; not timezone-portable; not oracle-comparable.
- **Treat tz/DST rules as fixed constants.** Rejected: they change by government decree, so a fixed table silently goes wrong; the rules must be versioned.

## Consequences

- A thin **boundary layer** at display/input owns absolute ↔ wall-clock conversion using a versioned IANA/DST ruleset.
- **Input** (e.g. "set this yahrzeit reminder for 8:00 pm local") converts to an absolute instant **at entry** using the current ruleset, and stores the absolute instant — never the wall-clock string.
- Ruleset updates flow through the management/version channel (ADR-0005). Offline devices hold last-known-good; staleness mis-labels at most, is bounded, and self-heals on update.
- The core's public outputs are instants; **all** zone/DST/locale rendering is downstream of the core.

## Open questions

- **Staleness policy at the edge:** whether and how to signal to an operator that the cached civil-time ruleset is known-stale across an imminent DST boundary (interface concern; shared with ADR-0005's operator-facing staleness note).
- Update cadence / trust model for `tzdata` delivery over the management channel.

## Related

ADR-0001 (core emits absolute instants), ADR-0005 (offline correctness; bounded labeling staleness).
