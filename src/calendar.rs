//! F3 — Hebrew calendar (Dershowitz–Reingold fixed arithmetic), ADR core-domain/0001 + /0014.
//!
//! The post-Hillel-II calendar is a **fixed arithmetic** system (Rambam, *Hilchot Kiddush
//! HaChodesh*): exact integer, no floats, no observation — so determinism is **structural**.
//! Conversions pivot on the Rata-Die day number. Month numbering: Nisan=1 … Tishrei=7 …
//! Adar=12 (common); Adar I=12, Adar II=13 (leap) — matching Wolfram + D–R.
//! Validated against Wolfram "Jewish" calendar + Hebcal cross-check.

use crate::params::AdarAnniversaryRule;

/// Rata Die: integer day number; RD 1 = proleptic-Gregorian 0001-01-01. The conversion pivot.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct RataDie(pub i64);

/// A Hebrew calendar date. `month`: Nisan=1 … Tishrei=7 … Adar=12 (common); Adar I=12 / Adar II=13 (leap).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct HebrewDate {
    /// Anno Mundi year.
    pub year: i32,
    /// Month, Nisan=1 … Adar(II)=12/13.
    pub month: u8,
    /// Day of month, 1..=30.
    pub day: u8,
}

/// A festival / fast / Rosh Chodesh anchor — a structural token; localized names are downstream.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Festival {
    /// 1 Tishrei.
    RoshHashanah,
    /// 10 Tishrei.
    YomKippur,
    /// 15 Tishrei.
    Sukkot,
    /// 15 Nisan.
    Pesach,
    /// 6 Sivan.
    Shavuot,
    /// 14 Adar (common) / 14 Adar II (leap).
    Purim,
    /// 25 Kislev.
    Chanukah,
}

const HEBREW_EPOCH: i64 = -1_373_427;

#[inline]
fn quotient(a: i64, b: i64) -> i64 {
    a.div_euclid(b)
}
#[inline]
fn amod(a: i64, b: i64) -> i64 {
    a.rem_euclid(b)
}

#[inline]
fn gregorian_leap(y: i64) -> bool {
    (y % 4 == 0 && y % 100 != 0) || y % 400 == 0
}

/// RD of a proleptic-Gregorian date.
pub fn fixed_from_gregorian(year: i32, month: u8, day: u8) -> RataDie {
    let (y, m, d) = (year as i64, month as i64, day as i64);
    let corr = if m <= 2 {
        0
    } else if gregorian_leap(y) {
        -1
    } else {
        -2
    };
    RataDie(
        365 * (y - 1) + quotient(y - 1, 4) - quotient(y - 1, 100)
            + quotient(y - 1, 400)
            + quotient(367 * m - 362, 12)
            + corr
            + d,
    )
}

fn gregorian_year_from_fixed(rd: i64) -> i64 {
    let d0 = rd - 1; // GREGORIAN_EPOCH = 1
    let n400 = quotient(d0, 146_097);
    let d1 = amod(d0, 146_097);
    let n100 = quotient(d1, 36_524);
    let d2 = amod(d1, 36_524);
    let n4 = quotient(d2, 1_461);
    let d3 = amod(d2, 1_461);
    let n1 = quotient(d3, 365);
    let year = 400 * n400 + 100 * n100 + 4 * n4 + n1;
    if n100 == 4 || n1 == 4 {
        year
    } else {
        year + 1
    }
}

/// Proleptic-Gregorian `(year, month, day)` from RD.
pub fn gregorian_from_fixed(rd: RataDie) -> (i32, u8, u8) {
    let rd = rd.0;
    let year = gregorian_year_from_fixed(rd);
    let prior = rd - fixed_from_gregorian(year as i32, 1, 1).0;
    let corr = if rd < fixed_from_gregorian(year as i32, 3, 1).0 {
        0
    } else if gregorian_leap(year) {
        1
    } else {
        2
    };
    let month = quotient(12 * (prior + corr) + 373, 367);
    let day = rd - fixed_from_gregorian(year as i32, month as u8, 1).0 + 1;
    (year as i32, month as u8, day as u8)
}

