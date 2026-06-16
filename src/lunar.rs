//! F2 lunar geometry — Meeus *Astronomical Algorithms* ch. 47 (abridged ELP-2000/82) →
//! **topocentric** geometric `moon_altitude_deg(t)`. All trig via `libm` (ADR core-domain/0010
//! determinism anchor). Returns the **topocentric geometric** altitude in degrees (no refraction,
//! no horizon dip — those compose in `events`/`optics` per the ADR core-domain/0006 seam).
//!
//! Topocentric parallax is **mandatory**: the Moon's geocentric vs topocentric altitude differs by
//! up to ~1° (≈ 2 min of rise/set time), far outside the ±1-min bar — so the observer-parallax
//! reduction (Meeus ch. 40) is applied here, not optional.

use crate::Site;
use libm::{asin, atan, atan2, cos, sin, tan};

const DEG: f64 = core::f64::consts::PI / 180.0;

/// TT − UT (seconds). PROVISIONAL constant for 2026 (≈ +69 s) — shared posture with `geometry.rs`;
/// a time-varying ΔT model/table is a TODO (ADR core-domain/0012, /0003, /0007). The Moon's position
/// is computed at TT = UT + ΔT, Earth-rotation (GMST) on UT.
const DELTA_T_SECS: f64 = 69.0;

/// Earth equatorial radius (km) — for the equatorial horizontal parallax sin π = 6378.14/Δ.
const EARTH_EQ_RADIUS_KM: f64 = 6378.14;

#[inline]
fn norm360(mut x: f64) -> f64 {
    x %= 360.0;
    if x < 0.0 {
        x += 360.0;
    }
    x
}

#[inline]
fn clamp(x: f64, lo: f64, hi: f64) -> f64 {
    if x < lo {
        lo
    } else if x > hi {
        hi
    } else {
        x
    }
}

/// A periodic term of Meeus table 47.A (longitude Σl in 1e-6°, distance Σr in 1e-3 km).
struct TermLR {
    d: i32,
    m: i32,
    mp: i32,
    f: i32,
    sl: i32,
    sr: i32,
}
/// A periodic term of Meeus table 47.B (latitude Σb in 1e-6°).
struct TermB {
    d: i32,
    m: i32,
    mp: i32,
    f: i32,
    sb: i32,
}

