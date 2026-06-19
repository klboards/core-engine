//! klboards core-engine — F1 (solar geometry), F2 (lunar geometry), F3 (Hebrew calendar) +
//! molad/Kiddush Levana, and the F1/F2/F3 couplings (ADR core-domain/0001, /0016): day-roll,
//! day-type, full Kiddush Levana, and tal-u-matar (see `couplings` + `tekufa`).
//!
//! Freestanding engine (ADR core-domain/0010; org/0006 Profile A). The canonical artifact is
//! built `--no-default-features` → `#![no_std]`. A default `std` feature exists ONLY so the std
//! integration-test harness and the freestanding wasm artifact can coexist in one crate; the
//! engine source itself uses only `core` + `libm` and never `std`.
//!
//! **Determinism (ADR core-domain/0010):** all transcendental math is routed through the `libm`
//! crate on *every* build, native and wasm. Under `--no-default-features` (`#![no_std]`),
//! `core` has no `f64::sin/cos/atan2`, so an accidental platform-libm call fails to compile —
//! the wasm build is therefore the enforcement gate, not mere discipline.
//!
//! The engine emits **absolute, timezone-free instants** (ADR core-domain/0001, /0007). All
//! civil-time / wall-clock / civil-day labelling is an EDGE concern (harness), never here.
#![cfg_attr(not(feature = "std"), no_std)]
#![deny(unsafe_code)]
#![deny(missing_docs)]

pub mod calendar;
pub mod couplings;
pub mod daf_yomi;
pub mod events;
pub mod ffi;
pub mod geometry;
pub mod kiddush_levana;
pub mod lunar;
pub mod optics;
pub mod params;
pub mod tekufa;
pub mod time;
pub mod units;
pub mod wire;

/// Freestanding panic handler — only for the `#![no_std]` (no `std` feature) artifact, e.g. wasm
/// and the eventual device build. The std test/host build uses std's handler.
#[cfg(not(feature = "std"))]
#[panic_handler]
fn panic_handler(_: &core::panic::PanicInfo) -> ! {
    loop {}
}

/// An absolute instant: nanoseconds since the Unix epoch, UTC, timezone-free
/// (ADR core-domain/0001). This is the only kind of value F1 emits.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct AbsoluteInstant {
    /// Nanoseconds since the Unix epoch, UTC.
    pub unix_nanos: i64,
}

impl AbsoluteInstant {
    /// Convert a UT Julian Day to an absolute instant. Rounded to ns via `libm::round`
    /// (deterministic on every target).
    #[inline]
    pub fn from_julian_day(jd_utc: f64) -> Self {
        let secs = (jd_utc - 2_440_587.5) * 86_400.0;
        AbsoluteInstant {
            unix_nanos: libm::round(secs * 1.0e9) as i64,
        }
    }
}

/// Result of an F1 read. `does-not-occur` is first-class (`None`) — a depression angle never
/// reached at high latitude in summer (ADR core-domain/0009). The civil-day (+1) tag is
/// deliberately NOT carried here: it is a tz/edge label (ADR core-domain/0007) computed by the
/// rendering boundary, not by the tz-free core. (See FINDINGS in the pass report.)
pub type ZmanResult = Option<AbsoluteInstant>;

/// A site: the natural givens of ADR core-domain/0001 (φ, λ, h). Timezone-free; λ east-positive.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Site {
    /// Latitude, degrees (north-positive).
    pub lat_deg: f64,
    /// Longitude, degrees (east-positive).
    pub lon_deg: f64,
    /// Elevation above the reference ellipsoid, metres.
    pub elev_m: f64,
}
