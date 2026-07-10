//! Refraction & horizon optics (ADR core-domain/0006, refined by /0013). Oracle-measured:
//! near the horizon, Bennett/Saemundsson-class refraction matches Wolfram to ≤0.035°; the
//! sunrise/sunset event is the **apparent** sun-centre at −(semidiameter + horizon dip).
//!
//! Knobs (serializable enums → no_std, alloc-free, CBOR-ready per /0011 — "core resolves none"):
//! the refraction model and the horizon mode are *parameters*, not hard-coded.

use crate::units::{ApparentAltitude, GeometricAltitude};
use libm::{acos, tan};

const DEG: f64 = core::f64::consts::PI / 180.0;

/// Atmospheric-refraction model (a knob; ADR core-domain/0006 Choice A = standard-atmospheric).
/// `None` = airless geometry (the depression-shita default, /0013).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum RefractionModel {
    /// No refraction (airless geometry) — the depression-shita default.
    None,
    /// Saemundsson (1986): refraction as a function of **true/geometric** altitude — no iteration.
    Saemundsson,
    /// Bennett (1982): refraction as a function of **apparent** altitude. Applied here by a
    /// two-step fixed-point from the geometric altitude. Best match to the oracle near and just
    /// below the horizon (ADR core-domain/0013 Phase-0 measurement: ≤0.035° to Wolfram).
    Bennett,
}

impl RefractionModel {
    /// Bennett refraction (degrees) for an **apparent** altitude `h_a` (degrees).
    #[inline]
    fn bennett_deg(h_a: f64) -> f64 {
        1.0 / tan((h_a + 7.31 / (h_a + 4.4)) * DEG) / 60.0
    }

    /// Refraction (degrees) to ADD to a geometric altitude to get apparent. 0 for `None`.
    #[inline]
    pub fn refraction_deg(self, geo: GeometricAltitude) -> f64 {
        let h = geo.deg();
        match self {
            RefractionModel::None => 0.0,
            RefractionModel::Saemundsson => 1.02 / tan((h + 10.3 / (h + 5.11)) * DEG) / 60.0,
            RefractionModel::Bennett => {
                // Bennett takes apparent altitude; recover it from geometric by a 2-step
                // fixed point (apparent ≈ geometric + R(apparent)).
                let a1 = h + Self::bennett_deg(h);
                Self::bennett_deg(h + Self::bennett_deg(a1))
            }
        }
    }

    /// `apparent = geometric + refraction(geometric)`.
    #[inline]
    pub fn apparent(self, geo: GeometricAltitude) -> ApparentAltitude {
        ApparentAltitude(geo.deg() + self.refraction_deg(geo))
    }
}

/// Halachic horizon convention (a knob): sea-level (mishor) vs visible (elevation dip) vs the
/// provisioned terrain skyline (ADR core-domain/0004, future).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum HorizonMode {
    /// Idealized sea-level horizon — elevation ignored.
    Mishor,
    /// Visible horizon — geometric dip from observer elevation applied in-core (/0013).
    Visible,
    /// Terrain skyline composed from a provisioned `(azimuth→angle)` profile (/0004). Live via
    /// `events::read_instant_with_horizon` when a `HorizonProfile` is supplied (/0018).
    TerrainProfile,
}

/// Which edge of the sun's disc defines the rise/set event (`solar.limb_reference`, spec §1.B) — the
/// netz-definition dispute. Upper limb = the sun first appears (majority; Biur Halacha 58); lower limb
/// = the whole disc has risen / "end of ascent" (Ish Matzliach; Yalkut Yosef 89:3); center = disc
/// centre. A `±semidiameter` shift on the horizon target — a convention-free *mechanism* knob (the
/// core resolves none; ADR core-domain/0020). Default `Upper` reproduces the pre-0020 behaviour.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum LimbReference {
    /// Upper limb tangent to the horizon — the sun first appears (default).
    #[default]
    Upper,
    /// Disc centre at the horizon.
    Center,
    /// Lower limb tangent — the whole disc has risen (end of ascent).
    Lower,
}

impl LimbReference {
    /// Semidiameter multiplier in the horizon target `−(factor·sd + dip)`: Upper +1, Center 0, Lower −1.
    #[inline]
    pub fn semidiameter_factor(self) -> f64 {
        match self {
            LimbReference::Upper => 1.0,
            LimbReference::Center => 0.0,
            LimbReference::Lower => -1.0,
        }
    }
}

/// Apparent solar radius (degrees), ~16′. Distance-based refinement is a TODO; fixed value is
/// within the ±1-min bar.
#[inline]
pub fn semidiameter_deg() -> f64 {
    16.0 / 60.0
}

/// Geometric dip of the horizon (degrees) from observer elevation: `acos(Rₑ/(Rₑ+h))`.
#[inline]
pub fn geometric_dip_deg(elev_m: f64) -> f64 {
    const RE: f64 = 6_371_008.8; // mean Earth radius (m)
    if elev_m <= 0.0 {
        0.0
    } else {
        acos(RE / (RE + elev_m)) / DEG
    }
}

/// Horizon dip (degrees) selected by the mode — 0 for Mishor, the elevation dip otherwise.
/// TerrainProfile falls back to the elevation dip ONLY on this scalar path (no profile supplied);
/// when a `HorizonProfile` is given, `events::terrain_horizon_crossing` bypasses this fn entirely.
#[inline]
fn horizon_dip_deg(mode: HorizonMode, elev_m: f64) -> f64 {
    match mode {
        HorizonMode::Mishor => 0.0,
        HorizonMode::Visible | HorizonMode::TerrainProfile => geometric_dip_deg(elev_m),
    }
}

/// Apparent body-centre altitude (degrees, negative) at the rise/set event for a body of the given
/// apparent `semidiameter_deg`: `−(semidiameter + dip)` per the horizon mode. Refraction is added
/// separately (by `events::effective_alt_deg`), so this is the *geometric* target the apparent limb
/// reaches the (dipped) horizon at. Shared by the Sun (F1) and Moon (F2).
#[inline]
pub fn horizon_target_deg(
    mode: HorizonMode,
    elev_m: f64,
    semidiameter_deg: f64,
    limb: LimbReference,
) -> f64 {
    -(limb.semidiameter_factor() * semidiameter_deg + horizon_dip_deg(mode, elev_m))
}

/// Apparent sun-centre altitude (degrees, negative) at the sunrise/sunset event:
/// `−(limb_sign·semidiameter + dip)` per the horizon mode + limb (ADR core-domain/0013/0020).
#[inline]
pub fn horizon_apparent_target_deg(mode: HorizonMode, elev_m: f64, limb: LimbReference) -> f64 {
    horizon_target_deg(mode, elev_m, semidiameter_deg(), limb)
}
