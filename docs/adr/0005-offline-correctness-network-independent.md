# ADR-0005 — Offline correctness is network-independent

- **Status:** Accepted (Determination 3)
- **Date:** 2026-06-15
- **Scope:** Stack-agnostic domain model and device contract.

## Context

A synagogue board is needed most precisely when the network is least available or least wanted — across Shabbat and multi-day chagim, when connectivity may be down and, in many communities, network use is itself avoided. A board that needs connectivity to show the right time fails at the only moments it must work. Competitor boards already treat offline operation as table stakes (e.g. CleverSign advertises offline operation; the mosque comparable Mawaqit ships an offline version).

## Decision

**Nothing about correctness may depend on the network.** The network exists only for **management, content, and ruleset/version updates** — never for computing a correct result.

Concretely, the device must produce correct zmanim, calendar data, and labels with **zero connectivity for at least a full multi-day chag + Shabbat**. Every correctness-bearing input is resident and valid on-device:

- the F1/F2/F3 engine computes locally (ADR-0003);
- the per-site horizon profile is pre-shipped at provisioning (ADR-0004);
- the civil-time / DST ruleset is versioned and cached locally (ADR-0007).

## Rationale

- **Fails-safe at the critical moment.** Forcing all correctness inputs on-device removes the single point of failure that a cloud-rendered or network-required board would have during chag.
- **Clean split between correctness and freshness.** Correctness (the instants and the calendar) must be offline-guaranteed. Freshness/management (announcements, remote admin, version updates) is a network feature and may degrade gracefully when offline — without ever regressing correctness.
- **Bounded staleness, not best-effort.** Where an input is human-mutable (civil time/DST, ADR-0007), the answer is *versioning with bounded failure*, not "hope for connectivity." A stale ruleset can mis-**label** but can never make a zman **wrong**, because the core stores absolute instants (ADR-0001/0007).

## Alternatives considered

- **Thin client / cloud-rendered board.** Rejected: a single point of failure during exactly the multi-day window the product exists to serve; also below the market's table-stakes bar for offline operation.
- **Network-required with "best-effort" local cache fallback.** Rejected: correctness must be guaranteed, not best-effort. A cache that can silently go stale on a mutable axis is handled by explicit versioning (ADR-0007), not by degrading correctness when the network is absent.

## Consequences

- All correctness-bearing data — engine, horizon profile, calendar rules, DST ruleset — must be present and valid on-device, each with explicit **version + staleness semantics**.
- The blast radius of offline staleness is **bounded to labeling**: a stale civil-time ruleset can mis-label, but the absolute instant remains correct and recoverable on the next update (ADR-0007).
- **Network features must degrade gracefully** — no correctness regression when offline; offline is the assumed steady state during chag.
- A concrete acceptance test follows directly: **simulate an N-day no-network chag + Shabbat and assert full correctness** throughout, with only labeling tolerant to a deliberately stale ruleset.

## Open questions

- Maximum guaranteed autonomy window `N` (days) to certify against — at least one full multi-day chag + adjacent Shabbat; exact certified maximum TBD.
- Operator-facing staleness policy: what, if anything, to surface when a cached ruleset is known-stale across a DST boundary (interface concern, owned with ADR-0007).

## Related

ADR-0001 (absolute instants bound the failure mode), ADR-0003 (on-device engine), ADR-0004 (pre-shipped horizon profile), ADR-0007 (versioned DST ruleset, bounded mislabeling).
