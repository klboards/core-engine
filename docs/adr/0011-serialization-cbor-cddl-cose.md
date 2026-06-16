# ADR core-domain/0011 — Serialization: CBOR family + CDDL contract + COSE signing

- **Status:** Accepted
- **Date:** 2026-06-16
- **Scope:** Decides the on-wire serialization for the cross-repo **parameter-vector +
  horizon-profile** contract. The stack-agnostic hard rule **does not apply to format-naming
  here** — the format *is* the decision; the chosen formats are named explicitly. Closes the
  serialization open-question (core-domain/0008; schema spec §4). Records four open sub-items.

## Context

core-domain/0008 flagged the param-vector + horizon-profile **serialization** as an OPEN
cross-repo encoding contract — provisioning *writes* it, the device/apps *read* it — and warned
explicitly against pre-resolving it with an in-ecosystem (Rust-native) default. core-domain/0009
§4 fixed only the *logical* shape. core-domain/0010 set the engine language = Rust. This ADR
decides the wire format itself.

## Decision

- **Wire-format family for BOTH artifacts = CBOR** (RFC 8949).
- **The cross-repo contract is expressed as a CDDL schema** (RFC 8610) — the shared, codegen-free
  contract all repos agree to; it lives in the shared contract location (`docs/spec/`, alongside
  the parameter-vector + interface contract).
- **Signing = COSE_Sign1** (RFC 9052) over deterministically-encoded CBOR — the standards-based
  path satisfying org/0006 §7 on-device signature verification (no_std-feasible).
- **TWO separate, independently-versioned, independently-signed artifacts** (not one blob):
  - **(a) parameter vector** — small structured config (core-domain/0009 schema);
  - **(b) horizon profile** — a CBOR envelope carrying **binding metadata** [the `(φ, λ, h)` it is
    bound to, the DEM source/version, and the refraction model + coefficient used at provisioning —
    required by the core-domain/0004 & core-domain/0006 provisioning↔runtime invariant] **plus a
    packed numeric field** for the azimuth→horizon-angle array.

## Rationale

- **The WRITER is provisioning-builder** (a different repo, **not** the Rust engine), so the
  contract must be **cross-language by construction**. A Rust-native format (postcard/bincode) is
  therefore excluded *despite* the engine being Rust — this is exactly the in-ecosystem default
  core-domain/0008 warned against.
- **CBOR clears the cross-language bar**: a first-class Python writer (cbor2), a JS reader for the
  PWA, and a genuine `no_std` Rust reader (minicbor-class) for the device — the `no_std` fit is a
  *bonus* that honors core-domain/0008, not the reason for the choice.
- **CDDL gives protobuf-like contract clarity WITHOUT codegen lock-in.**
- **COSE_Sign1 makes serialization and org/0006 §7's signed-artifact requirement the SAME
  decision** — a Rust-native format gives nothing here; CBOR + COSE gives the signed-update story
  for free.
- **Compactness:** storing horizon angles as f32/f16/fixed-point does **not** violate the F1/F2
  double-precision floor — the floor governs **computation**; provisioned angles are **input data**
  composed with double-precision math on-device. (Stated explicitly because it reads as a
  contradiction otherwise.)

## Alternatives considered

- **Rust-native (postcard/bincode)** — leanest `no_std`, but no cross-language writer → excluded;
  also the in-ecosystem default core-domain/0008 forbade.
- **Protobuf** — cross-language but codegen in every repo, awkward `no_std`, and no clean
  self-describing-audit / sign-over-bytes story → heavier than needed.
- **FlatBuffers / Cap'n Proto** — zero-copy device reads, but premature at low-KB sizes; **kept as
  a named fallback** if device profiling later shows real parse/RAM pressure on the horizon
  profile.

## Consequences

- **Closes the serialization open-question** flagged by core-domain/0008 and schema spec §4.
- **OPEN sub-decisions (recorded, NOT resolved here):**
  1. exact **deterministic-CBOR profile** (core-deterministic vs dCBOR);
  2. **horizon-angle value type** (f32 / f16 / fixed-point milliarcminutes — size vs resolution);
  3. **one channel vs two** for the two artifacts;
  4. the **device-side COSE verification path**, coupled to org/0006 §7's still-open root-of-trust
     (HW secure-boot vs pinned-key) — **must NOT be force-closed here.**
- Engine stays Rust (core-domain/0010); a minicbor-class `no_std` reader on the device.

## Related

core-domain/0002 (param-vector schema), core-domain/0004 (horizon profile + binding metadata),
core-domain/0008 (serialization flagged open; in-ecosystem-default warning), core-domain/0009
(logical shape; spec §4), core-domain/0010 (engine = Rust); org/0006 (§7 signed-artifact /
root-of-trust).
