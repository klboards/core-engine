//! The F1 read-spec union (ADR core-domain/0009): every zman is a typed *read* off the solar
//! altitude curve. The read is resolved against an **effective altitude** = geometric + refraction,
//! where the refraction model is a knob (ADR core-domain/0006/0013):
//! - **netz/shkia** → apparent (Saemundsson) sun-centre at −(semidiameter + dip) per `HorizonMode`;
//! - **depression shitot** → geometric (refraction off, classical/halachic default).
//!
//! Reads are anchored to a caller-supplied UT reference Julian Day `ref_jd` (the harness derives it
//! from the civil date + tz — a tz/edge concern per ADR core-domain/0007; the core stays tz-free).
//! "GRA vs Magen Avraham" and "day definition" are settings of the `proportional_day_bounds` knob
//! (the `start`/`end` bounds), NOT code branches.

use crate::geometry::solar_altitude_deg;
use crate::lunar::{moon_altitude_deg, moon_semidiameter_deg};
use crate::optics::{horizon_apparent_target_deg, horizon_target_deg, RefractionModel};
use crate::params::Optics;
use crate::units::GeometricAltitude;
use crate::{AbsoluteInstant, Site, ZmanResult};

/// Which body's altitude curve a crossing is solved against. The crossing machinery is otherwise
/// body-agnostic; the Sun (F1) and Moon (F2) differ only in the altitude function (and the Moon's
/// distance-dependent semidiameter + mandatory topocentric parallax, handled in `lunar`).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Body {
    /// The Sun (F1).
    Sun,
    /// The Moon (F2).
    Moon,
}

/// Sense of an altitude crossing.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Direction {
    /// Sun ascending (morning).
    Rising,
    /// Sun descending (evening).
    Setting,
}

/// A bound of the proportional ("seasonal-hour") day — data, set by the `proportional_day_bounds`
/// knob. GRA = (Netz, Shkia); MGA = (depression −16.1 rising, depression −16.1 setting).
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Bound {
    /// Sunrise (apparent horizon crossing).
    Netz,
    /// Sunset (apparent horizon crossing).
    Shkia,
    /// A depression-angle bound (e.g. MGA's alot/tzeit at −16.1°).
    Depression {
        /// Depression below the horizon, degrees (magnitude).
        angle_deg: f64,
        /// Morning (rising) or evening (setting).
        dir: Direction,
    },
}

/// A typed read off the altitude curve.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum ReadSpec {
    /// Geometric depression (refraction per `Optics::depression_refraction`, default off).
    DepressionAngle {
        /// Depression below the horizon, degrees (magnitude).
        angle_deg: f64,
        /// Morning (rising) or evening (setting).
        dir: Direction,
    },
    /// Apparent sunrise/sunset: apparent sun-centre at −(semidiameter + dip) (ADR core-domain/0013).
    HorizonCrossing {
        /// Morning (rising = netz) or evening (setting = shkia).
        dir: Direction,
    },
    /// Chatzot = midpoint(netz, shkia).
    ExtremumMidpoint,
    /// Proportional: `start + fraction·(end − start)`, bounds set by the knob.
    Proportional {
        /// Fraction of the seasonal-hour day (e.g. 0.25 = 3 proportional hours).
        fraction: f64,
        /// Start bound of the proportional day.
        start: Bound,
        /// End bound of the proportional day.
        end: Bound,
    },
}

const SCAN_STEPS: u32 = 1080; // ~1-min scan over a 0.75-day window to bracket a crossing
const BISECT_ITERS: u32 = 60; // 60 halvings → far sub-nanosecond

/// Effective altitude (degrees) the read is solved against: geometric + refraction model.
/// `RefractionModel::None` ⇒ geometric.
#[inline]
fn effective_alt_deg(jd: f64, site: &Site, body: Body, refr: RefractionModel) -> f64 {
    let geo = GeometricAltitude(match body {
        Body::Sun => solar_altitude_deg(jd, site),
        Body::Moon => moon_altitude_deg(jd, site),
    });
    refr.apparent(geo).deg()
}

/// Sun effective altitude (degrees) at UT JD `jd` for `site`, under the depression-refraction knob —
/// exposed (additive) so the Kiddush-Levana **night** predicate (coupling #3, ADR core-domain/0016)
/// can test "sun below −angle" without re-implementing [`effective_alt_deg`]. Geometric when
/// `optics.depression_refraction` is `None` (the default for depression shitot).
pub fn sun_effective_alt_deg(jd: f64, site: &Site, optics: &Optics) -> f64 {
    effective_alt_deg(jd, site, Body::Sun, optics.depression_refraction)
}

/// Find the UT JD in `[lo, hi]` where the effective altitude crosses `target` with the slope
/// matching `dir`. `None` if no such crossing — i.e. does-not-occur (ADR core-domain/0009).
fn find_crossing(
    site: &Site,
    lo: f64,
    hi: f64,
    target: f64,
    dir: Direction,
    body: Body,
    refr: RefractionModel,
) -> Option<f64> {
    let mut prev_t = lo;
    let mut prev_f = effective_alt_deg(lo, site, body, refr) - target;
    let mut i = 1u32;
    while i <= SCAN_STEPS {
        let t = lo + (hi - lo) * (i as f64 / SCAN_STEPS as f64);
        let f = effective_alt_deg(t, site, body, refr) - target;
        if (prev_f < 0.0) != (f < 0.0) {
            let increasing = f > prev_f;
            let want_increasing = matches!(dir, Direction::Rising);
            if increasing == want_increasing {
                return Some(bisect(site, prev_t, t, target, body, refr));
            }
        }
        prev_t = t;
        prev_f = f;
        i += 1;
    }
    None
}