/// Meeus table 47.A — Moon longitude (Σl) and distance (Σr). Terms with the Sun's mean anomaly `M`
/// are scaled by `E^|M|` (eccentricity correction).
const TBL_LR: &[TermLR] = &[
    TermLR {
        d: 0,
        m: 0,
        mp: 1,
        f: 0,
        sl: 6_288_774,
        sr: -20_905_355,
    },
    TermLR {
        d: 2,
        m: 0,
        mp: -1,
        f: 0,
        sl: 1_274_027,
        sr: -3_699_111,
    },
    TermLR {
        d: 2,
        m: 0,
        mp: 0,
        f: 0,
        sl: 658_314,
        sr: -2_955_968,
    },
    TermLR {
        d: 0,
        m: 0,
        mp: 2,
        f: 0,
        sl: 213_618,
        sr: -569_925,
    },
    TermLR {
        d: 0,
        m: 1,
        mp: 0,
        f: 0,
        sl: -185_116,
        sr: 48_888,
    },
    TermLR {
        d: 0,
        m: 0,
        mp: 0,
        f: 2,
        sl: -114_332,
        sr: -3_149,
    },
    TermLR {
        d: 2,
        m: 0,
        mp: -2,
        f: 0,
        sl: 58_793,
        sr: 246_158,
    },
    TermLR {
        d: 2,
        m: -1,
        mp: -1,
        f: 0,
        sl: 57_066,
        sr: -152_138,
    },
    TermLR {
        d: 2,
        m: 0,
        mp: 1,
        f: 0,
        sl: 53_322,
        sr: -170_733,
    },
    TermLR {
        d: 2,
        m: -1,
        mp: 0,
        f: 0,
        sl: 45_758,
        sr: -204_586,
    },
    TermLR {
        d: 0,
        m: 1,
        mp: -1,
        f: 0,
        sl: -40_923,
        sr: -129_620,
    },
    TermLR {
        d: 1,
        m: 0,
        mp: 0,
        f: 0,
        sl: -34_720,
        sr: 108_743,
    },
    TermLR {
        d: 0,
        m: 1,
        mp: 1,
        f: 0,
        sl: -30_383,
        sr: 104_755,
    },
    TermLR {
        d: 2,
        m: 0,
        mp: 0,
        f: -2,
        sl: 15_327,
        sr: 10_321,
    },
    TermLR {
        d: 0,
        m: 0,
        mp: 1,
        f: 2,
        sl: -12_528,
        sr: 0,
    },
    TermLR {
        d: 0,
        m: 0,
        mp: 1,
        f: -2,
        sl: 10_980,
        sr: 79_661,
    },
    TermLR {
        d: 4,
        m: 0,
        mp: -1,
        f: 0,
        sl: 10_675,
        sr: -34_782,
    },
    TermLR {
        d: 0,
        m: 0,
        mp: 3,
        f: 0,
        sl: 10_034,
        sr: -23_210,
    },
    TermLR {
        d: 4,
        m: 0,
        mp: -2,
        f: 0,
        sl: 8_548,
        sr: -21_636,
    },
    TermLR {
        d: 2,
        m: 1,
        mp: -1,
        f: 0,
        sl: -7_888,
        sr: 24_208,
    },
    TermLR {
        d: 2,
        m: 1,
        mp: 0,
        f: 0,
        sl: -6_766,
        sr: 30_824,
    },
    TermLR {
        d: 1,
        m: 0,
        mp: -1,
        f: 0,
        sl: -5_163,
        sr: -8_379,
    },
    TermLR {
        d: 1,
        m: 1,
        mp: 0,
        f: 0,
        sl: 4_987,
        sr: -16_675,
    },
    TermLR {
        d: 2,
        m: -1,
        mp: 1,
        f: 0,
        sl: 4_036,
        sr: -12_831,
    },
    TermLR {
        d: 2,
        m: 0,
        mp: 2,
        f: 0,
        sl: 3_994,
        sr: -10_445,
    },
    TermLR {
        d: 4,
        m: 0,
        mp: 0,
        f: 0,
        sl: 3_861,
        sr: -11_650,
    },
    TermLR {
        d: 2,
        m: 0,
        mp: -3,
        f: 0,
        sl: 3_665,
        sr: 14_403,
    },
    TermLR {
        d: 0,
        m: 1,
        mp: -2,
        f: 0,
        sl: -2_689,
        sr: -7_003,
    },
    TermLR {
        d: 2,
        m: 0,
        mp: -1,
        f: 2,
        sl: -2_602,
        sr: 0,
    },
    TermLR {
        d: 2,
        m: -1,
        mp: -2,
        f: 0,
        sl: 2_390,
        sr: 10_056,
    },
    TermLR {
        d: 1,
        m: 0,
        mp: 1,
        f: 0,
        sl: -2_348,
        sr: 6_322,
    },
    TermLR {
        d: 2,
        m: -2,
        mp: 0,
        f: 0,
        sl: 2_236,
        sr: -9_884,
    },
    TermLR {
        d: 0,
        m: 1,
        mp: 2,
        f: 0,
        sl: -2_120,
        sr: 5_751,
    },
    TermLR {
        d: 0,
        m: 2,
        mp: 0,
        f: 0,
        sl: -2_069,
        sr: 0,
    },
    TermLR {
        d: 2,
        m: -2,
        mp: -1,
        f: 0,
        sl: 2_048,
        sr: -4_950,
    },
    TermLR {
        d: 2,
        m: 0,
        mp: 1,
        f: -2,
        sl: -1_773,
        sr: 4_130,
    },
    TermLR {
        d: 2,
        m: 0,
        mp: 0,
        f: 2,
        sl: -1_595,
        sr: 0,
    },
    TermLR {
        d: 4,
        m: -1,
        mp: -1,
        f: 0,
        sl: 1_215,
        sr: -3_958,
    },
    TermLR {
        d: 0,
        m: 0,
        mp: 2,
        f: 2,
        sl: -1_110,
        sr: 0,
    },
    TermLR {
        d: 3,
        m: 0,
        mp: -1,
        f: 0,
        sl: -892,
        sr: 3_258,
    },
    TermLR {
        d: 2,
        m: 1,
        mp: 1,
        f: 0,
        sl: -810,
        sr: 2_616,
    },
    TermLR {
        d: 4,
        m: -1,
        mp: -2,
        f: 0,
        sl: 759,
        sr: -1_897,
    },
    TermLR {
        d: 0,
        m: 2,
        mp: -1,
        f: 0,
        sl: -713,
        sr: -2_117,
    },
    TermLR {
        d: 2,
        m: 2,
        mp: -1,
        f: 0,
        sl: -700,
        sr: 2_354,
    },
    TermLR {
        d: 2,
        m: 1,
        mp: -2,
        f: 0,
        sl: 691,
        sr: 0,
    },
    TermLR {
        d: 2,
        m: -1,
        mp: 0,
        f: -2,
        sl: 596,
        sr: 0,
    },
    TermLR {
        d: 4,
        m: 0,
        mp: 1,
        f: 0,
        sl: 549,
        sr: -1_423,
    },
    TermLR {
        d: 0,
        m: 0,
        mp: 4,
        f: 0,
        sl: 537,
        sr: -1_117,
    },
    TermLR {
        d: 4,
        m: -1,
        mp: 0,
        f: 0,
        sl: 520,
        sr: -1_571,
    },
    TermLR {
        d: 1,
        m: 0,
        mp: -2,
        f: 0,
        sl: -487,
        sr: -1_739,
    },
    TermLR {
        d: 2,
        m: 1,
        mp: 0,
        f: -2,
        sl: -399,
        sr: 0,
    },
    TermLR {
        d: 0,
        m: 0,
        mp: 2,
        f: -2,
        sl: -381,
        sr: -4_421,
    },
    TermLR {
        d: 1,
        m: 1,
        mp: 1,
        f: 0,
        sl: 351,
        sr: 0,
    },
    TermLR {
        d: 3,
        m: 0,
        mp: -2,
        f: 0,
        sl: -340,
        sr: 0,
    },
    TermLR {
        d: 4,
        m: 0,
        mp: -3,
        f: 0,
        sl: 330,
        sr: 0,
    },
    TermLR {
        d: 2,
        m: -1,
        mp: 2,
        f: 0,
        sl: 327,
        sr: 0,
    },
    TermLR {
        d: 0,
        m: 2,
        mp: 1,
        f: 0,
        sl: -323,
        sr: 1_165,
    },
    TermLR {
        d: 1,
        m: 1,
        mp: -1,
        f: 0,
        sl: 299,
        sr: 0,
    },
    TermLR {
        d: 2,
        m: 0,
        mp: 3,
        f: 0,
        sl: 294,
        sr: 0,
    },
    TermLR {
        d: 2,
        m: 0,
        mp: -1,
        f: -2,
        sl: 0,
        sr: 8_752,
    },
];

