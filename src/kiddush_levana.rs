//! Kiddush Levana (Birkat Halevana) window (ADR core-domain/0015) — the period in which the
//! blessing over the moon may be recited, derived from the F3 **molad** (`calendar::molad_instant`)
//! plus the start/end knobs. Plus a thin F2 **visibility** primitive (`moon_visible`).
//!
//! **Scope boundary (Phase 2):** this module ships the *molad-derived window bounds* and the
//! *moon-up* primitive. The full halachic answer — recite only **at night**, with the **moon above
//! the horizon**, **inside the window** — is the F1 ∩ F2 ∩ window intersection, a **Phase 3**
//! coupling (mirrors the F3↔F1 day-roll deferral of ADR core-domain/0014). The core resolves no
//! convention: start/end are knobs.

use crate::calendar::{molad_instant, CHALAKIM_PER_DAY, CHALAKIM_PER_MONTH};
use crate::lunar::moon_altitude_deg;
use crate::params::{KiddushLevanaEnd, KiddushLevanaStart, Optics};
use crate::units::GeometricAltitude;
use crate::{AbsoluteInstant, Site};
use libm::round;

const NANOS_PER_DAY: f64 = 86_400.0 * 1.0e9;

/// Window offset (in days from the molad) for the start knob.
fn start_offset_days(start: KiddushLevanaStart) -> f64 {
    match start {
        KiddushLevanaStart::ThreeDays => 3.0,
        KiddushLevanaStart::SevenDays => 7.0,
        KiddushLevanaStart::Molad => 0.0,
    }
}

/// Window offset (in days from the molad) for the end knob.
fn end_offset_days(end: KiddushLevanaEnd) -> f64 {
    match end {
        // Half a mean synodic month, exact: (765433/2) chalakim → days.
        KiddushLevanaEnd::HalfMonth => CHALAKIM_PER_MONTH as f64 / 2.0 / CHALAKIM_PER_DAY as f64,
        KiddushLevanaEnd::FifteenDays => 15.0,
    }
}

/// Shift an absolute instant by a (possibly fractional) number of days, rounded to the nanosecond
/// via `libm::round` (deterministic on every target — ADR core-domain/0010). Uses **saturating**
/// addition: the `AbsoluteInstant` i64-nanosecond domain tops out near 2262 CE, so a far-future
/// (out-of-domain) molad clamps to the bound rather than overflowing — a defined value, never a
/// debug panic / release silent-wrap (the "never silently wrong" invariant; bug found in /0017).
#[inline]
fn shift_days(t: AbsoluteInstant, days: f64) -> AbsoluteInstant {
    AbsoluteInstant {
        unix_nanos: t
            .unix_nanos
            .saturating_add(round(days * NANOS_PER_DAY) as i64),
    }
}

/// The Kiddush Levana window `[earliest, latest]` for the Hebrew `(year, month)` under the knobs —
/// `earliest = molad + start`, `latest = molad + end`, as absolute instants (ADR core-domain/0001).
/// The molad's meridian assumption (`calendar::MOLAD_MERIDIAN_DEG_EAST`) flows through; the window
/// is days wide, so that assumption is immaterial to its practical use (flagged in /0015).
pub fn kiddush_levana_window(
    year: i32,
    month: u8,
    start: KiddushLevanaStart,
    end: KiddushLevanaEnd,
) -> (AbsoluteInstant, AbsoluteInstant) {
    let molad = molad_instant(year, month);
    (
        shift_days(molad, start_offset_days(start)),
        shift_days(molad, end_offset_days(end)),
    )
}

/// Whether the Moon is above the horizon (F2 visibility primitive) at UT Julian Day `jd` for `site`:
/// the topocentric **apparent** lunar centre altitude > 0, with refraction per `optics`. A building
/// block for the Phase-3 night ∩ moon-up ∩ window intersection — NOT itself the full KL answer.
pub fn moon_visible(jd: f64, site: &Site, optics: &Optics) -> bool {
    let geo = GeometricAltitude(moon_altitude_deg(jd, site));
    optics.horizon_refraction.apparent(geo).deg() > 0.0
}
