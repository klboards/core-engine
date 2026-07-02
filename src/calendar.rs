//! F3 — Hebrew calendar (Dershowitz–Reingold fixed arithmetic), ADR core-domain/0001 + /0014.
//!
//! The post-Hillel-II calendar is a **fixed arithmetic** system (Rambam, *Hilchot Kiddush
//! HaChodesh*): exact integer, no floats, no observation — so determinism is **structural**.
//! Conversions pivot on the Rata-Die day number. Month numbering: Nisan=1 … Tishrei=7 …
//! Adar=12 (common); Adar I=12, Adar II=13 (leap) — matching Wolfram + D–R.
//! Validated against Wolfram "Jewish" calendar + Hebcal cross-check.

use crate::params::{AdarAnniversaryRule, Realm};
use crate::AbsoluteInstant;

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

// ── Day-type classification (coupling #2, ADR core-domain/0016). Pure F3 integer arithmetic: a
// Hebrew date + realm → a `DayClass` of co-holding flags. The token *selects* which solar reads
// matter (candle-lighting on erev, fast start/end on fasts) — that selection is an edge concern;
// the core only emits the token. Realm gates Yom Tov Sheni (the diaspora second festival day).

/// A single dominant day-kind token (coupling #2), by the precedence
/// `YomTov > Shabbat > CholHaMoed > FastDay > RoshChodesh > Erev > Weekday`. Lossy by design —
/// [`DayClass`] is the full answer (a day can be e.g. both Shabbat and Rosh Chodesh).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum DayKind {
    /// Ordinary weekday.
    Weekday,
    /// Shabbat.
    Shabbat,
    /// A melacha-forbidden festival day (incl. the diaspora second day, and Yom Kippur).
    YomTov,
    /// Chol HaMoed (intermediate days of Pesach / Sukkot).
    CholHaMoed,
    /// Erev Shabbat or Erev Yom Tov (the day whose evening enters a Shabbat/Yom-Tov).
    Erev,
    /// Rosh Chodesh.
    RoshChodesh,
    /// A public fast day.
    FastDay,
}

/// The full classification of one Hebrew day as a set of **co-holding** flags (a day can be both
/// Shabbat and Rosh Chodesh, or Yom Kippur which is both `yom_tov` and `fast_day`). Realm-gated.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct DayClass {
    /// Shabbat (Saturday by the Hebrew day).
    pub shabbat: bool,
    /// A melacha-forbidden festival day (incl. diaspora Yom Tov Sheni, and Yom Kippur).
    pub yom_tov: bool,
    /// Chol HaMoed.
    pub chol_hamoed: bool,
    /// Erev Shabbat / Erev Yom Tov (candle-lighting relevance).
    pub erev: bool,
    /// Rosh Chodesh.
    pub rosh_chodesh: bool,
    /// A public fast day (Gedaliah / 10 Tevet / Ta'anit Esther / 17 Tammuz / 9 Av / Yom Kippur),
    /// with the standard Shabbat-deferral (nidcheh) applied.
    pub fast_day: bool,
}

impl DayClass {
    /// The dominant [`DayKind`] for display selection, by the documented precedence.
    pub fn primary(self) -> DayKind {
        if self.yom_tov {
            DayKind::YomTov
        } else if self.shabbat {
            DayKind::Shabbat
        } else if self.chol_hamoed {
            DayKind::CholHaMoed
        } else if self.fast_day {
            DayKind::FastDay
        } else if self.rosh_chodesh {
            DayKind::RoshChodesh
        } else if self.erev {
            DayKind::Erev
        } else {
            DayKind::Weekday
        }
    }
}

/// Day-of-week from a Rata Die: 0 = Sunday … 6 = Saturday (RD 1 = 0001-01-01 = Monday).
pub fn weekday_from_fixed(rd: RataDie) -> u8 {
    amod(rd.0, 7) as u8
}

