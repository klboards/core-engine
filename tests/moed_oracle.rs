//! F3 moed-identity (`moed_of`) validation against the **Hebcal oracle** (holidays for the Gregorian
//! year 2026, spanning Hebrew years 5786→5787; diaspora observance). Grounds the fine-grained moed gate
//! that the board's day-type/rotation gating consumes. Exact integer comparison (no tolerance).
//!
//! Dates are asserted by Hebrew (year, month, day) constructed directly, cross-checked to the Hebcal
//! Gregorian dates in the comments. Realm = Diaspora (the oracle table shows Yom Tov Sheni / Simchas
//! Torah on 23 Tishrei / Pesach VIII). Run: `cargo test --test moed_oracle -- --nocapture`.

use core_engine::calendar::{in_nine_days, in_three_weeks, moed_of, HebrewDate, Moed};
use core_engine::params::Realm;

fn hd(year: i32, month: u8, day: u8) -> HebrewDate {
    HebrewDate { year, month, day }
}

#[test]
fn moed_of_matches_hebcal_2026() {
    let d = Realm::Diaspora;
    // month numbering: Nisan=1 … Elul=6, Tishrei=7, Cheshvan=8, Kislev=9, Tevet=10, Shevat=11, Adar=12.
    let cases: &[(HebrewDate, Option<Moed>)] = &[
        // ── Tishrei 5787 (Rosh Hashana window) ──
        (hd(5786, 6, 29), Some(Moed::ErevRoshHashana)), // 2026-09-11 Erev Rosh Hashana
        (hd(5787, 7, 1), Some(Moed::RoshHashana)),      // 2026-09-12 Rosh Hashana
        (hd(5787, 7, 2), Some(Moed::RoshHashana)),      // 2026-09-13 Rosh Hashana II
        (hd(5787, 7, 3), Some(Moed::TzomGedaliah)),     // 2026-09-14 Tzom Gedaliah (not deferred)
        (hd(5787, 7, 9), Some(Moed::ErevYomKippur)),    // 2026-09-20 Erev Yom Kippur
        (hd(5787, 7, 10), Some(Moed::YomKippur)),       // 2026-09-21 Yom Kippur
        (hd(5787, 7, 14), Some(Moed::ErevSukkos)),      // 2026-09-25 Erev Sukkot
        (hd(5787, 7, 15), Some(Moed::Sukkos)),          // 2026-09-26 Sukkot I
        (hd(5787, 7, 16), Some(Moed::Sukkos)),          // 2026-09-27 Sukkot II (diaspora YT)
        (hd(5787, 7, 17), Some(Moed::CholHamoedSukkos)), // 2026-09-28 Sukkot III (CH"M)
        (hd(5787, 7, 20), Some(Moed::CholHamoedSukkos)), // 2026-10-01 Sukkot VI (CH"M)
        (hd(5787, 7, 21), Some(Moed::HoshanaRabba)),    // 2026-10-02 Hoshana Raba
        (hd(5787, 7, 22), Some(Moed::ShminiAtzeres)),   // 2026-10-03 Shmini Atzeret
        (hd(5787, 7, 23), Some(Moed::SimchasTorah)),    // 2026-10-04 Simchat Torah (diaspora)
        (hd(5787, 10, 10), Some(Moed::AsaraBeteves)),   // 2026-12-20 Asara B'Tevet
        // ── Chanukah 5787 (25 Kislev … 2 Tevet; Kislev is 30 days this year) ──
        (hd(5787, 9, 25), Some(Moed::Chanukah)),        // 2026-12-05 Chanukah day 1 (2 candles)
        (hd(5787, 9, 30), Some(Moed::Chanukah)),        // 2026-12-10 Chanukah (7 candles)
        (hd(5787, 10, 2), Some(Moed::Chanukah)),        // 2026-12-12 Chanukah 8th day
        (hd(5787, 10, 3), None),                        // 3 Tevet — after Chanukah
        // ── Shevat / Adar 5786 ──
        (hd(5786, 11, 15), Some(Moed::TuBishvat)),      // 2026-02-02 Tu BiShvat
        (hd(5786, 12, 13), Some(Moed::TaanisEsther)),   // 2026-03-02 Ta'anit Esther (not deferred)
        (hd(5786, 12, 14), Some(Moed::Purim)),          // 2026-03-03 Purim
        (hd(5786, 12, 15), Some(Moed::ShushanPurim)),   // 2026-03-04 Shushan Purim
        // ── Nisan 5786 (Pesach) ──
        (hd(5786, 1, 14), Some(Moed::ErevPesach)),      // 2026-04-01 Erev Pesach
        (hd(5786, 1, 15), Some(Moed::Pesach)),          // 2026-04-02 Pesach I
        (hd(5786, 1, 16), Some(Moed::Pesach)),          // 2026-04-03 Pesach II (diaspora YT)
        (hd(5786, 1, 17), Some(Moed::CholHamoedPesach)), // 2026-04-04 Pesach III (CH"M)
        (hd(5786, 1, 20), Some(Moed::CholHamoedPesach)), // 2026-04-07 Pesach VI (CH"M)
        (hd(5786, 1, 21), Some(Moed::Pesach)),          // 2026-04-08 Pesach VII
        (hd(5786, 1, 22), Some(Moed::Pesach)),          // 2026-04-09 Pesach VIII (diaspora)
        // ── Iyar / Sivan 5786 ──
        (hd(5786, 2, 14), Some(Moed::PesachSheni)),     // 2026-05-01 Pesach Sheni
        (hd(5786, 2, 18), Some(Moed::LagBaomer)),       // 2026-05-05 Lag BaOmer
        (hd(5786, 3, 5), Some(Moed::ErevShavuos)),      // 2026-05-21 Erev Shavuot
        (hd(5786, 3, 6), Some(Moed::Shavuos)),          // 2026-05-22 Shavuot I
        (hd(5786, 3, 7), Some(Moed::Shavuos)),          // 2026-05-23 Shavuot II (diaspora)
        // ── Tammuz / Av 5786 (the fasts) ──
        (hd(5786, 4, 17), Some(Moed::TzomTammuz)),      // 2026-07-02 Tzom Tammuz
        (hd(5786, 5, 8), Some(Moed::ErevTishaBav)),     // 2026-07-22 Erev Tish'a B'Av
        (hd(5786, 5, 9), Some(Moed::TishaBav)),         // 2026-07-23 Tish'a B'Av
        (hd(5786, 5, 15), Some(Moed::TuBeav)),          // 2026-07-29 Tu B'Av
        // ── ordinary days ──
        (hd(5786, 8, 5), None),  // a plain Cheshvan weekday
        (hd(5787, 11, 3), None), // a plain Shevat weekday
    ];
    for (date, want) in cases {
        assert_eq!(
            moed_of(*date, d),
            *want,
            "moed_of({:?}) mismatch",
            date
        );
    }
}