/// 19-year Metonic leap rule.
pub fn is_hebrew_leap_year(year: i32) -> bool {
    amod(7 * (year as i64) + 1, 19) < 7
}

/// Last month number of the Hebrew year (12 common, 13 leap = Adar II).
pub fn last_month_of_year(year: i32) -> u8 {
    if is_hebrew_leap_year(year) {
        13
    } else {
        12
    }
}

fn hebrew_calendar_elapsed_days(year: i32) -> i64 {
    let months_elapsed = quotient(235 * (year as i64) - 234, 19);
    let parts_elapsed = 12_084 + 13_753 * months_elapsed;
    let days = 29 * months_elapsed + quotient(parts_elapsed, 25_920);
    // ADU partial + Molad-Zaken folded in here (D–R compact form).
    if amod(3 * (days + 1), 7) < 3 {
        days + 1
    } else {
        days
    }
}

fn hebrew_year_length_correction(year: i32) -> i64 {
    let ny0 = hebrew_calendar_elapsed_days(year - 1);
    let ny1 = hebrew_calendar_elapsed_days(year);
    let ny2 = hebrew_calendar_elapsed_days(year + 1);
    if ny2 - ny1 == 356 {
        2 // BeTUTaKPaT
    } else if ny1 - ny0 == 382 {
        1 // GaTaRaD
    } else {
        0
    }
}

/// RD of 1 Tishrei of the Hebrew `year` (Rosh Hashanah), all four dechiyot applied.
pub fn hebrew_new_year(year: i32) -> RataDie {
    RataDie(HEBREW_EPOCH + hebrew_calendar_elapsed_days(year) + hebrew_year_length_correction(year))
}

/// Days in the Hebrew year (353/354/355 common; 383/384/385 leap).
pub fn hebrew_year_length(year: i32) -> i64 {
    hebrew_new_year(year + 1).0 - hebrew_new_year(year).0
}

fn long_marcheshvan(year: i32) -> bool {
    matches!(hebrew_year_length(year), 355 | 385)
}
fn short_kislev(year: i32) -> bool {
    matches!(hebrew_year_length(year), 353 | 383)
}

/// Days in the given Hebrew month of the year.
pub fn last_day_of_month(year: i32, month: u8) -> u8 {
    match month {
        2 | 4 | 6 | 10 | 13 => 29,
        12 if !is_hebrew_leap_year(year) => 29, // Adar (common year)
        8 if !long_marcheshvan(year) => 29,     // Cheshvan
        9 if short_kislev(year) => 29,          // Kislev
        _ => 30,
    }
}

/// RD of a Hebrew date.
pub fn fixed_from_hebrew(date: HebrewDate) -> RataDie {
    let HebrewDate { year, month, day } = date;
    let mut rd = hebrew_new_year(year).0 + day as i64 - 1;
    if month < 7 {
        // Nisan..Elul are in the second (spring/summer) half of the Hebrew year's RD span.
        let last = last_month_of_year(year);
        let mut m = 7;
        while m <= last {
            rd += last_day_of_month(year, m) as i64;
            m += 1;
        }
        let mut m = 1;
        while m < month {
            rd += last_day_of_month(year, m) as i64;
            m += 1;
        }
    } else {
        let mut m = 7;
        while m < month {
            rd += last_day_of_month(year, m) as i64;
            m += 1;
        }
    }
    RataDie(rd)
}

