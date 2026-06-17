//! C-ABI probes for the native-vs-wasm FP-determinism check (ADR core-domain/0010). These run
//! the FULL F1 read and return the emitted absolute instant as `i64` nanoseconds — the integer
//! whose EXACT cross-target equality is the reproducibility invariant. `i64::MIN` = does-not-occur.
//!
//! (This C-ABI is also the shape the device/FFI relinkability boundary of ADR core-domain/0003
//! will take; here it exists for the determinism harness.)

use crate::calendar::{fixed_from_hebrew, molad_instant};
use crate::couplings::{hebrew_date_at_instant, DayRoll, DEFAULT_DAY_BOUNDARY};
use crate::events::{
    moon_rise_set, read_instant, sun_effective_alt_deg, terrain_horizon_crossing, Bound, Direction,
    ReadSpec,
};
use crate::geometry::solar_azimuth_deg;
use crate::lunar::moon_altitude_deg;
use crate::optics::{LimbReference, RefractionModel};
use crate::params::{Optics, TekufaMethod};
use crate::tekufa::{tekufa_instant, Season};
use crate::units::GeometricAltitude;
use crate::wire::HorizonProfile;
use crate::{AbsoluteInstant, Site};

/// Does-not-occur sentinel for the C-ABI (Option can't cross the boundary).
pub const DOES_NOT_OCCUR: i64 = i64::MIN;

