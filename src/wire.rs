//! Intake boundary (ADR core-domain/0011 + /0018): the `no_std` CBOR **reader** for the two
//! cross-repo artifacts — the **parameter vector** (knobs) and the **horizon profile** (terrain
//! skyline). The *writer* lives in provisioning-builder, not here. Decoding is via `minicbor`
//! (no_std, **no heap**); all numeric wire fields are **fixed-point integers** (float-free wire →
//! trivially deterministic, CDE / RFC 8949 §4.2.1), converted to `f64` on read. COSE_Sign1
//! verification is **deferred** (coupled to org/0006 §7 root-of-trust) — this layer decodes only.
//!
//! Contract shape: integer-keyed CBOR maps; see `docs/spec/parameter-vector.cddl` +
//! `docs/spec/horizon-profile.cddl`.

use crate::optics::{HorizonMode, RefractionModel};
use crate::params::{
    AdarAnniversaryRule, KiddushLevanaEnd, KiddushLevanaStart, Optics, Realm, TalUmatarBasis,
    TekufaMethod,
};
use crate::Site;
use libm::{fabs, floor};
use minicbor::{Decode, Encode};

/// Milliarcminutes per degree (1 mam = 1/1000 arcminute = 1/60000°). The fixed-point unit for the
/// packed horizon-angle array — ~0.06″ resolution, far below the ±1-min/arc-second bar (ADR-0018).
const MAM_PER_DEG: f64 = 60_000.0;
/// Binding tolerances (φ/λ in degrees, h in metres) for matching a profile to a site (0004/0006).
const BIND_TOL_DEG: f64 = 1.0e-3;
const BIND_TOL_M: f64 = 5.0;

/// The contract version this build understands.
pub const SCHEMA_VERSION: u16 = 1;

/// A typed decode failure — never a panic (the /0017 no-panic invariant extends to the reader).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum DecodeError {
    /// Malformed or non-conforming CBOR (incl. missing required fields).
    Cbor,
    /// `schema_version` not understood by this build.
    Schema,
    /// `engine_selection` is not the owned-primary engine (the only one shipping).
    Engine,
    /// An enum discriminant is out of range for a knob.
    Range,
    /// A knob value names a real spec option this build does not yet implement (flagged, not faked).
    Unimplemented,
    /// Horizon profile binding metadata `(φ, λ, h)` does not match the site (0004/0006 invariant).
    Binding,
}

/// The decoded **parameter vector** (ADR core-domain/0009 §1, integer-keyed CBOR map). Fields are the
/// raw wire discriminants; `resolve_*` maps them to the engine's typed knobs. This retires
/// `Optics::default()` as the *only* input path — knobs now come from a decoded, signable artifact.
#[derive(Clone, Debug, Decode, Encode)]
#[cbor(map)]
pub struct ParameterVector {
    /// Contract version (§1.A `schema.version`).
    #[n(0)]
    pub schema_version: u16,
    /// Engine selection (§1.A `engine.selection`); 0 = owned-primary (only engine shipping).
    #[n(1)]
    pub engine_selection: u8,
    /// Horizon mode (§1.B): 0 sea-level (Mishor), 1 visible (in-core dip), 2 terrain-profile.
    #[n(2)]
    pub horizon_mode: u8,
    /// Refraction model (§1.B): 0 standard-atmospheric, 1 meeus-noaa, 2 halachic-fixed-coefficient.
    #[n(3)]
    pub refraction_model: u8,
    /// Fixed refraction coefficient, micro-arcminutes (§1.B; required only for halachic-fixed).
    #[n(4)]
    pub refraction_coeff_micro: Option<i32>,
    /// Solar position reference (§1.B): 0 apparent, 1 geometric.
    #[n(5)]
    pub solar_position_reference: u8,
    /// Solar limb reference (§1.B): 0 upper, 1 center, 2 lower.
    #[n(6)]
    pub solar_limb_reference: u8,
    /// Realm (§1.D): 0 Eretz-Yisrael, 1 diaspora.
    #[n(7)]
    pub locale_realm: u8,
    /// tal-u-matar basis (§1.D): 0 tekufa-based, 1 fixed-7-cheshvan.
    #[n(8)]
    pub tal_umatar_basis: u8,
    /// Tekufa method (§1.D): 0 Shmuel, 1 Rav-Ada.
    #[n(9)]
    pub tekufa_method: u8,
    /// Adar anniversary rule (§1.D): 0 Adar II, 1 Adar I, 2 both.
    #[n(10)]
    pub adar_anniversary_rule: u8,
    /// Kiddush Levana start (§1.D/0015): 0 three-days, 1 seven-days, 2 molad.
    #[n(11)]
    pub kiddush_levana_start: u8,
    /// Kiddush Levana end (§1.D/0015): 0 half-month, 1 fifteen-days.
    #[n(12)]
    pub kiddush_levana_end: u8,
    /// Rounding stringency (§1.E): 0 lehakel, 1 lehachmir, 2 nearest, 3 truncate.
    #[n(13)]
    pub rounding_stringency: u8,
    /// Rounding granularity (§1.E): 0 second, 1 minute.
    #[n(14)]
    pub rounding_granularity: u8,
}

