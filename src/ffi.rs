//! C-ABI probes for the native-vs-wasm FP-determinism check (ADR core-domain/0010). These run
//! the FULL F1 read and return the emitted absolute instant as `i64` nanoseconds — the integer
//! whose EXACT cross-target equality is the reproducibility invariant. `i64::MIN` = does-not-occur.
//!
//! (This C-ABI is also the shape the device/FFI relinkability boundary of ADR core-domain/0003
//! will take; here it exists for the determinism harness.)

use crate::calendar::molad_instant;
use crate::events::{moon_rise_set, read_instant, Bound, Direction, ReadSpec};
use crate::lunar::moon_altitude_deg;
use crate::optics::RefractionModel;
use crate::params::Optics;
use crate::units::GeometricAltitude;
use crate::Site;

/// Does-not-occur sentinel for the C-ABI (Option can't cross the boundary).
pub const DOES_NOT_OCCUR: i64 = i64::MIN;

/// `kind`: 0 depression-rising · 1 depression-setting · 2 netz · 3 shkia · 4 chatzot ·
/// 5 sof-zman-shma GRA · 6 sof-zman-shma MGA(−16.1) · **7 moonrise · 8 moonset** (F2; `ref_jd` =
/// local-midnight day-start) · **9 moon apparent altitude** (F2; returns `round(deg·1e9)`) ·
/// **10 molad instant** (F3; `ref_jd`=year, `angle`=month). `angle` used by kinds 0/1/10.
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
        _ => return DOES_NOT_OCCUR,
    };
    match read_instant(&site, ref_jd, spec, &Optics::default()) {
        Some(ai) => ai.unix_nanos,
        None => DOES_NOT_OCCUR,
    }
}
