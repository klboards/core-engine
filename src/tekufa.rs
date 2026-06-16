//! Arithmetic tekufa (seasons) — ADR core-domain/0016, coupling #4. **Pure calendar arithmetic, not
//! astronomy:** Shmuel's tekufa is the Julian 365¼-day construct (year = 4 × 91d 7h 30m), Julian-locked
//! and drifting against the Gregorian/true equinox; Rav Ada's is the 19-year Metonic mean (235 synodic
//! months / 19). Both are exact integers — worked in **regaim** (1/76 of a chalakim) because Rav Ada's
//! year (365d 5h 997p 48 regaim = 235·765433/19 chalakim) is not a whole number of chalakim. The single
//! float is the UT projection, which reuses the molad's [`crate::calendar::chalakim_to_instant`] — so
//! coupling #4 introduces **no new** meridian/geo assumption.
//!
//! Reusable for Birkat Hachama (Tekufat Nisan on Shmuel's 28-year cycle).
//!
//! **Anchor (flagged, ADR core-domain/0016):** the creation phase — Tekufat Nisan of year 1 — is taken
//! from the Rambam relation to the BaHaRaD molad and then **calibrated to the universally-published
//! Tekufat Tishrei (≈ 7 October, Gregorian, in the current era) / tal-u-matar Dec-4/5 dates**. Only the
//! integer phase is pinned this way; the season length (the "physics") is fixed and exact, so this is a
//! phase calibration, not a fit. The **Shmuel** branch is oracle-validated (the tal-u-matar fixtures);
//! the **Rav-Ada** branch shares the same creation anchor and uses the exact Metonic year length but is
//! **not yet independently oracle-validated** — surfaced, not buried.

use crate::calendar::{chalakim_to_instant, molad_chalakim, RataDie, CHALAKIM_PER_DAY};
use crate::params::TekufaMethod;
use crate::AbsoluteInstant;

/// Regaim (the smallest tekufa unit) per chalakim.
const REGAIM_PER_CHALAKIM: i64 = 76;
/// Regaim per day = 76 × 25920.
const REGAIM_PER_DAY: i64 = REGAIM_PER_CHALAKIM * CHALAKIM_PER_DAY;

/// Shmuel's solar year in regaim = 365¼ d = 9_467_280 chalakim × 76.
const SHMUEL_YEAR_REGAIM: i64 = 9_467_280 * REGAIM_PER_CHALAKIM;
/// Rav Ada's solar year in regaim = 235 synodic months / 19 = 235 × 765433 × 76 / 19 (exact).
const RAV_ADA_YEAR_REGAIM: i64 = 235 * 765_433 * REGAIM_PER_CHALAKIM / 19;

/// Tekufat Nisan of year 1, in regaim-since-RD-0 (molad mean-time frame). Rambam's relation places it
/// 7d 9h 642p before the BaHaRaD molad (= `molad_chalakim(1, 7)`); `ANCHOR_CALIBRATION_REGAIM` then
/// pins the integer phase to the published Tekufat Tishrei (see module note). Shared by both methods.
const TEKUFAT_NISAN_Y1_OFFSET_CHALAKIM: i64 = 7 * CHALAKIM_PER_DAY + 9 * 1_080 + 642;
/// Additive phase calibration (regaim) — see module note. Pins Tekufat Tishrei (Shmuel) of HY 5787 to
/// 2026-10-07 15:00:00 mean time (3 PM — the published value; HY mod 4 = 3 in the 3am/9am/3pm/9pm
/// cycle, cross-checked against the Birkat-Hachama-2009 anchor, HY 5770 ≈ Oct 7 9 AM). Only the integer
/// phase is pinned; the Julian-locked season length is exact, so this is calibration, not a fit.
const ANCHOR_CALIBRATION_REGAIM: i64 = -370_475_832;

fn tekufat_nisan_y1_regaim() -> i64 {
    (molad_chalakim(1, 7) - TEKUFAT_NISAN_Y1_OFFSET_CHALAKIM) * REGAIM_PER_CHALAKIM
        + ANCHOR_CALIBRATION_REGAIM
}

/// The four seasons (tekufot): Nisan = vernal, Tammuz = summer, Tishrei = autumnal, Tevet = winter.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Season {
    /// Vernal (spring) tekufa — Nisan.
    Nisan,
    /// Summer tekufa — Tammuz.
    Tammuz,
    /// Autumnal tekufa — Tishrei (the one tal-u-matar counts from).
    Tishrei,
    /// Winter tekufa — Tevet.
    Tevet,
}

/// Season ordinal within the tekufa year, counting from Tekufat Nisan (0 = Nisan … 3 = Tevet).
fn season_offset(season: Season) -> i64 {
    match season {
        Season::Nisan => 0,
        Season::Tammuz => 1,
        Season::Tishrei => 2,
        Season::Tevet => 3,
    }
}

fn year_regaim(method: TekufaMethod) -> i64 {
    match method {
        TekufaMethod::Shmuel => SHMUEL_YEAR_REGAIM,
        TekufaMethod::RavAda => RAV_ADA_YEAR_REGAIM,
    }
}

/// Regaim-since-RD-0 of `(year, season)`'s tekufa under `method` — exact integer.
fn tekufa_regaim(year: i32, season: Season, method: TekufaMethod) -> i64 {
    // season length = year / 4; the year is divisible by 4 in regaim for both methods.
    let season_len = year_regaim(method) / 4;
    let season_index = (year as i64 - 1) * 4 + season_offset(season);
    tekufat_nisan_y1_regaim() + season_index * season_len
}

/// Tekufa of `(year, season)` as **exact-integer chalakim since RD 0** (molad mean-time frame). For
/// Rav Ada this truncates a sub-chalakim (≤ 75/76) remainder; the exact date is from [`tekufa_civil`].
pub fn tekufa_chalakim(year: i32, season: Season, method: TekufaMethod) -> i64 {
    tekufa_regaim(year, season, method).div_euclid(REGAIM_PER_CHALAKIM)
}

/// Tekufa of `(year, season)` rendered meridian-free as `(civil RD, hour, minute, chalakim)` — exact,
/// the citable form (mirrors [`crate::calendar::molad_civil`]). Computed from regaim so Rav Ada is exact.
pub fn tekufa_civil(year: i32, season: Season, method: TekufaMethod) -> (RataDie, u8, u8, u16) {
    let rg = tekufa_regaim(year, season, method);
    let rd = rg.div_euclid(REGAIM_PER_DAY);
    let in_day_ch = rg.rem_euclid(REGAIM_PER_DAY) / REGAIM_PER_CHALAKIM; // chalakim into the day
    let hour = in_day_ch / 1_080;
    let rem = in_day_ch % 1_080;
    (RataDie(rd), hour as u8, (rem / 18) as u8, (rem % 18) as u16)
}

/// Tekufa of `(year, season)` as an absolute UT instant. The one float step — reuses the molad's
/// projection ([`chalakim_to_instant`]), so no new meridian assumption (flagged, ADR core-domain/0016).
pub fn tekufa_instant(year: i32, season: Season, method: TekufaMethod) -> AbsoluteInstant {
    chalakim_to_instant(tekufa_chalakim(year, season, method))
}
