# ADR core-domain/0010 — Engine language = Rust; device runtime model = freestanding (no_std)

- **Status:** Accepted
- **Date:** 2026-06-16
- **Scope:** Decides the **engine** (the correctness core, core-domain/0001) only. Resolves the
  **engine-language + runtime-model** portion of open-decision #8. Management-side repos
  (provisioning-builder, control-plane, admin-web, community-pwa, community-android, infra) keep
  their languages **OPEN**, chosen per-repo; they are not correctness-bearing and must not be
  coupled to this decision. This ADR **records a decided choice** — it does not default an
  undecided one, so the stack-agnostic hard rule is honored.

## Context

open-decision #8 left the engine implementation **language** OPEN. core-domain/0008 decided the
engine *posture* (own primary engine behind a pluggable interface) but explicitly **not** the
language; org/0006 set the device runtime memory class to **Profile A (freestanding / no resident
managed runtime)** as design intent, gated on this decision. This ADR closes the language and
runtime-model question, decided explicitly on the decision track.

## Decision

- The owned correctness engine (F1/F2/F3, core-domain/0001) is written in **Rust**.
- Runtime model = **freestanding**: `no_std` on the device (org/0006 memory **Profile A** — no
  managed runtime / VM / GC resident). Choosing Rust subsumes and closes the runtime-class
  question; this ADR records **both** (no separate freestanding draft is needed).
- Scope: the **engine only**. Management-side repos keep their languages open and light, chosen
  per-repo; they must **not** be coupled to this decision.

### Binding spec (more fundamental than the language label)

- **One engine SOURCE, compiled to every target** — native for device/server, WASM for browser,
  native/WASM for Android. This is the concrete delivery mechanism for the "one core, no drift
  across surfaces" guarantee (core-domain/0008, core-domain/0009).
- **Floating-point determinism:** outputs must be tolerance-reproducible across **all** compiled
  targets. In practice this requires a **controlled/vendored math library** rather than each
  platform's `libm`, so "no drift" holds in fact, not just in slogan. This is also what lets the
  device's doubles reproduce the oracle golden vectors (org/0006
  on-device-matches-recompute acceptance test).
- **Precision floor** (carried from org/0006): freestanding = lean **footprint**, not cheap
  **math**. F1/F2 stay **double-precision**; only F3 (calendar) is **exact integer**. `no_std`
  must **not** be read as license for fixed-point in F1/F2.
- **LGPL relinkability** (core-domain/0003) is preserved via a **C-ABI FFI boundary** to the open
  KosherJava/Hebcal-class engine kept as oracle + optional alternative (engine.selection,
  core-domain/0009).

## Rationale

- **Memory safety without a GC** — exactly the freestanding gap a freestanding model opens: giving
  up the managed safety net, Rust supplies safety without a resident runtime, which C does not.
- **Single source → native AND WASM** from one codebase, directly serving one-core-no-drift
  (core-domain/0008, core-domain/0009).
- **`no_std` fits the edge envelope** (org/0006); the workload is bursty-then-idle with tiny math
  (a few dozen double-precision evals per refresh), so no rich numeric runtime is needed and a
  freestanding binary serves the **cold-start invariant** (fast, predictable boot, no VM-init on
  the bounded recovery path).
- **Mature toolchain**; a vendorable portable `libm` satisfies the FP-determinism requirement
  above.

## Alternatives considered

- **C** — principled fallback: maximal portability, tiniest footprint, trivial freestanding, and
  an existing KosherJava C port (in-language oracle/alternative). Rejected as primary because it
  lacks **memory-safety-without-GC**, which is the freestanding gap. *Tipping condition recorded:*
  C would win if portability + the existing C port + simplicity were weighted over borrow-checker
  safety.
- **Zig** — elegant for this (freestanding, comptime, strong WASM, can consume the C port) but
  **pre-1.0**; rejected — too much toolchain risk for a decade-long correctness trust anchor.
- **Managed languages (Go, JVM Java/Kotlin, C#, JS/TS)** — excluded **for the engine role** by the
  freestanding rule (GC/VM resident). They remain valid candidates for **management** repos.

## Consequences

- Closes open-decision #8's **engine-language + runtime-model**. **Serialization stays OPEN**
  (Rust pulls toward serde-friendly encodings, but the wire format remains its own undecided
  cross-repo contract — not resolved here, core-domain/0008). Management-side languages stay open
  by design.
- The **validation strategy** (neutral-oracle + legacy-characterization) is a separate item and is
  **not** recorded here.

## Related

core-domain/0001 (the engine these functions form), core-domain/0003 (open-license oracle +
relinkability), core-domain/0008 (engine posture; pluggable interface), core-domain/0009
(parameter-vector schema; engine.selection knob); org/0006 (edge envelope; runtime Profile A);
open-decision #8 (engine-language portion resolved here).
