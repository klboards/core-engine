//! Refraction & horizon optics (ADR core-domain/0006). Standard-atmospheric model
//! (Bennett-class) is the ratified default (Choice A, ADR core-domain/0009). Composed ONLY by
//! the `HorizonCrossing` read; depression-angle reads are refraction-independent geometry.
//!
//! LIMB (`solar.limb_reference`, ADR core-domain/0009) — RESOLVED by the fixture README
//! (2026-06-16): the oracle uses **sun CENTER** (Wolfram Sunrise/Sunset), with **no** separate
//! upper-limb / solar-semidiameter term. This module's center + refraction approach therefore
//! matches the oracle's limb convention. (The dip *magnitude* is a separate question — see the
//! horizon-crossing diagnostic in the pass report.)

use libm::{acos, tan};

const DEG: f64 = core::f64::consts::PI / 180.0;

/// Bennett (1982) refraction in arcminutes for apparent altitude `h_a` (degrees).
#[inline]
pub fn bennett_refraction_arcmin(h_a_deg: f64) -> f64 {
    1.0 / tan((h_a_deg + 7.31 / (h_a_deg + 4.4)) * DEG)
}

/// Standard-atmospheric refraction at the apparent horizon (h_a = 0), in degrees (~0.575°).
#[inline]
pub fn refraction_at_horizon_deg() -> f64 {
    bennett_refraction_arcmin(0.0) / 60.0
}

/// Geometric dip of the horizon (degrees) due to observer elevation `elev_m` above the
/// reference ellipsoid: `acos(Re / (Re + h))`. "Geometric horizon dip" per the build spec —
/// this is the sea-level + elevation-dip path, NOT the terrain-profile/DTM path (out of scope).
#[inline]
pub fn geometric_dip_deg(elev_m: f64) -> f64 {
    const RE: f64 = 6_371_008.8; // mean Earth radius (m)
    if elev_m <= 0.0 {
        return 0.0;
    }
    acos(RE / (RE + elev_m)) / DEG
}

/// The standard sunrise/sunset depression: sun **center** 50′ (0.8333°) below the horizon —
/// 34′ refraction + 16′ apparent solar radius bundled into one convention (Wolfram
/// `ReferenceAltitude` default; USNO standard). This is the ratified horizon-event baseline
/// (ADR core-domain/0012, refining 0006); it is the engine's `reference_altitude` knob value.
#[inline]
pub fn standard_center_depression_deg() -> f64 {
    50.0 / 60.0
}

/// Target geometric altitude (degrees, negative) for the horizon-crossing event: the **50′
/// sun-center sea-level** standard (ADR core-domain/0012). Elevation is intentionally NOT applied
/// here: accurate elevated/terrain horizons are the **core-domain/0004 horizon-profile path**
/// (composed on-device from a provisioned `(azimuth→angle)` profile), not a core point-dip. The
/// `_elev_m` argument is kept for interface stability; `geometric_dip_deg` below remains a flagged
/// interim helper for that future profile-composition path, not the oracle-matched elevated model.
#[inline]
pub fn horizon_crossing_target_deg(_elev_m: f64) -> f64 {
    -standard_center_depression_deg()
}
