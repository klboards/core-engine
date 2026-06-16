//! The F1 read-spec union (ADR core-domain/0009): every zman is a typed *read* off the
//! continuous `altitude(t)` curve. "GRA vs Magen Avraham" and "day definition" are settings of
//! the `proportional_day_bounds` knob (the `start`/`end` bounds), NOT code branches.
//!
//! Reads are anchored to a caller-supplied UT reference Julian Day `ref_jd` (the harness derives
//! it from the civil date + tz — a tz/edge concern per ADR core-domain/0007; the core stays
//! tz-free). Rising events are searched in `[ref_jd-0.5, ref_jd]`, setting events in
//! `[ref_jd, ref_jd+0.5]`.

use crate::refraction::horizon_crossing_target_deg;
use crate::solar::solar_altitude_deg;
use crate::{AbsoluteInstant, Site, ZmanResult};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Direction {
    Rising,
    Setting,
}

/// A bound of the proportional ("seasonal-hour") day — data, set by the
/// `proportional_day_bounds` knob. GRA = (Netz, Shkia); MGA = (depression −16.1 rising,
/// depression −16.1 setting).
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Bound {
    Netz,
    Shkia,
    Depression { angle_deg: f64, dir: Direction },
}

/// A typed read off the altitude curve.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum ReadSpec {
    /// Refraction-INDEPENDENT geometry: solve geometric altitude = −`angle_deg`.
    DepressionAngle { angle_deg: f64, dir: Direction },
    /// Apparent sunrise/sunset: composes refraction + geometric dip (ADR core-domain/0006).
    HorizonCrossing { dir: Direction },
    /// Chatzot = midpoint(netz, shkia).
    ExtremumMidpoint,
    /// Proportional: `start + fraction·(end − start)`, bounds set by the knob.
    Proportional {
        fraction: f64,
        start: Bound,
        end: Bound,
    },
}

const SCAN_STEPS: u32 = 1440; // 1-minute scan to bracket a crossing
const BISECT_ITERS: u32 = 60; // 60 halvings of a 1-min bracket → far sub-nanosecond

#[inline]
fn altitude(jd: f64, site: &Site) -> f64 {
    solar_altitude_deg(jd, site)
}

/// Find the UT JD in `[lo, hi]` where geometric altitude crosses `target` with the slope
/// matching `dir` (Rising = increasing, Setting = decreasing). `None` if no such crossing —
/// i.e. does-not-occur (ADR core-domain/0009).
fn find_crossing(site: &Site, lo: f64, hi: f64, target: f64, dir: Direction) -> Option<f64> {
    let mut prev_t = lo;
    let mut prev_f = altitude(lo, site) - target;
    let mut i = 1u32;
    while i <= SCAN_STEPS {
        let t = lo + (hi - lo) * (i as f64 / SCAN_STEPS as f64);
        let f = altitude(t, site) - target;
        if (prev_f < 0.0) != (f < 0.0) {
            let increasing = f > prev_f;
            let want_increasing = matches!(dir, Direction::Rising);
            if increasing == want_increasing {
                return Some(bisect(site, prev_t, t, target));
            }
        }
        prev_t = t;
        prev_f = f;
        i += 1;
    }
    None
}

fn bisect(site: &Site, mut a: f64, mut b: f64, target: f64) -> f64 {
    let mut fa = altitude(a, site) - target;
    let mut k = 0u32;
    while k < BISECT_ITERS {
        let mid = 0.5 * (a + b);
        let fmid = altitude(mid, site) - target;
        if (fa < 0.0) != (fmid < 0.0) {
            b = mid;
        } else {
            a = mid;
            fa = fmid;
        }
        k += 1;
    }
    0.5 * (a + b)
}

#[inline]
fn window(ref_jd: f64, dir: Direction) -> (f64, f64) {
    // 0.75-day half-windows around the local-noon anchor. Wider than ±0.5 so an evening
    // depression event that crosses past civil midnight (e.g. Paris June tzeit R"T at ≈ref+0.53)
    // is still captured; the direction filter keeps the correct (morning/evening) crossing.
    match dir {
        Direction::Rising => (ref_jd - 0.75, ref_jd),
        Direction::Setting => (ref_jd, ref_jd + 0.75),
    }
}

/// Resolve a read to a UT Julian Day, or `None` (does-not-occur).
pub fn read_jd(site: &Site, ref_jd: f64, spec: ReadSpec) -> Option<f64> {
    match spec {
        ReadSpec::DepressionAngle { angle_deg, dir } => {
            let (lo, hi) = window(ref_jd, dir);
            find_crossing(site, lo, hi, -angle_deg, dir)
        }
        ReadSpec::HorizonCrossing { dir } => {
            let (lo, hi) = window(ref_jd, dir);
            find_crossing(site, lo, hi, horizon_crossing_target_deg(site.elev_m), dir)
        }
        ReadSpec::ExtremumMidpoint => {
            let netz = read_jd(site, ref_jd, ReadSpec::HorizonCrossing { dir: Direction::Rising })?;
            let shkia =
                read_jd(site, ref_jd, ReadSpec::HorizonCrossing { dir: Direction::Setting })?;
            Some(0.5 * (netz + shkia))
        }
        ReadSpec::Proportional { fraction, start, end } => {
            let s = bound_jd(site, ref_jd, start)?;
            let e = bound_jd(site, ref_jd, end)?;
            Some(s + fraction * (e - s))
        }
    }
}

/// Span (in days) of the proportional day for the given bounds — `None` if either bound
/// does-not-occur. One sha'ah zmanit = span / 12.
pub fn proportional_span_days(site: &Site, ref_jd: f64, start: Bound, end: Bound) -> Option<f64> {
    let s = bound_jd(site, ref_jd, start)?;
    let e = bound_jd(site, ref_jd, end)?;
    Some(e - s)
}

fn bound_jd(site: &Site, ref_jd: f64, bound: Bound) -> Option<f64> {
    match bound {
        Bound::Netz => read_jd(site, ref_jd, ReadSpec::HorizonCrossing { dir: Direction::Rising }),
        Bound::Shkia => {
            read_jd(site, ref_jd, ReadSpec::HorizonCrossing { dir: Direction::Setting })
        }
        Bound::Depression { angle_deg, dir } => {
            read_jd(site, ref_jd, ReadSpec::DepressionAngle { angle_deg, dir })
        }
    }
}

/// Resolve a read to an absolute instant (ADR core-domain/0001), or `None` (does-not-occur).
#[inline]
pub fn read_instant(site: &Site, ref_jd: f64, spec: ReadSpec) -> ZmanResult {
    read_jd(site, ref_jd, spec).map(AbsoluteInstant::from_julian_day)
}
