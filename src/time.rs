//! no_std civil-date ↔ Julian Date (Meeus, *Astronomical Algorithms* ch. 7).
//! Pure integer/float arithmetic — no time crate, no std. The core stays timezone-free
//! (ADR core-domain/0007); callers pass UT Julian Days.

use libm::floor;

/// Julian Day (UT) for a proleptic-Gregorian date+time given as a fractional day.
/// `day` may carry a fraction (e.g. 15.5 = the 15th at 12:00 UT).
pub fn jd_from_gregorian(year: i32, month: u32, day: f64) -> f64 {
    let (y, m) = if month <= 2 {
        (year - 1, month as i32 + 12)
    } else {
        (year, month as i32)
    };
    let a = floor(y as f64 / 100.0);
    let b = 2.0 - a + floor(a / 4.0);
    floor(365.25 * (y as f64 + 4716.0)) + floor(30.6001 * (m as f64 + 1.0)) + day + b - 1524.5
}

/// UT Julian Day from Unix seconds.
#[inline]
pub fn jd_from_unix_secs(unix_secs: f64) -> f64 {
    unix_secs / 86_400.0 + 2_440_587.5
}