/// Is `(month, day)` a melacha-forbidden festival day under `realm`? (Months: Nisan=1 … Tishrei=7.)
/// The diaspora-only second days (Yom Tov Sheni) are gated by `realm` — the realm parameter, not a
/// per-stream code path (ADR core-domain/0002).
fn is_yom_tov_md(month: u8, day: u8, realm: Realm) -> bool {
    let diaspora = matches!(realm, Realm::Diaspora);
    match (month, day) {
        // Tishrei: RH (2 days everywhere), Yom Kippur, Sukkot day 1, Shmini Atzeret.
        (7, 1) | (7, 2) | (7, 10) | (7, 15) | (7, 22) => true,
        // Sukkot day 2 / Simchat Torah — diaspora only.
        (7, 16) | (7, 23) => diaspora,
        // Pesach day 1 & day 7; Shavuot day 1.
        (1, 15) | (1, 21) | (3, 6) => true,
        // Pesach day 2 & day 8; Shavuot day 2 — diaspora only.
        (1, 16) | (1, 22) | (3, 7) => diaspora,
        _ => false,
    }
}

/// Is `(month, day)` Chol HaMoed under `diaspora`? Sukkot: EY 16–21, diaspora 17–21 (day 16 is
/// Yom Tov Sheni). Pesach: EY 16–20, diaspora 17–20 (day 16 is Yom Tov Sheni; day 21 is the 7th-day
/// Yom Tov, day 22 the 8th in the diaspora).
fn is_chol_hamoed_md(month: u8, day: u8, diaspora: bool) -> bool {
    let lo = if diaspora { 17 } else { 16 };
    match month {
        7 => day >= lo && day <= 21,
        1 => day >= lo && day <= 20,
        _ => false,
    }
}

/// True if the day at `rd` is itself Shabbat or a Yom Tov (used to flag the *erev* before it).
fn is_yt_or_shabbat(rd: RataDie, realm: Realm) -> bool {
    if weekday_from_fixed(rd) == 6 {
        return true;
    }
    let d = hebrew_from_fixed(rd);
    is_yom_tov_md(d.month, d.day, realm)
}

/// Observed RD of a fast whose nominal date is `(base_month, base_day)`, applying the Shabbat
/// deferral: most fasts push **forward** to Sunday; Ta'anit Esther pushes **back** to Thursday
/// (`defer_forward = false`). A fast never falls on Shabbat (it is moved), so comparing against this
/// observed RD yields the correct single day.
fn fast_observed(year: i32, base_month: u8, base_day: u8, defer_forward: bool) -> RataDie {
    let base = fixed_from_hebrew(HebrewDate {
        year,
        month: base_month,
        day: base_day,
    });
    if weekday_from_fixed(base) == 6 {
        RataDie(if defer_forward {
            base.0 + 1
        } else {
            base.0 - 2
        })
    } else {
        base
    }
}

/// Classify the Hebrew day at `date` under `realm` (coupling #2). Pure integer arithmetic — no F1,
/// no instant; the caller supplies the Hebrew date (already day-rolled by coupling #1). Realm gates
/// Yom Tov Sheni and the Chol HaMoed boundary. Fasts carry the standard Shabbat-deferral.
pub fn classify_day(date: HebrewDate, realm: Realm) -> DayClass {
    let rd = fixed_from_hebrew(date);
    let diaspora = matches!(realm, Realm::Diaspora);
    let last_month = last_month_of_year(date.year);
    DayClass {
        shabbat: weekday_from_fixed(rd) == 6,
        yom_tov: is_yom_tov_md(date.month, date.day, realm),
        chol_hamoed: is_chol_hamoed_md(date.month, date.day, diaspora),
        // Rosh Chodesh: the 1st of every month except Tishrei (whose 1st is Rosh Hashanah), plus the
        // 30th of any 30-day month (the first of a two-day Rosh Chodesh).
        rosh_chodesh: date.day == 30 || (date.day == 1 && date.month != 7),
        erev: is_yt_or_shabbat(RataDie(rd.0 + 1), realm),
        fast_day: (date.month == 7 && date.day == 10) // Yom Kippur
            || rd == fast_observed(date.year, 7, 3, true) // Tzom Gedaliah (3 Tishrei → Sun)
            || rd
                == fixed_from_hebrew(HebrewDate {
                    year: date.year,
                    month: 10,
                    day: 10,
                }) // Asarah b'Tevet (never deferred)
            || rd == fast_observed(date.year, last_month, 13, false) // Ta'anit Esther (→ Thu)
            || rd == fast_observed(date.year, 4, 17, true) // 17 Tammuz (→ Sun)
            || rd == fast_observed(date.year, 5, 9, true), // 9 Av (→ Sun)
    }
}