/// Meeus table 47.B — Moon latitude (Σb). Terms with `M` scaled by `E^|M|`.
const TBL_B: &[TermB] = &[
    TermB {
        d: 0,
        m: 0,
        mp: 0,
        f: 1,
        sb: 5_128_122,
    },
    TermB {
        d: 0,
        m: 0,
        mp: 1,
        f: 1,
        sb: 280_602,
    },
    TermB {
        d: 0,
        m: 0,
        mp: 1,
        f: -1,
        sb: 277_693,
    },
    TermB {
        d: 2,
        m: 0,
        mp: 0,
        f: -1,
        sb: 173_237,
    },
    TermB {
        d: 2,
        m: 0,
        mp: -1,
        f: 1,
        sb: 55_413,
    },
    TermB {
        d: 2,
        m: 0,
        mp: -1,
        f: -1,
        sb: 46_271,
    },
    TermB {
        d: 2,
        m: 0,
        mp: 0,
        f: 1,
        sb: 32_573,
    },
    TermB {
        d: 0,
        m: 0,
        mp: 2,
        f: 1,
        sb: 17_198,
    },
    TermB {
        d: 2,
        m: 0,
        mp: 1,
        f: -1,
        sb: 9_266,
    },
    TermB {
        d: 0,
        m: 0,
        mp: 2,
        f: -1,
        sb: 8_822,
    },
    TermB {
        d: 2,
        m: -1,
        mp: 0,
        f: -1,
        sb: 8_216,
    },
    TermB {
        d: 2,
        m: 0,
        mp: -2,
        f: -1,
        sb: 4_324,
    },
    TermB {
        d: 2,
        m: 0,
        mp: 1,
        f: 1,
        sb: 4_200,
    },
    TermB {
        d: 2,
        m: 1,
        mp: 0,
        f: -1,
        sb: -3_359,
    },
    TermB {
        d: 2,
        m: -1,
        mp: -1,
        f: 1,
        sb: 2_463,
    },
    TermB {
        d: 2,
        m: -1,
        mp: 0,
        f: 1,
        sb: 2_211,
    },
    TermB {
        d: 2,
        m: -1,
        mp: -1,
        f: -1,
        sb: 2_065,
    },
    TermB {
        d: 0,
        m: 1,
        mp: -1,
        f: -1,
        sb: -1_870,
    },
    TermB {
        d: 4,
        m: 0,
        mp: -1,
        f: -1,
        sb: 1_828,
    },
    TermB {
        d: 0,
        m: 1,
        mp: 0,
        f: 1,
        sb: -1_794,
    },
    TermB {
        d: 0,
        m: 0,
        mp: 0,
        f: 3,
        sb: -1_749,
    },
    TermB {
        d: 0,
        m: 1,
        mp: -1,
        f: 1,
        sb: -1_565,
    },
    TermB {
        d: 1,
        m: 0,
        mp: 0,
        f: 1,
        sb: -1_491,
    },
    TermB {
        d: 0,
        m: 1,
        mp: 1,
        f: 1,
        sb: -1_475,
    },
    TermB {
        d: 0,
        m: 1,
        mp: 1,
        f: -1,
        sb: -1_410,
    },
    TermB {
        d: 0,
        m: 1,
        mp: 0,
        f: -1,
        sb: -1_344,
    },
    TermB {
        d: 1,
        m: 0,
        mp: 0,
        f: -1,
        sb: -1_335,
    },
    TermB {
        d: 0,
        m: 0,
        mp: 3,
        f: 1,
        sb: 1_107,
    },
    TermB {
        d: 4,
        m: 0,
        mp: 0,
        f: -1,
        sb: 1_021,
    },
    TermB {
        d: 4,
        m: 0,
        mp: -1,
        f: 1,
        sb: 833,
    },
    TermB {
        d: 0,
        m: 0,
        mp: 1,
        f: -3,
        sb: 777,
    },
    TermB {
        d: 4,
        m: 0,
        mp: -2,
        f: 1,
        sb: 671,
    },
    TermB {
        d: 2,
        m: 0,
        mp: 0,
        f: -3,
        sb: 607,
    },
    TermB {
        d: 2,
        m: 0,
        mp: 2,
        f: -1,
        sb: 596,
    },
    TermB {
        d: 2,
        m: -1,
        mp: 1,
        f: -1,
        sb: 491,
    },
    TermB {
        d: 2,
        m: 0,
        mp: -2,
        f: 1,
        sb: -451,
    },
    TermB {
        d: 0,
        m: 0,
        mp: 3,
        f: -1,
        sb: 439,
    },
    TermB {
        d: 2,
        m: 0,
        mp: 2,
        f: 1,
        sb: 422,
    },
    TermB {
        d: 2,
        m: 0,
        mp: -3,
        f: -1,
        sb: 421,
    },
    TermB {
        d: 2,
        m: 1,
        mp: -1,
        f: 1,
        sb: -366,
    },
    TermB {
        d: 2,
        m: 1,
        mp: 0,
        f: 1,
        sb: -351,
    },
    TermB {
        d: 4,
        m: 0,
        mp: 0,
        f: 1,
        sb: 331,
    },
    TermB {
        d: 2,
        m: -1,
        mp: 1,
        f: 1,
        sb: 315,
    },
    TermB {
        d: 2,
        m: -2,
        mp: 0,
        f: -1,
        sb: 302,
    },
    TermB {
        d: 0,
        m: 0,
        mp: 1,
        f: 3,
        sb: -283,
    },
    TermB {
        d: 2,
        m: 1,
        mp: 1,
        f: -1,
        sb: -229,
    },
    TermB {
        d: 1,
        m: 1,
        mp: 0,
        f: -1,
        sb: 223,
    },
    TermB {
        d: 1,
        m: 1,
        mp: 0,
        f: 1,
        sb: 223,
    },
    TermB {
        d: 0,
        m: 1,
        mp: -2,
        f: -1,
        sb: -220,
    },
    TermB {
        d: 2,
        m: 1,
        mp: -1,
        f: -1,
        sb: -220,
    },
    TermB {
        d: 1,
        m: 0,
        mp: 1,
        f: 1,
        sb: -185,
    },
    TermB {
        d: 2,
        m: -1,
        mp: -2,
        f: -1,
        sb: 181,
    },
    TermB {
        d: 0,
        m: 1,
        mp: 2,
        f: 1,
        sb: -177,
    },
    TermB {
        d: 4,
        m: 0,
        mp: -2,
        f: -1,
        sb: 176,
    },
    TermB {
        d: 4,
        m: -1,
        mp: -1,
        f: -1,
        sb: 166,
    },
    TermB {
        d: 1,
        m: 0,
        mp: 1,
        f: -1,
        sb: -164,
    },
    TermB {
        d: 4,
        m: 0,
        mp: 1,
        f: -1,
        sb: 132,
    },
    TermB {
        d: 1,
        m: 0,
        mp: -1,
        f: -1,
        sb: -119,
    },
    TermB {
        d: 4,
        m: -1,
        mp: 0,
        f: -1,
        sb: 115,
    },
    TermB {
        d: 2,
        m: -2,
        mp: 0,
        f: 1,
        sb: 107,
    },
];