/// Decode + validate a parameter vector. Returns a typed [`DecodeError`]; never panics.
pub fn decode_parameter_vector(bytes: &[u8]) -> Result<ParameterVector, DecodeError> {
    let pv: ParameterVector = minicbor::decode(bytes).map_err(|_| DecodeError::Cbor)?;
    if pv.schema_version != SCHEMA_VERSION {
        return Err(DecodeError::Schema);
    }
    if pv.engine_selection != 0 {
        return Err(DecodeError::Engine); // alternative engines not yet present (ADR-0008)
    }
    Ok(pv)
}

impl ParameterVector {
    /// Resolve the optics sub-vector. The spec's high-level `refraction.model` maps to the engine's
    /// per-read [`RefractionModel`] pair (ADR-0013): `standard-atmospheric` → apparent horizon
    /// (Bennett) + geometric depression (None). FINDING (ADR-0018): `meeus-noaa` and
    /// `halachic-fixed-coefficient` are real spec options the engine does not yet implement →
    /// `Unimplemented` (not silently substituted). `horizon_mode` maps 0/1/2 → Mishor/Visible/
    /// TerrainProfile (the spec §1.B enumerates only sea-level/terrain-profile — `visible`, added in
    /// /0013, is the third mode; flagged for a spec update).
    pub fn resolve_optics(&self) -> Result<Optics, DecodeError> {
        let horizon_mode = match self.horizon_mode {
            0 => HorizonMode::Mishor,
            1 => HorizonMode::Visible,
            2 => HorizonMode::TerrainProfile,
            _ => return Err(DecodeError::Range),
        };
        let (horizon_refraction, depression_refraction) = match self.refraction_model {
            0 => (RefractionModel::Bennett, RefractionModel::None), // standard-atmospheric (/0013)
            1 | 2 => return Err(DecodeError::Unimplemented),        // meeus-noaa / halachic-fixed
            _ => return Err(DecodeError::Range),
        };
        Ok(Optics {
            horizon_refraction,
            depression_refraction,
            horizon_mode,
        })
    }

    /// Realm (coupling #2/#4 gate).
    pub fn realm(&self) -> Result<Realm, DecodeError> {
        match self.locale_realm {
            0 => Ok(Realm::EretzYisrael),
            1 => Ok(Realm::Diaspora),
            _ => Err(DecodeError::Range),
        }
    }
    /// tal-u-matar basis.
    pub fn tal_umatar_basis(&self) -> Result<TalUmatarBasis, DecodeError> {
        match self.tal_umatar_basis {
            0 => Ok(TalUmatarBasis::TekufaBased),
            1 => Ok(TalUmatarBasis::Fixed7Cheshvan),
            _ => Err(DecodeError::Range),
        }
    }
    /// Tekufa method.
    pub fn tekufa_method(&self) -> Result<TekufaMethod, DecodeError> {
        match self.tekufa_method {
            0 => Ok(TekufaMethod::Shmuel),
            1 => Ok(TekufaMethod::RavAda),
            _ => Err(DecodeError::Range),
        }
    }
    /// Adar anniversary rule.
    pub fn adar_anniversary_rule(&self) -> Result<AdarAnniversaryRule, DecodeError> {
        match self.adar_anniversary_rule {
            0 => Ok(AdarAnniversaryRule::AdarII),
            1 => Ok(AdarAnniversaryRule::AdarI),
            2 => Ok(AdarAnniversaryRule::Both),
            _ => Err(DecodeError::Range),
        }
    }
    /// Kiddush Levana window knobs.
    pub fn kiddush_levana(&self) -> Result<(KiddushLevanaStart, KiddushLevanaEnd), DecodeError> {
        let start = match self.kiddush_levana_start {
            0 => KiddushLevanaStart::ThreeDays,
            1 => KiddushLevanaStart::SevenDays,
            2 => KiddushLevanaStart::Molad,
            _ => return Err(DecodeError::Range),
        };
        let end = match self.kiddush_levana_end {
            0 => KiddushLevanaEnd::HalfMonth,
            1 => KiddushLevanaEnd::FifteenDays,
            _ => return Err(DecodeError::Range),
        };
        Ok((start, end))
    }
    /// Validate the currently-fixed-behaviour knobs (ADR-0018 findings): the engine's apparent/
    /// geometric split and upper-limb reference are baked per /0013, so the parameter-vector values
    /// must match — `upper-limb` (0) and `apparent` (0) — else the request names behaviour the engine
    /// does not yet vary. Surfaced, not silently ignored.
    pub fn check_fixed_behaviour(&self) -> Result<(), DecodeError> {
        if self.solar_limb_reference != 0 {
            return Err(DecodeError::Unimplemented); // center/lower-limb not yet a knob
        }
        if self.solar_position_reference != 0 {
            return Err(DecodeError::Unimplemented); // geometric-everywhere not yet a knob
        }
        Ok(())
    }
}