fn bisect(
    site: &Site,
    mut a: f64,
    mut b: f64,
    target: f64,
    body: Body,
    refr: RefractionModel,
) -> f64 {
    let mut fa = effective_alt_deg(a, site, body, refr) - target;
    let mut k = 0u32;
    while k < BISECT_ITERS {
        let mid = 0.5 * (a + b);
        let fmid = effective_alt_deg(mid, site, body, refr) - target;
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
    // 0.75-day half-windows around the local-noon anchor, so an evening event that crosses past
    // civil midnight (e.g. Paris June tzeit R"T) is still captured; the slope filter keeps the
    // correct (morning/evening) crossing.
    match dir {
        Direction::Rising => (ref_jd - 0.75, ref_jd),
        Direction::Setting => (ref_jd, ref_jd + 0.75),
    }
}

/// Resolve a read to a UT Julian Day under the given optics knobs, or `None` (does-not-occur).
pub fn read_jd(site: &Site, ref_jd: f64, spec: ReadSpec, optics: &Optics) -> Option<f64> {
    match spec {
        ReadSpec::DepressionAngle { angle_deg, dir } => {
            let (lo, hi) = window(ref_jd, dir);
            find_crossing(
                site,
                lo,
                hi,
                -angle_deg,
                dir,
                Body::Sun,
                optics.depression_refraction,
            )
        }
        ReadSpec::HorizonCrossing { dir } => {
            let (lo, hi) = window(ref_jd, dir);
            let target = horizon_apparent_target_deg(optics.horizon_mode, site.elev_m);
            find_crossing(
                site,
                lo,
                hi,
                target,
                dir,
                Body::Sun,
                optics.horizon_refraction,
            )
        }
        ReadSpec::ExtremumMidpoint => {
            let netz = read_jd(
                site,
                ref_jd,
                ReadSpec::HorizonCrossing {
                    dir: Direction::Rising,
                },
                optics,
            )?;
            let shkia = read_jd(
                site,
                ref_jd,
                ReadSpec::HorizonCrossing {
                    dir: Direction::Setting,
                },
                optics,
            )?;
            Some(0.5 * (netz + shkia))
        }
        ReadSpec::Proportional {
            fraction,
            start,
            end,
        } => {
            let s = bound_jd(site, ref_jd, start, optics)?;
            let e = bound_jd(site, ref_jd, end, optics)?;
            Some(s + fraction * (e - s))
        }
    }
}

/// Span (in days) of the proportional day for the given bounds — `None` if either bound
/// does-not-occur. One sha'ah zmanit = span / 12.
pub fn proportional_span_days(
    site: &Site,
    ref_jd: f64,
    start: Bound,
    end: Bound,
    optics: &Optics,
) -> Option<f64> {
    let s = bound_jd(site, ref_jd, start, optics)?;
    let e = bound_jd(site, ref_jd, end, optics)?;
    Some(e - s)
}

fn bound_jd(site: &Site, ref_jd: f64, bound: Bound, optics: &Optics) -> Option<f64> {
    match bound {
        Bound::Netz => read_jd(
            site,
            ref_jd,
            ReadSpec::HorizonCrossing {
                dir: Direction::Rising,
            },
            optics,
        ),
        Bound::Shkia => read_jd(
            site,
            ref_jd,
            ReadSpec::HorizonCrossing {
                dir: Direction::Setting,
            },
            optics,
        ),
        Bound::Depression { angle_deg, dir } => read_jd(
            site,
            ref_jd,
            ReadSpec::DepressionAngle { angle_deg, dir },
            optics,
        ),
    }
}

/// Resolve a read to an absolute instant (ADR core-domain/0001), or `None` (does-not-occur).
#[inline]
pub fn read_instant(site: &Site, ref_jd: f64, spec: ReadSpec, optics: &Optics) -> ZmanResult {
    read_jd(site, ref_jd, spec, optics).map(AbsoluteInstant::from_julian_day)
}

/// Moonrise/moonset (F2) within the civil day starting at `day_start_jd` (the UT Julian Day of the
/// local-midnight start of the date) — the **first** rise (`Rising`) or set (`Setting`) event in
/// `[day_start_jd, day_start_jd + 1]`, or `None` on a day the Moon does not rise/set (it skips
/// ~once a month). Unlike the Sun, lunar rise/set do not bracket local noon, so the search spans a
/// full civil day rather than `window()`'s noon-anchored half-day.
///
/// The event is the apparent upper limb at the dipped horizon: target = `−(moon semidiameter + dip)`
/// with `optics.horizon_refraction` added by the crossing solver. The Moon's semidiameter is taken
/// at the day midpoint (it varies <0.1′ across the day → sub-second).
pub fn moon_rise_set(
    site: &Site,
    day_start_jd: f64,
    dir: Direction,
    optics: &Optics,
) -> ZmanResult {
    let sd = moon_semidiameter_deg(day_start_jd + 0.5);
    let target = horizon_target_deg(optics.horizon_mode, site.elev_m, sd);
    find_crossing(
        site,
        day_start_jd,
        day_start_jd + 1.0,
        target,
        dir,
        Body::Moon,
        optics.horizon_refraction,
    )
    .map(AbsoluteInstant::from_julian_day)
}