/// Geocentric apparent ecliptic longitude/latitude (deg) and distance (km) of the Moon at TT `t`
/// (Julian centuries since J2000.0). Meeus ch. 47. (Public for oracle validation against Meeus's
/// own worked example 47.a.)
pub fn moon_ecliptic(t: f64) -> (f64, f64, f64) {
    // Mean arguments (degrees).
    let lp = norm360(
        218.316_447_7 + 481_267.881_234_21 * t - 0.001_578_6 * t * t + t * t * t / 538_841.0
            - t * t * t * t / 65_194_000.0,
    );
    let d = norm360(
        297.850_192_1 + 445_267.111_403_4 * t - 0.001_881_9 * t * t + t * t * t / 545_868.0
            - t * t * t * t / 113_065_000.0,
    );
    let m = norm360(
        357.529_109_2 + 35_999.050_290_9 * t - 0.000_153_6 * t * t + t * t * t / 24_490_000.0,
    );
    let mp = norm360(
        134.963_396_4 + 477_198.867_505_5 * t + 0.008_741_4 * t * t + t * t * t / 69_699.0
            - t * t * t * t / 14_712_000.0,
    );
    let f = norm360(
        93.272_095_0 + 483_202.017_523_3 * t - 0.003_653_9 * t * t - t * t * t / 3_526_000.0
            + t * t * t * t / 863_310_000.0,
    );
    let a1 = norm360(119.75 + 131.849 * t);
    let a2 = norm360(53.09 + 479_264.290 * t);
    let a3 = norm360(313.45 + 481_266.484 * t);
    let e = 1.0 - 0.002_516 * t - 0.000_007_4 * t * t;

    let mut sum_l = 0.0_f64;
    let mut sum_r = 0.0_f64;
    for term in TBL_LR {
        let arg =
            (term.d as f64 * d + term.m as f64 * m + term.mp as f64 * mp + term.f as f64 * f) * DEG;
        let ecc = match term.m.abs() {
            1 => e,
            2 => e * e,
            _ => 1.0,
        };
        sum_l += ecc * term.sl as f64 * sin(arg);
        sum_r += ecc * term.sr as f64 * cos(arg);
    }
    let mut sum_b = 0.0_f64;
    for term in TBL_B {
        let arg =
            (term.d as f64 * d + term.m as f64 * m + term.mp as f64 * mp + term.f as f64 * f) * DEG;
        let ecc = match term.m.abs() {
            1 => e,
            2 => e * e,
            _ => 1.0,
        };
        sum_b += ecc * term.sb as f64 * sin(arg);
    }

    // Additive terms (Meeus, after the tables).
    sum_l += 3_958.0 * sin(a1 * DEG) + 1_962.0 * sin((lp - f) * DEG) + 318.0 * sin(a2 * DEG);
    sum_b += -2_235.0 * sin(lp * DEG)
        + 382.0 * sin(a3 * DEG)
        + 175.0 * sin((a1 - f) * DEG)
        + 175.0 * sin((a1 + f) * DEG)
        + 127.0 * sin((lp - mp) * DEG)
        - 115.0 * sin((lp + mp) * DEG);

    let lambda = norm360(lp + sum_l / 1_000_000.0); // apparent ecliptic longitude (deg)
    let beta = sum_b / 1_000_000.0; // ecliptic latitude (deg)
    let dist = 385_000.56 + sum_r / 1_000.0; // distance Earth→Moon centre (km)
    (lambda, beta, dist)
}

