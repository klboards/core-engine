//! F1 solar geometry — NOAA / Meeus low-precision solar position → geometric `altitude(t)`.
//! All trig via `libm` (ADR core-domain/0010 determinism anchor). Returns the **geometric**
//! altitude in degrees (no refraction, no horizon dip — those compose in `reads`/`refraction`
//! per the ADR core-domain/0006 seam).

use crate::Site;
use libm::{asin, atan2, cos, sin};

const DEG: f64 = core::f64::consts::PI / 180.0;

/// TT − UT (seconds). PROVISIONAL constant for 2026 (≈ +69 s); a time-varying ΔT model/table is a
/// TODO (ADR core-domain/0012, /0003, /0007). The Sun's position is computed at TT = UT + ΔT.
const DELTA_T_SECS: f64 = 69.0;

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

/// Geometric solar altitude (degrees) at UT Julian Day `jd` for `site`.
pub fn solar_altitude_deg(jd: f64, site: &Site) -> f64 {
    // The Sun's position is an ephemeris quantity → compute it at Terrestrial Time
    // (TT = UT + ΔT), while Earth-rotation (sidereal time) stays on UT (ADR core-domain/0012).
    // ΔT is a provisional 2026 constant; a time-varying ΔT model is a TODO (core-domain/0003/0007).
    let jd_tt = jd + DELTA_T_SECS / 86_400.0;
    let t = (jd_tt - 2_451_545.0) / 36_525.0; // Julian centuries (TT) since J2000.0

    let l0 = norm360(280.46646 + t * (36_000.76983 + 0.0003032 * t)); // geom mean longitude
    let m = 357.52911 + t * (35_999.05029 - 0.0001537 * t); // geom mean anomaly (deg)
    let mr = m * DEG;

    // Sun's equation of center
    let c = sin(mr) * (1.914602 - t * (0.004817 + 0.000014 * t))
        + sin(2.0 * mr) * (0.019993 - 0.000101 * t)
        + sin(3.0 * mr) * 0.000289;

    let true_long = l0 + c;
    let omega = 125.04 - 1934.136 * t;
    let app_long = true_long - 0.00569 - 0.00478 * sin(omega * DEG); // apparent longitude

    // Obliquity of the ecliptic (corrected)
    let eps0 = 23.0 + (26.0 + (21.448 - t * (46.815 + t * (0.00059 - t * 0.001813))) / 60.0) / 60.0;
    let eps = (eps0 + 0.00256 * cos(omega * DEG)) * DEG;

    let lam = app_long * DEG;
    let decl = asin(sin(eps) * sin(lam)); // declination (rad)
    let ra = atan2(cos(eps) * sin(lam), cos(lam)); // right ascension (rad)
    let ra_deg = norm360(ra / DEG);

    // Greenwich mean sidereal time (deg) — Earth rotation, on UT (NOT TT).
    let t_ut = (jd - 2_451_545.0) / 36_525.0;
    let gmst = norm360(
        280.46061837 + 360.98564736629 * (jd - 2_451_545.0) + t_ut * t_ut * 0.000387933
            - t_ut * t_ut * t_ut / 38_710_000.0,
    );

    let hour_angle = (gmst + site.lon_deg - ra_deg) * DEG; // local hour angle (rad)
    let phi = site.lat_deg * DEG;

    let sin_alt = sin(phi) * sin(decl) + cos(phi) * cos(decl) * cos(hour_angle);
    asin(clamp(sin_alt, -1.0, 1.0)) / DEG
}