/// A specific moed (festival / fast / special day) identity — the fine-grained companion to
/// [`DayClass`]'s six coarse flags, so a board can gate content on *which* chag it is (Chanukah vs
/// Pesach vs a fast), not merely "Yom Tov". Structural tokens; localized names are downstream (the
/// management/content layer). Pure F3 integer arithmetic — the reverse of [`classify_day`]'s predicates,
/// with **no new astronomy and no correctness degree**. Mutually exclusive **by date** (one festival
/// identity per day); co-holding periods (Sefiras HaOmer, the Three Weeks / Nine Days) are surfaced
/// separately by the caller, not here. Realm gates only the diaspora-second-day identities.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[allow(missing_docs)]
pub enum Moed {
    ErevRoshHashana,
    RoshHashana,
    TzomGedaliah,
    ErevYomKippur,
    YomKippur,
    ErevSukkos,
    Sukkos,
    CholHamoedSukkos,
    HoshanaRabba,
    ShminiAtzeres,
    SimchasTorah,
    Chanukah,
    AsaraBeteves,
    TuBishvat,
    TaanisEsther,
    Purim,
    ShushanPurim,
    ErevPesach,
    Pesach,
    CholHamoedPesach,
    PesachSheni,
    LagBaomer,
    ErevShavuos,
    Shavuos,
    TzomTammuz,
    ErevTishaBav,
    TishaBav,
    TuBeav,
}

/// The specific [`Moed`] identity of the Hebrew day at `date` under `realm`, or `None` for an ordinary
/// day. One identity per date (Yom Tov days map to their chag; Chol HaMoed to the chag's CH"M; the
/// erev-of-chag and the fasts to themselves), reusing the same predicates as [`classify_day`] so the two
/// never drift. Realm gates the diaspora-only second-day identities (Simchas Torah, Pesach 8th).
pub fn moed_of(date: HebrewDate, realm: Realm) -> Option<Moed> {
    let rd = fixed_from_hebrew(date);
    let (m, d) = (date.month, date.day);
    let last = last_month_of_year(date.year);
    let diaspora = matches!(realm, Realm::Diaspora);

    // Melacha-forbidden festival days → their chag identity (mirrors is_yom_tov_md's (month,day) set).
    if is_yom_tov_md(m, d, realm) {
        return Some(match (m, d) {
            (7, 1) | (7, 2) => Moed::RoshHashana,
            (7, 10) => Moed::YomKippur,
            (7, 15) | (7, 16) => Moed::Sukkos,
            (7, 22) => Moed::ShminiAtzeres, // in EY this day is also Simchas Torah (combined)
            (7, 23) => Moed::SimchasTorah,  // diaspora only (is_yom_tov_md gates it on realm)
            (1, 15) | (1, 16) | (1, 21) | (1, 22) => Moed::Pesach,
            (3, 6) | (3, 7) => Moed::Shavuos,
            _ => return None, // unreachable given is_yom_tov_md's set
        });
    }
    // Hoshana Rabba (21 Tishrei) is a distinct day within CH"M Sukkos — check before the CH"M fallthrough.
    if m == 7 && d == 21 {
        return Some(Moed::HoshanaRabba);
    }
    if is_chol_hamoed_md(m, d, diaspora) {
        return Some(if m == 7 {
            Moed::CholHamoedSukkos
        } else {
            Moed::CholHamoedPesach
        });
    }
    // Erev-of-chag (more specific than DayClass::erev).
    match (m, d) {
        (6, 29) => return Some(Moed::ErevRoshHashana), // 29 Elul
        (7, 9) => return Some(Moed::ErevYomKippur),
        (7, 14) => return Some(Moed::ErevSukkos),
        (1, 14) => return Some(Moed::ErevPesach),
        (3, 5) => return Some(Moed::ErevShavuos),
        _ => {}
    }
    // Public fasts (observed, with the same deferral classify_day uses).
    if rd == fast_observed(date.year, 7, 3, true) {
        return Some(Moed::TzomGedaliah);
    }
    if m == 10 && d == 10 {
        return Some(Moed::AsaraBeteves); // never deferred
    }
    if rd == fast_observed(date.year, last, 13, false) {
        return Some(Moed::TaanisEsther);
    }
    if rd == fast_observed(date.year, 4, 17, true) {
        return Some(Moed::TzomTammuz);
    }
    // Tisha B'Av + its erev (both relative to the OBSERVED fast, so a nidcheh year shifts them together).
    let tisha_bav = fast_observed(date.year, 5, 9, true);
    if rd == tisha_bav {
        return Some(Moed::TishaBav);
    }
    if rd.0 == tisha_bav.0 - 1 {
        return Some(Moed::ErevTishaBav);
    }
    // Chanukah spans the Kislev→Tevet boundary (8 days from 25 Kislev); Kislev 29/30 handled by RD math.
    let chanukah_start = fixed_from_hebrew(HebrewDate {
        year: date.year,
        month: 9,
        day: 25,
    })
    .0;
    if (0..8).contains(&(rd.0 - chanukah_start)) {
        return Some(Moed::Chanukah);
    }
    // Purim / Shushan Purim: 14 / 15 of the last month (Adar, or Adar II in a leap year).
    if m == last && d == 14 {
        return Some(Moed::Purim);
    }
    if m == last && d == 15 {
        return Some(Moed::ShushanPurim);
    }
    // Other rabbinic / minor named days (fixed month+day).
    match (m, d) {
        (11, 15) => return Some(Moed::TuBishvat),  // 15 Shevat
        (2, 14) => return Some(Moed::PesachSheni), // 14 Iyar
        (2, 18) => return Some(Moed::LagBaomer),   // 18 Iyar
        (5, 15) => return Some(Moed::TuBeav),      // 15 Av
        _ => {}
    }
    None
}