/// `kind`: 0 depression-rising · 1 depression-setting · 2 netz · 3 shkia · 4 chatzot ·
/// 5 sof-zman-shma GRA · 6 sof-zman-shma MGA(−16.1) · **7 moonrise · 8 moonset** (F2; `ref_jd` =
/// local-midnight day-start) · **9 moon apparent altitude** (F2; returns `round(deg·1e9)`) ·
/// **10 molad instant** (F3; `ref_jd`=year, `angle`=month). Phase-3 couplings (ADR core-domain/0016):
/// **11 tekufa instant Shmuel · 12 tekufa instant Rav Ada** (`ref_jd`=Hebrew year, `angle`=season
/// ordinal 0 Nisan/1 Tammuz/2 Tishrei/3 Tevet) · **13 day-roll resolved RD** (`ref_jd`=JD of the
/// instant; returns the rolled Hebrew Rata Die, or sentinel if the boundary does-not-occur) ·
/// **14 sun effective altitude** (the night predicate; returns `round(deg·1e9)`). Intake/terrain
/// (ADR core-domain/0018): **15 solar azimuth** (compass, `round(deg·1e9)`) · **16 TerrainProfile
/// crossing** (`angle`=constant skyline angle deg; synthetic profile). Read-spec vocabulary
/// (ADR core-domain/0020): **17 lower-limb netz** (the ±semidiameter limb-shift float path) ·
/// **18 fixed-minute-offset netz** (`angle`=offset minutes, fixed clock) · **19 seasonal/zmaniyos
/// minute-offset netz** (`angle`=offset minutes scaled by the netz→shkia span). `angle` used by
/// kinds 0/1/10/11/12/16/18/19.
// The crate denies unsafe_code; this single C-ABI export (the determinism-harness / future
// ADR-0003 relinkability boundary) is the one justified exception.
#[allow(unsafe_code)]
#[no_mangle]
pub extern "C" fn probe_zman_nanos(
    kind: u32,
    lat_deg: f64,
    lon_deg: f64,
    elev_m: f64,
    ref_jd: f64,
    angle_deg: f64,
) -> i64 {
    let site = Site {
        lat_deg,
        lon_deg,
        elev_m,
    };
    // F2/F3 kinds don't fit the F1 ReadSpec model — handle them up front (each is a float path the
    // /0010 native==wasm gate must cover: lunar position+parallax, the moon crossing, the molad
    // UT projection).
    match kind {
        7 => {
            return moon_rise_set(&site, ref_jd, Direction::Rising, &Optics::default())
                .map(|ai| ai.unix_nanos)
                .unwrap_or(DOES_NOT_OCCUR)
        }
        8 => {
            return moon_rise_set(&site, ref_jd, Direction::Setting, &Optics::default())
                .map(|ai| ai.unix_nanos)
                .unwrap_or(DOES_NOT_OCCUR)
        }
        9 => {
            let geo = GeometricAltitude(moon_altitude_deg(ref_jd, &site));
            let app = RefractionModel::Bennett.apparent(geo).deg();
            return libm::round(app * 1.0e9) as i64;
        }
        10 => return molad_instant(ref_jd as i32, angle_deg as u8).unix_nanos,
        11 | 12 => {
            let season = match angle_deg as i32 {
                0 => Season::Nisan,
                1 => Season::Tammuz,
                2 => Season::Tishrei,
                _ => Season::Tevet,
            };
            let method = if kind == 11 {
                TekufaMethod::Shmuel
            } else {
                TekufaMethod::RavAda
            };
            return tekufa_instant(ref_jd as i32, season, method).unix_nanos;
        }
        13 => {
            let t = AbsoluteInstant::from_julian_day(ref_jd);
            return match hebrew_date_at_instant(t, &site, DEFAULT_DAY_BOUNDARY, &Optics::default())
            {
                DayRoll::Resolved(d) => fixed_from_hebrew(d).0,
                DayRoll::BoundaryDoesNotOccur => DOES_NOT_OCCUR,
            };
        }
        14 => {
            return libm::round(sun_effective_alt_deg(ref_jd, &site, &Optics::default()) * 1.0e9)
                as i64
        }
        15 => return libm::round(solar_azimuth_deg(ref_jd, &site) * 1.0e9) as i64,
        16 => {
            // TerrainProfile crossing (the /0018 moat float path: azimuth + per-azimuth angle +
            // dynamic-target scan/bisect) against a synthetic constant-angle profile (`angle_deg`).
            let mam = libm::round(angle_deg * 60_000.0) as i32;
            let b = mam.to_le_bytes();
            let mut buf = [0u8; 16]; // 4 samples × 4 bytes (LE i32 milliarcminutes)
            let mut s = 0;
            while s < 4 {
                buf[s * 4] = b[0];
                buf[s * 4 + 1] = b[1];
                buf[s * 4 + 2] = b[2];
                buf[s * 4 + 3] = b[3];
                s += 1;
            }
            let hp = HorizonProfile {
                lat_microdeg: 0,
                lon_microdeg: 0,
                elev_mm: 0,
                dem_source: 0,
                dem_version: 0,
                prov_refraction_model: 0,
                prov_refraction_coeff_micro: None,
                angles_mam: &buf,
            };
            return terrain_horizon_crossing(
                &site,
                ref_jd,
                Direction::Rising,
                &Optics::default(),
                &hp,
            )
            .map(|ai| ai.unix_nanos)
            .unwrap_or(DOES_NOT_OCCUR);
        }
        17 => {
            // Lower-limb netz (/0020): the whole-disc-up reference — a +2·semidiameter target shift.
            let optics = Optics {
                limb: LimbReference::Lower,
                ..Optics::default()
            };
            return read_instant(
                &site,
                ref_jd,
                ReadSpec::HorizonCrossing {
                    dir: Direction::Rising,
                },
                &optics,
            )
            .map(|ai| ai.unix_nanos)
            .unwrap_or(DOES_NOT_OCCUR);
        }
        _ => {}
    }
    let spec = match kind {
        0 => ReadSpec::DepressionAngle {
            angle_deg,
            dir: Direction::Rising,
        },
        1 => ReadSpec::DepressionAngle {
            angle_deg,
            dir: Direction::Setting,
        },
        2 => ReadSpec::HorizonCrossing {
            dir: Direction::Rising,
        },
        3 => ReadSpec::HorizonCrossing {
            dir: Direction::Setting,
        },
        4 => ReadSpec::ExtremumMidpoint,
        5 => ReadSpec::Proportional {
            fraction: 3.0 / 12.0,
            start: Bound::Netz,
            end: Bound::Shkia,
        },
        6 => ReadSpec::Proportional {
            fraction: 3.0 / 12.0,
            start: Bound::Depression {
                angle_deg: 16.1,
                dir: Direction::Rising,
            },
            end: Bound::Depression {
                angle_deg: 16.1,
                dir: Direction::Setting,
            },
        },
        18 => ReadSpec::FixedMinuteOffset {
            base: Bound::Netz,
            offset_min: angle_deg, // fixed clock minutes
            seasonal: None,
        },
        19 => ReadSpec::FixedMinuteOffset {
            base: Bound::Netz,
            offset_min: angle_deg, // zmaniyos minutes over the netz→shkia span
            seasonal: Some((Bound::Netz, Bound::Shkia)),
        },
        _ => return DOES_NOT_OCCUR,
    };
    match read_instant(&site, ref_jd, spec, &Optics::default()) {
        Some(ai) => ai.unix_nanos,
        None => DOES_NOT_OCCUR,
    }
}