#[test]
fn eretz_yisrael_has_no_second_days() {
    // In EY, the diaspora-second-day identities collapse: 23 Tishrei and Pesach VIII are ordinary.
    let ey = Realm::EretzYisrael;
    assert_eq!(moed_of(hd(5787, 7, 23), ey), None, "no Simchas Torah day in EY");
    assert_eq!(moed_of(hd(5786, 1, 22), ey), None, "no Pesach VIII in EY");
    // But the shared festival days still hold.
    assert_eq!(moed_of(hd(5787, 7, 22), ey), Some(Moed::ShminiAtzeres));
    assert_eq!(moed_of(hd(5786, 1, 21), ey), Some(Moed::Pesach));
    // EY Sukkos CH"M starts a day earlier (16 Tishrei is CH"M, not Yom Tov Sheni).
    assert_eq!(moed_of(hd(5787, 7, 16), ey), Some(Moed::CholHamoedSukkos));
}

#[test]
fn three_weeks_and_nine_days_ranges() {
    // Three Weeks: 17 Tammuz … 9 Av inclusive; Nine Days: 1 Av … 9 Av inclusive (5786).
    assert!(!in_three_weeks(hd(5786, 4, 16)), "day before 17 Tammuz");
    assert!(in_three_weeks(hd(5786, 4, 17)), "17 Tammuz starts the Three Weeks");
    assert!(in_three_weeks(hd(5786, 5, 1)), "1 Av is inside the Three Weeks");
    assert!(in_three_weeks(hd(5786, 5, 9)), "9 Av is the last day");
    assert!(!in_three_weeks(hd(5786, 5, 10)), "10 Av is after");

    assert!(!in_nine_days(hd(5786, 4, 29)), "29 Tammuz (last of Tammuz) is before the Nine Days");
    assert!(in_nine_days(hd(5786, 5, 1)), "1 Av starts the Nine Days");
    assert!(in_nine_days(hd(5786, 5, 9)), "9 Av is the last day");
    assert!(!in_nine_days(hd(5786, 5, 10)), "10 Av is after");
}