/// True if `date` is within the **Three Weeks** mourning period (17 Tammuz … 9 Av, nominal). A
/// co-holding period token (it overlaps the fasts at its endpoints); surfaced alongside a [`Moed`], not
/// instead of one. Realm-invariant.
pub fn in_three_weeks(date: HebrewDate) -> bool {
    let rd = fixed_from_hebrew(date).0;
    let start = fixed_from_hebrew(HebrewDate {
        year: date.year,
        month: 4,
        day: 17,
    })
    .0;
    let end = fixed_from_hebrew(HebrewDate {
        year: date.year,
        month: 5,
        day: 9,
    })
    .0;
    (start..=end).contains(&rd)
}

/// True if `date` is within the **Nine Days** (1 Av … 9 Av, nominal) — the stricter sub-period of the
/// Three Weeks. Co-holding; surfaced alongside a [`Moed`].
pub fn in_nine_days(date: HebrewDate) -> bool {
    let rd = fixed_from_hebrew(date).0;
    let start = fixed_from_hebrew(HebrewDate {
        year: date.year,
        month: 5,
        day: 1,
    })
    .0;
    let end = fixed_from_hebrew(HebrewDate {
        year: date.year,
        month: 5,
        day: 9,
    })
    .0;
    (start..=end).contains(&rd)
}

// ── Molad (mean lunar conjunction) — the F3 deferral from ADR core-domain/0014, needed by F2's
// Kiddush Levana. The molad is *calendar arithmetic* (the fixed mean conjunction), distinct from
// the observed moon (F2 proper); it stays here in F3. Exact integer in the molad's own mean-time
// frame; only the final UT projection (`molad_instant`) is float and carries a meridian assumption.

