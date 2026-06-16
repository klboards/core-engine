//! The ADR core-domain/0001 cross-function couplings (Phase 3, ADR core-domain/0016). This is the
//! **only** module that depends on both F1/F2 (`events`) and F3 (`calendar`/`tekufa`). The dependency
//! is one-way and explicit: F1 boundary instants roll the F3 day (#1); F3 date+realm context selects
//! F1 reads (#2, in `calendar`); F3 windows are confirmed by F2/F1 (#3); F3 arithmetic gates a
//! recitation start (#4). `events` never imports F3 and `calendar` never imports F1 — keeping the DAG
//! acyclic and the float/integer seam clean.
//!
//! **Open gates (NOT resolved here, ADR core-domain/0016):** the day boundary is a caller-supplied
//! `ReadSpec` (the shkia-vs-tzeit bein-hashmashot safek is the caller's, not the core's); `Realm` is a
//! provisioned input; high-latitude does-not-occur propagates as a typed value, never a guessed date.

use crate::calendar::{
    fixed_from_hebrew, gregorian_from_fixed, hebrew_from_fixed, HebrewDate, RataDie,
};
use crate::events::{read_instant, read_jd, sun_effective_alt_deg, Direction, ReadSpec};
use crate::kiddush_levana::{kiddush_levana_window, moon_visible};
use crate::params::{KiddushLevanaEnd, KiddushLevanaStart, TalUmatarBasis, TekufaMethod};
use crate::tekufa::{tekufa_civil, Season};
use crate::time::{jd_from_gregorian, jd_from_unix_secs};
use crate::{AbsoluteInstant, Site, ZmanResult};

/// Julian Day at RD 0, 00:00 UT (RD 1 = 0001-01-01 = JD 1_721_425.5). Mirrors the (private)
/// `calendar::RD0_JULIAN_DAY`; used to map an absolute instant to its civil Rata Die.
const JD_AT_RD0: f64 = 1_721_424.5;

/// The day-boundary read used when a caller has no specific shita: apparent sunset (shkia). The
/// shkia-vs-tzeit choice (the bein-hashmashot safek) is an open gate — pass a `DepressionAngle`
/// setting read for a tzeit boundary instead (ADR core-domain/0016).
pub const DEFAULT_DAY_BOUNDARY: ReadSpec = ReadSpec::HorizonCrossing {
    dir: Direction::Setting,
};

/// Outcome of a day-roll (coupling #1). `BoundaryDoesNotOccur` is the high-latitude does-not-occur
/// signal: the core surfaces it rather than silently falling back to civil midnight (gate (d)).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum DayRoll {
    /// The Hebrew date current at the instant, the boundary having resolved normally.
    Resolved(HebrewDate),
    /// A day-boundary read in the determining window did-not-occur (polar day/night) — the roll is
    /// undefined; what to display is an edge decision (open gate, ADR core-domain/0016).
    BoundaryDoesNotOccur,
}

/// Local-noon UT anchor (Julian Day) for civil Rata Die `rd`, the reference `events` reads expect.
/// Local noon = noon UT − λ/360 day; this keeps an evening event inside the read's setting window for
/// any longitude (the UTC-vs-local subtlety of coupling #1; see [`hebrew_date_at_instant`]).
fn local_noon_jd(rd: i64, lon_deg: f64) -> f64 {
    let (y, m, d) = gregorian_from_fixed(RataDie(rd));
    jd_from_gregorian(y, m as u32, d as f64 + 0.5) - lon_deg / 360.0
}

/// The Hebrew date current at absolute instant `t` for `site`, given the day-boundary read + optics
/// (coupling #1: the Hebrew day rolls at a sun-defined instant, not civil midnight).
///
/// The Hebrew date labelling civil day `D` covers `[boundary(D−1), boundary(D))`, so the result is
/// `hebrew_from_fixed(D)` for the least civil day `D` whose boundary is strictly after `t`. The civil
/// candidate is seeded from `t`'s **UTC** date (the core is tz-free, ADR core-domain/0007) but the
/// decision is made purely by comparing `t` against each day's boundary *instant* — so far-longitude
/// instants near 00:00 UT do not go off-by-one. If a boundary in the scanned window does-not-occur,
/// the roll is reported as [`DayRoll::BoundaryDoesNotOccur`] (the high-latitude gate stays open).
pub fn hebrew_date_at_instant(
    t: AbsoluteInstant,
    site: &Site,
    boundary: ReadSpec,
    optics: &crate::params::Optics,
) -> DayRoll {
    let jd_t = jd_from_unix_secs(t.unix_nanos as f64 / 1.0e9);
    let rd_t = libm::floor(jd_t - JD_AT_RD0) as i64;
    // D = least civil day whose boundary is strictly after t; Hebrew date = hebrew_from_fixed(D).
    // Scan the ±1-day neighbourhood (longitude robustness). Any does-not-occur boundary in the
    // window makes the roll undefined (conservative; the policy is an open gate).
    let mut d = rd_t - 1;
    while d <= rd_t + 2 {
        match read_jd(site, local_noon_jd(d, site.lon_deg), boundary, optics) {
            Some(b_jd) if b_jd > jd_t => return DayRoll::Resolved(hebrew_from_fixed(RataDie(d))),
            Some(_) => {}
            None => return DayRoll::BoundaryDoesNotOccur,
        }
        d += 1;
    }
    DayRoll::BoundaryDoesNotOccur
}

