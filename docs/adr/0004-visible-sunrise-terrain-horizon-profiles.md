# ADR-0004 — Visible sunrise: terrain-corrected horizon profiles computed at provisioning

- **Status:** Accepted (Determination 2)
- **Date:** 2026-06-15
- **Scope:** Stack-agnostic. Names specific elevation datasets only because the determination selects them by license/resolution; distribution channels (e.g. open-data registries) are noted as facts about the data, not as a stack choice.

## Context

Visible-horizon ("visual sunrise/sunset") zmanim — terrain-corrected for the real skyline, not an idealized sea-level horizon — is identified by the substrate research as the **most differentiated and genuinely scarce** capability in the engine layer. Generic engines (Hebcal, KosherJava) approximate it but do not fully replicate it; the specialists who do (ChaiTables / Sky-View) are engine-layer rivals. The research also notes that a vendor serving Israeli/Sephardi communities that does **not** handle visible sunrise carries a quality gap in exactly the market that most values it.

This capability is therefore the defensible part of the product (consistent with the moat logic in ADR-0003) and must be built in-house, offline, and on commercially clean data.

## Decision

Provide terrain-corrected sunrise/sunset via a **per-site horizon profile**:

- **DEM baseline:** Copernicus GLO-30 — global 30 m, commercially permissive (distributed via open-data mirrors including the AWS Open Data registry).
- **Per-site high-resolution overrides:** 1 m lidar where available — **USGS 3DEP** (US), **IGN RGE ALTI** (France). **Israel high-resolution DTM source = TODO** (open).
- **FABDEM** (forest/building-removed DEM) only under its **commercial license**, where licensed.
- **Compute the horizon at provisioning time** (server-side, where network, full DEM, and compute are available): ray-trace the skyline per azimuth, apply the refraction-aware optics, and **emit a small per-site `(azimuth → horizon-angle)` profile**.
- **Ship the compact profile to the device.** At runtime the device **composes the stored profile with F1 geometry + a refraction model, fully offline** (ADR-0006), to read the apparent horizon-crossing instant.

## Rationale

- **Moat capability, built not borrowed.** This is the scarce differentiator; depending on ChaiTables/Sky-View tables would reintroduce a competitor/third-party dependency (same logic as ADR-0003) and break offline operation.
- **Heavy work at provisioning, trivial work at runtime.** Full-DEM ray-tracing is large and compute-intensive; doing it once per site, server-side, and shipping a compact profile keeps the device offline-correct (ADR-0005) and cheap. The runtime cost is composing a small lookup with the live `altitude(t)` curve.
- **Dataset choices follow license + resolution.** Copernicus GLO-30 is a clean global baseline; national 1 m lidar gives per-site precision where it exists; FABDEM is gated because its license is commercial.

## Alternatives considered

- **Sea-level / mishor only (no terrain).** Rejected: discards the scarce differentiator and is a known quality gap for terrain-affected and stringency-minded (Sephardi / Eretz-Yisroel) sites.
- **Full-DEM ray-tracing on-device at runtime.** Rejected: DEM size and compute require network/heavy resources and break the offline + cheap-device constraints.
- **Depend on ChaiTables / Sky-View terrain tables.** Rejected: third-party/competitor engine dependency and an online dependency; contradicts ADR-0003 and ADR-0005.
- **FABDEM as the baseline.** Rejected for baseline on commercial-license cost; permitted only on sites where a license is held.

## Consequences

- A **server-side provisioning pipeline** is required: DEM selection → per-azimuth skyline ray-trace → refraction-aware optics → compact `(azimuth → horizon-angle)` profile. The optics conventions in that pipeline are parameters (ADR-0006).
- The horizon profile is a **provisioning artifact bound to `(φ, λ, h)`**; if a site relocates or its `h` changes, it must be re-provisioned.
- **"Visible sunrise ON/OFF" is a parameter** (ADR-0002), as is the refraction convention applied.
- Until the Israel DTM source is resolved, **Israel sites fall back to Copernicus GLO-30 (30 m)** — a known precision gap in precisely the market that values visible sunrise most. Prioritize closing it.

## Open questions

- **Israel high-resolution DTM source** — explicitly flagged open question; the current ceiling for Israel sites and a priority because of market fit.
- **Refraction model** used in the ray-trace — owned by ADR-0006; the provisioning pipeline and the device runtime must use the *same* model so the shipped profile composes correctly.

## Related

ADR-0002 (terrain ON/OFF as a knob), ADR-0003 (build-not-borrow moat logic), ADR-0005 (offline runtime composition), ADR-0006 (refraction/horizon optics seam).