/// Chalakim (1/1080 hour) per day.
pub const CHALAKIM_PER_DAY: i64 = 25_920;
/// Chalakim in one mean synodic month = 29d 12h 793p (= 29 × 25920 + 12 × 1080 + 793).
pub const CHALAKIM_PER_MONTH: i64 = 765_433;
/// D–R molad epoch offset: the molad of Tishrei AM 1 (BaHaRaD) is `HEBREW_EPOCH − 876/25920` (RD).
const MOLAD_EPOCH_OFFSET_CHALAKIM: i64 = 876;
/// Julian Day at RD 0, 00:00 (RD 1 = proleptic-Gregorian 0001-01-01 = JD 1_721_425.5).
const RD0_JULIAN_DAY: f64 = 1_721_424.5;

/// **ASSUMPTION (flagged finding, ADR core-domain/0015):** the meridian whose *local mean solar
/// time* the molad reckoning is referenced to, for projecting the molad to UT. Taken as Jerusalem
/// (35.2354°E ≈ the traditional 2h21m offset). The molad's day-of-week + hour + chalakim are
/// meridian-free and exact; ONLY the absolute-UT projection depends on this constant.
pub const MOLAD_MERIDIAN_DEG_EAST: f64 = 35.2354;

/// Lunar months elapsed from the BaHaRaD epoch to the molad of `(year, month)` (D–R molad).
fn molad_months_elapsed(year: i32, month: u8) -> i64 {
    // The year number rolls at Tishrei (m7); Nisan..Elul (m<7) belong to the half-year that began
    // the *previous* Tishrei, so they count against (year + 1).
    let y = if month < 7 {
        year as i64 + 1
    } else {
        year as i64
    };
    (month as i64 - 7) + quotient(235 * y - 234, 19)
}

/// Molad of `(year, month)` as **exact-integer chalakim since RD 0** in the molad mean-time frame
/// (no float, no meridian — structurally deterministic). The interval between consecutive molads is
/// exactly [`CHALAKIM_PER_MONTH`].
pub fn molad_chalakim(year: i32, month: u8) -> i64 {
    HEBREW_EPOCH * CHALAKIM_PER_DAY - MOLAD_EPOCH_OFFSET_CHALAKIM
        + molad_months_elapsed(year, month) * CHALAKIM_PER_MONTH
}

/// Molad of `(year, month)` rendered in its mean-time frame as `(civil RD, hour, minute, chalakim)`
/// — meridian-free and exact (hour 0..24 from civil midnight). This is the canonical, citable form
/// (a molad table's day/hour/chalakim derive from it).
pub fn molad_civil(year: i32, month: u8) -> (RataDie, u8, u8, u16) {
    let ch = molad_chalakim(year, month);
    let rd = ch.div_euclid(CHALAKIM_PER_DAY);
    let in_day = ch.rem_euclid(CHALAKIM_PER_DAY);
    let hour = in_day / 1_080;
    let rem = in_day % 1_080;
    (RataDie(rd), hour as u8, (rem / 18) as u8, (rem % 18) as u16)
}

/// Project an exact-integer **chalakim-since-RD-0** count (molad mean-time frame) to an absolute UT
/// instant via [`MOLAD_MERIDIAN_DEG_EAST`] — the one float/assumption step (see that constant's
/// note). Shared by the molad ([`molad_instant`]) and the arithmetic tekufa (`tekufa`), so coupling
/// #4 introduces **no new** geo assumption. Extracted from `molad_instant` byte-for-byte (the molad
/// FP probe, kind 10, is the regression gate, ADR core-domain/0016).
pub fn chalakim_to_instant(chalakim_since_rd0: i64) -> AbsoluteInstant {
    let molad_days = chalakim_since_rd0 as f64 / CHALAKIM_PER_DAY as f64;
    let jd_mean_time = molad_days + RD0_JULIAN_DAY;
    let jd_ut = jd_mean_time - MOLAD_MERIDIAN_DEG_EAST / 15.0 / 24.0;
    AbsoluteInstant::from_julian_day(jd_ut)
}