/// The decoded **horizon profile** (ADR core-domain/0004 + /0011, integer-keyed CBOR map): binding
/// metadata + a packed azimuth→angle array. The angle array is a borrowed CBOR **byte string** of
/// little-endian `i32` **milliarcminutes**, evenly spaced over `[0°, 360°)` — zero-copy (no heap),
/// so `HorizonProfile` borrows the input bytes.
#[derive(Clone, Debug, Decode, Encode)]
#[cbor(map)]
pub struct HorizonProfile<'b> {
    /// Bound latitude, microdegrees (north-positive).
    #[n(0)]
    pub lat_microdeg: i32,
    /// Bound longitude, microdegrees (east-positive).
    #[n(1)]
    pub lon_microdeg: i32,
    /// Bound elevation, millimetres.
    #[n(2)]
    pub elev_mm: i32,
    /// DEM source id (e.g. Copernicus GLO-30, USGS 3DEP …) — a registered enum, not a string.
    #[n(3)]
    pub dem_source: u16,
    /// DEM dataset version.
    #[n(4)]
    pub dem_version: u32,
    /// Refraction model used at provisioning (0/1/2 as in the parameter vector) — the 0004/0006
    /// provisioning↔runtime invariant requires the runtime to use the same model.
    #[n(5)]
    pub prov_refraction_model: u8,
    /// Fixed refraction coefficient used at provisioning, micro-arcminutes (if model = halachic-fixed).
    #[n(6)]
    pub prov_refraction_coeff_micro: Option<i32>,
    /// Packed azimuth→horizon-angle array: little-endian `i32` milliarcminutes, `len % 4 == 0`,
    /// `n = len/4` samples evenly spaced over `[0°, 360°)`. Borrowed (no heap).
    #[n(7)]
    #[cbor(with = "minicbor::bytes")]
    pub angles_mam: &'b [u8],
}

/// Decode + validate a horizon profile. Requires `prov_refraction_model` in range and a non-empty
/// 4-byte-aligned angle array; typed error otherwise. Never panics.
pub fn decode_horizon_profile(bytes: &[u8]) -> Result<HorizonProfile<'_>, DecodeError> {
    let hp: HorizonProfile = minicbor::decode(bytes).map_err(|_| DecodeError::Cbor)?;
    if hp.prov_refraction_model > 2 {
        return Err(DecodeError::Range);
    }
    if hp.angles_mam.is_empty() || !hp.angles_mam.len().is_multiple_of(4) {
        return Err(DecodeError::Cbor);
    }
    Ok(hp)
}

impl HorizonProfile<'_> {
    /// Number of azimuth samples.
    pub fn sample_count(&self) -> usize {
        self.angles_mam.len() / 4
    }

    fn angle_mam(&self, i: usize) -> i32 {
        let o = i * 4;
        i32::from_le_bytes([
            self.angles_mam[o],
            self.angles_mam[o + 1],
            self.angles_mam[o + 2],
            self.angles_mam[o + 3],
        ])
    }

    /// Horizon angle (degrees) at azimuth `az_deg`, by linear interpolation between the two nearest
    /// evenly-spaced samples with 360° wraparound. `sample_count()` is guaranteed ≥ 1 by the decoder.
    pub fn horizon_angle_deg_at(&self, az_deg: f64) -> f64 {
        let n = self.sample_count();
        // no_std: f64::rem_euclid is std-only — fold the angle into [0,360) by hand.
        let mut az = az_deg % 360.0;
        if az < 0.0 {
            az += 360.0;
        }
        let x = az / 360.0 * n as f64;
        let i0 = (floor(x) as usize) % n;
        let i1 = (i0 + 1) % n;
        let frac = x - floor(x);
        let a0 = self.angle_mam(i0) as f64;
        let a1 = self.angle_mam(i1) as f64;
        (a0 + (a1 - a0) * frac) / MAM_PER_DEG
    }

    /// Enforce the provisioning↔runtime binding (0004/0006): the profile's `(φ, λ, h)` must match the
    /// site within tolerance, else it is the wrong profile for this site — a typed [`DecodeError`].
    pub fn check_binding(&self, site: &Site) -> Result<(), DecodeError> {
        let dlat = fabs(self.lat_microdeg as f64 / 1.0e6 - site.lat_deg);
        let dlon = fabs(self.lon_microdeg as f64 / 1.0e6 - site.lon_deg);
        let delev = fabs(self.elev_mm as f64 / 1.0e3 - site.elev_m);
        if dlat <= BIND_TOL_DEG && dlon <= BIND_TOL_DEG && delev <= BIND_TOL_M {
            Ok(())
        } else {
            Err(DecodeError::Binding)
        }
    }
}