/// Hebrew date from RD.
pub fn hebrew_from_fixed(rd: RataDie) -> HebrewDate {
    let approx = quotient((rd.0 - HEBREW_EPOCH) * 98_496, 35_975_351) + 1;
    let mut year = approx as i32;
    while hebrew_new_year(year + 1).0 <= rd.0 {
        year += 1;
    }
    while hebrew_new_year(year).0 > rd.0 {
        year -= 1;
    }
    let start = if rd
        < fixed_from_hebrew(HebrewDate {
            year,
            month: 1,
            day: 1,
        }) {
        7
    } else {
        1
    };
    let mut month = start;
    while rd
        > fixed_from_hebrew(HebrewDate {
            year,
            month,
            day: last_day_of_month(year, month),
        })
    {
        month += 1;
    }
    let day = (rd.0
        - fixed_from_hebrew(HebrewDate {
            year,
            month,
            day: 1,
        })
        .0
        + 1) as u8;
    HebrewDate { year, month, day }
}

/// RD of a festival/fast/RC anchor in the given Hebrew year.
pub fn festival_date(year: i32, fest: Festival) -> RataDie {
    let h = |m, d| HebrewDate {
        year,
        month: m,
        day: d,
    };
    match fest {
        Festival::RoshHashanah => hebrew_new_year(year),
        Festival::YomKippur => fixed_from_hebrew(h(7, 10)),
        Festival::Sukkot => fixed_from_hebrew(h(7, 15)),
        Festival::Pesach => fixed_from_hebrew(h(1, 15)),
        Festival::Shavuot => fixed_from_hebrew(h(3, 6)),
        Festival::Purim => fixed_from_hebrew(HebrewDate {
            year,
            month: last_month_of_year(year),
            day: 14,
        }),
        Festival::Chanukah => fixed_from_hebrew(h(9, 25)),
    }
}

/// Which month a death-in-Adar anniversary falls in, for the target year, under the knob.
fn adar_anniversary_month(death: HebrewDate, target_year: i32, rule: AdarAnniversaryRule) -> u8 {
    let target_leap = is_hebrew_leap_year(target_year);
    match (death.month, target_leap) {
        // Death in Adar of a COMMON year (m12), observed in a LEAP year → knob decides.
        (12, true) if !is_hebrew_leap_year(death.year) => match rule {
            AdarAnniversaryRule::AdarI => 12,
            // AdarII (default) and Both both anchor the single-date result in Adar II.
            AdarAnniversaryRule::AdarII | AdarAnniversaryRule::Both => 13,
        },
        // Death in Adar II (m13), observed in a COMMON year → the single Adar (m12).
        (13, false) => 12,
        (m, _) => m,
    }
}

/// Yahrzeit / anniversary date in `target_year` for a death on `death`, under the Adar knob
/// (ADR core-domain/0014). Handles the 30-Cheshvan / 30-Kislev short-month edges and clamps the
/// day to the target month's length. NOTE: `AdarAnniversaryRule::Both` returns the Adar II date
/// here (the primary observance); the dual-observance API is deferred.
pub fn yahrzeit(death: HebrewDate, target_year: i32, rule: AdarAnniversaryRule) -> RataDie {
    // 30 Cheshvan died in a year whose Cheshvan had 30 days, but target's predecessor is short →
    // observe 29 Cheshvan (= day before 1 Kislev).
    if death.month == 8 && death.day == 30 && !long_marcheshvan(target_year) {
        return RataDie(
            fixed_from_hebrew(HebrewDate {
                year: target_year,
                month: 9,
                day: 1,
            })
            .0 - 1,
        );
    }
    // 30 Kislev died long, target's Kislev is short → observe 29 Kislev (= day before 1 Tevet).
    if death.month == 9 && death.day == 30 && short_kislev(target_year) {
        return RataDie(
            fixed_from_hebrew(HebrewDate {
                year: target_year,
                month: 10,
                day: 1,
            })
            .0 - 1,
        );
    }
    let month = adar_anniversary_month(death, target_year, rule);
    let day = death.day.min(last_day_of_month(target_year, month));
    fixed_from_hebrew(HebrewDate {
        year: target_year,
        month,
        day,
    })
}