/// Molad of `(year, month)` as an absolute UT instant (ADR core-domain/0001). The exact-integer
/// chalakim are projected from the molad mean-time frame to UT via [`chalakim_to_instant`]
/// (the one float/assumption step — see [`MOLAD_MERIDIAN_DEG_EAST`]).
pub fn molad_instant(year: i32, month: u8) -> AbsoluteInstant {
    chalakim_to_instant(molad_chalakim(year, month))
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

/// Sefiras-haomer day count (1..=49) on the Hebrew `date`, or `None` outside the omer. Day 1 is 16
/// Nisan (the night after the first day of Pesach) through day 49 = 5 Sivan; realm-invariant. A board
/// calendar object (ADR core-domain/0022); offline integer arithmetic.
pub fn omer_day(date: HebrewDate) -> Option<u16> {
    let start = fixed_from_hebrew(HebrewDate {
        year: date.year,
        month: 1,
        day: 16,
    })
    .0;
    let n = fixed_from_hebrew(date).0 - start;
    if (0..49).contains(&n) {
        Some((n + 1) as u16)
    } else {
        None
    }
}

// ── Modern Israeli national days (ADR core-domain/0022). Fixed Hebrew dates whose OBSERVANCE is shifted
// off the nominal day by the Knesset rules to avoid Shabbat desecration. Realm/community opt-in — a
// board surfaces these only for communities that observe them; the arithmetic itself is realm-free.
// Weekday encoding (weekday_from_fixed = amod(rd,7)): Sun=0, Mon=1 … Thu=4, Fri=5, Sat=6.

/// A modern Israeli national day.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum IsraeliDay {
    /// Yom HaShoah — Holocaust Remembrance (nominal 27 Nisan).
    YomHaShoah,
    /// Yom HaZikaron — Memorial Day (nominal 4 Iyar).
    YomHaZikaron,
    /// Yom HaAtzmaut — Independence Day (the day after Yom HaZikaron).
    YomHaAtzmaut,
    /// Yom Yerushalayim — Jerusalem Day (28 Iyar, unshifted).
    YomYerushalayim,
}

/// Observed RataDie of Yom HaShoah in `year` (nominal 27 Nisan): Friday → −1 (Thursday); Sunday → +1
/// (Monday); else unchanged.
fn yom_hashoah_observed(year: i32) -> RataDie {
    let base = fixed_from_hebrew(HebrewDate {
        year,
        month: 1,
        day: 27,
    });
    match weekday_from_fixed(base) {
        5 => RataDie(base.0 - 1), // Friday → Thursday
        0 => RataDie(base.0 + 1), // Sunday → Monday
        _ => base,
    }
}

/// Observed RataDie of Yom HaZikaron in `year` (nominal 4 Iyar): Thursday → −1, Friday → −2 (both to
/// Wednesday), Sunday → +1 (Monday); else unchanged. Yom HaAtzmaut is always the following day.
fn yom_hazikaron_observed(year: i32) -> RataDie {
    let base = fixed_from_hebrew(HebrewDate {
        year,
        month: 2,
        day: 4,
    });
    match weekday_from_fixed(base) {
        4 => RataDie(base.0 - 1), // Thursday → Wednesday
        5 => RataDie(base.0 - 2), // Friday → Wednesday
        0 => RataDie(base.0 + 1), // Sunday → Monday
        _ => base,
    }
}

/// The Israeli national day observed on `date`, if any — the Knesset Shabbat-shift applied. Opt-in per
/// community (a board surfaces it only where observed); the computation is realm-independent.
pub fn israeli_national_day(date: HebrewDate) -> Option<IsraeliDay> {
    let rd = fixed_from_hebrew(date);
    let zikaron = yom_hazikaron_observed(date.year);
    if rd == yom_hashoah_observed(date.year) {
        Some(IsraeliDay::YomHaShoah)
    } else if rd == zikaron {
        Some(IsraeliDay::YomHaZikaron)
    } else if rd.0 == zikaron.0 + 1 {
        Some(IsraeliDay::YomHaAtzmaut)
    } else if rd
        == fixed_from_hebrew(HebrewDate {
            year: date.year,
            month: 2,
            day: 28,
        })
    {
        Some(IsraeliDay::YomYerushalayim)
    } else {
        None
    }
}