/// Mean obliquity of the ecliptic (radians) at TT `t` (Meeus 22.2, low-order — adequate at ±1 min).
#[inline]
fn obliquity_rad(t: f64) -> f64 {
    let eps0 = 23.0 + (26.0 + (21.448 - t * (46.815 + t * (0.00059 - t * 0.001813))) / 60.0) / 60.0;
    eps0 * DEG
}

/// Greenwich mean sidereal time (deg) at UT Julian Day `jd` — Earth rotation, on UT (NOT TT).
/// Identical expression to `geometry.rs` so the two bodies share one Earth-rotation model.
#[inline]
fn gmst_deg(jd: f64) -> f64 {
    let t_ut = (jd - 2_451_545.0) / 36_525.0;
    norm360(
        280.460_618_37 + 360.985_647_366_29 * (jd - 2_451_545.0) + t_ut * t_ut * 0.000_387_933
            - t_ut * t_ut * t_ut / 38_710_000.0,
    )
}

/// Geocentric apparent equatorial coords (RA, Dec in radians) and distance (km) of the Moon.
fn moon_equatorial(jd: f64) -> (f64, f64, f64) {
    let jd_tt = jd + DELTA_T_SECS / 86_400.0;
    let t = (jd_tt - 2_451_545.0) / 36_525.0;
    let (lambda, beta, dist) = moon_ecliptic(t);
    let eps = obliquity_rad(t);
    let lam = lambda * DEG;
    let bet = beta * DEG;
    // Ecliptic → equatorial (Meeus 13.3/13.4).
    let ra = atan2(sin(lam) * cos(eps) - tan(bet) * sin(eps), cos(lam));
    let dec = asin(clamp(
        sin(bet) * cos(eps) + cos(bet) * sin(eps) * sin(lam),
        -1.0,
        1.0,
    ));
    (ra, dec, dist)
}