/// Whether Kiddush Levana is sayable at absolute instant `t` (coupling #3: molad window ∩ F2 moon-up
/// ∩ F1 night). `night_depression_deg` is the caller's tzeit shita (the core resolves none). Day-type
/// exclusions (not on Shabbat / Yom Tov / pre-Tisha-b'Av, motzaei-Shabbat-only customs) are **edge**
/// policy — minhag-contested, so kept out of correctness (ADR core-domain/0016); intersect with
/// `calendar::classify_day` upstream if your minhag requires it. The molad-meridian assumption flows
/// through the window (immaterial at day-width) and stays flagged.
#[allow(clippy::too_many_arguments)]
pub fn kiddush_levana_sayable_at(
    t: AbsoluteInstant,
    year: i32,
    month: u8,
    site: &Site,
    night_depression_deg: f64,
    start: KiddushLevanaStart,
    end: KiddushLevanaEnd,
    optics: &crate::params::Optics,
) -> bool {
    let (open, close) = kiddush_levana_window(year, month, start, end);
    if t.unix_nanos < open.unix_nanos || t.unix_nanos > close.unix_nanos {
        return false;
    }
    let jd = jd_from_unix_secs(t.unix_nanos as f64 / 1.0e9);
    // Night: the Sun is below the tzeit depression.
    if sun_effective_alt_deg(jd, site, optics) >= -night_depression_deg {
        return false;
    }
    // Moon apparently up.
    moon_visible(jd, site, optics)
}

/// The window ∩ night bracket on the night whose evening sits at local-noon anchor `night_ref_jd`
/// (coupling #3): `[max(window_open, dusk), min(window_close, dawn)]`, or `None` if empty or a night
/// boundary does-not-occur. **Moon-up is NOT folded in here** — it can be a sub-interval (the moon may
/// rise mid-night); confirm it at a candidate instant with [`kiddush_levana_sayable_at`]. Keeping this
/// to the window∩night bracket avoids a fragile moonrise-in-interval search; `_sayable_at` is the
/// authoritative all-three test.
#[allow(clippy::too_many_arguments)]
pub fn kiddush_levana_interval_on_night(
    night_ref_jd: f64,
    year: i32,
    month: u8,
    site: &Site,
    night_depression_deg: f64,
    start: KiddushLevanaStart,
    end: KiddushLevanaEnd,
    optics: &crate::params::Optics,
) -> Option<(AbsoluteInstant, AbsoluteInstant)> {
    let (open, close) = kiddush_levana_window(year, month, start, end);
    let dusk = read_instant(
        site,
        night_ref_jd,
        ReadSpec::DepressionAngle {
            angle_deg: night_depression_deg,
            dir: Direction::Setting,
        },
        optics,
    )?;
    let dawn = read_instant(
        site,
        night_ref_jd + 1.0,
        ReadSpec::DepressionAngle {
            angle_deg: night_depression_deg,
            dir: Direction::Rising,
        },
        optics,
    )?;
    let lo = open.unix_nanos.max(dusk.unix_nanos);
    let hi = close.unix_nanos.min(dawn.unix_nanos);
    if lo >= hi {
        None
    } else {
        Some((
            AbsoluteInstant { unix_nanos: lo },
            AbsoluteInstant { unix_nanos: hi },
        ))
    }
}

/// The Hebrew date on which tal-u-matar (ve'ten tal u-matar / she'elat geshamim) begins (coupling #4),
/// under the basis knob. Pure F3 integer arithmetic — **always computable** (never does-not-occur):
/// - `Fixed7Cheshvan` (Eretz Yisrael): 7 Cheshvan (month 8, day 7).
/// - `TekufaBased` (diaspora): the 60th day after Tekufat Tishrei of `year` (the tekufa day counts as
///   day 1, so day 60 = tekufa RD + 59), via the arithmetic `tekufa` engine.
///
/// `year` is the **Hebrew** year. `realm` selects `basis` upstream (the spec treats `tal_umatar.basis`
/// as the F3 knob), so it is not threaded here — pass the basis your locale follows.
pub fn tal_umatar_start_date(year: i32, basis: TalUmatarBasis, method: TekufaMethod) -> HebrewDate {
    match basis {
        TalUmatarBasis::Fixed7Cheshvan => HebrewDate {
            year,
            month: 8,
            day: 7,
        },
        TalUmatarBasis::TekufaBased => {
            let (tekufa_rd, hour, _, _) = tekufa_civil(year, Season::Tishrei, method);
            // The counting-day rolls at 18:00 mean time: a tekufa after nightfall belongs to the next
            // day (this is the Dec-5-vs-Dec-6 mechanism — the year-before-civil-leap 9 PM case). The
            // tekufa day counts as day 1, so the start (60th day) is day 1 + 59.
            let day1 = tekufa_rd.0 + if hour >= 18 { 1 } else { 0 };
            hebrew_from_fixed(RataDie(day1 + 59))
        }
    }
}

/// The absolute instant tal-u-matar begins (coupling #4): recitation starts at maariv **entering** the
/// start date, i.e. the day-boundary instant on the civil day before it. `None` if that boundary
/// does-not-occur (high-latitude gate). The date itself is always available from
/// [`tal_umatar_start_date`] — separated so an edge that only needs the date never fails.
pub fn tal_umatar_start_instant(
    year: i32,
    basis: TalUmatarBasis,
    method: TekufaMethod,
    site: &Site,
    boundary: ReadSpec,
    optics: &crate::params::Optics,
) -> ZmanResult {
    let date = tal_umatar_start_date(year, basis, method);
    let date_rd = fixed_from_hebrew(date).0;
    // The Hebrew date begins at the boundary of the previous civil day.
    read_instant(
        site,
        local_noon_jd(date_rd - 1, site.lon_deg),
        boundary,
        optics,
    )
}