/// Topocentric **geometric** lunar altitude (degrees) at UT Julian Day `jd` for `site`.
/// Geocentric position is reduced to the observer by the parallax correction (Meeus ch. 40), which
/// for the Moon is first-order (~1°) — never optional.
pub fn moon_altitude_deg(jd: f64, site: &Site) -> f64 {
    let (ra, dec, dist) = moon_equatorial(jd);

    // Equatorial horizontal parallax.
    let sin_pi = clamp(EARTH_EQ_RADIUS_KM / dist, -1.0, 1.0);

    // Observer's geocentric quantities ρ·sinφ' and ρ·cosφ' (Meeus ch. 11), elevation-aware.
    let phi = site.lat_deg * DEG;
    let u = atan(0.996_647_19 * tan(phi));
    let h_over_a = site.elev_m / 6_378_140.0;
    let rho_sin_phip = 0.996_647_19 * sin(u) + h_over_a * sin(phi);
    let rho_cos_phip = cos(u) + h_over_a * cos(phi);

    // Local hour angle of the geocentric Moon (radians).
    let gmst = gmst_deg(jd);
    let h = norm360(gmst + site.lon_deg - ra / DEG) * DEG;

    // Topocentric RA/Dec (Meeus 40.2/40.3).
    let delta_ra = atan2(
        -rho_cos_phip * sin_pi * sin(h),
        cos(dec) - rho_cos_phip * sin_pi * cos(h),
    );
    let dec_topo = atan2(
        (sin(dec) - rho_sin_phip * sin_pi) * cos(delta_ra),
        cos(dec) - rho_cos_phip * sin_pi * cos(h),
    );
    // Topocentric hour angle: H' = LST − α' = (LST − α) − Δα = H − Δα.
    let h_topo = h - delta_ra;

    let sin_alt = sin(phi) * sin(dec_topo) + cos(phi) * cos(dec_topo) * cos(h_topo);
    asin(clamp(sin_alt, -1.0, 1.0)) / DEG
}

/// Apparent lunar semidiameter (degrees) at UT `jd` — distance-dependent
/// (`sin s = 0.272481 · sin π`, Meeus). Ranges ~14.7′–16.7′, more honest than a fixed value.
pub fn moon_semidiameter_deg(jd: f64) -> f64 {
    let (_, _, dist) = moon_equatorial(jd);
    let sin_pi = clamp(EARTH_EQ_RADIUS_KM / dist, -1.0, 1.0);
    asin(clamp(0.272_481 * sin_pi, -1.0, 1.0)) / DEG
}
