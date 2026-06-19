//! Israeli national-days differential vs **Hebcal** (ADR core-domain/0022). The committed fixture is
//! Hebcal's observed Gregorian dates over 2025–2033 — a span that exercises every Knesset Shabbat-shift
//! branch (Yom HaShoah Fri→−1 / Sun→+1; Yom HaZikaron Thu→−1 / Fri→−2 / Sun→+1; and unshifted years).
//! Exact (deterministic calendar arithmetic, no tolerance).

use core_engine::calendar::{
    fixed_from_gregorian, hebrew_from_fixed, israeli_national_day, omer_day, IsraeliDay,
};

const FIXTURE: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/israeli_days_vectors.csv"
);

fn parse(tag: &str) -> IsraeliDay {
    match tag {
        "shoah" => IsraeliDay::YomHaShoah,
        "zikaron" => IsraeliDay::YomHaZikaron,
        "atzmaut" => IsraeliDay::YomHaAtzmaut,
        "yerushalayim" => IsraeliDay::YomYerushalayim,
        other => panic!("unknown israeli-day tag '{other}'"),
    }
}

#[test]
fn israeli_days_vs_hebcal() {
    let data = std::fs::read_to_string(FIXTURE).expect("israeli-days fixture present");
    let (mut pass, mut fail) = (0u32, 0u32);
    for line in data.lines().skip(1) {
        if line.trim().is_empty() {
            continue;
        }
        let f: Vec<&str> = line.split(',').collect();
        let dp: Vec<&str> = f[0].split('-').collect();
        let (y, m, d): (i32, u8, u8) = (
            dp[0].parse().unwrap(),
            dp[1].parse().unwrap(),
            dp[2].parse().unwrap(),
        );
        let want = parse(f[1]);
        let hd = hebrew_from_fixed(fixed_from_gregorian(y, m, d));
        let got = israeli_national_day(hd);
        if got == Some(want) {
            pass += 1;
        } else {
            fail += 1;
            eprintln!("!! {} : got {got:?} want {want:?}", f[0]);
        }
    }
    eprintln!("israeli-days vs Hebcal: {pass} ok, {fail} fail");
    assert_eq!(fail, 0, "{fail} Israeli-day date(s) diverged from Hebcal");
}

#[test]
fn ordinary_days_are_none() {
    // A weekday in the middle of the year is not any Israeli national day.
    for (y, m, d) in [(2026, 1, 15), (2026, 7, 1), (2026, 12, 25)] {
        let hd = hebrew_from_fixed(fixed_from_gregorian(y, m, d));
        assert_eq!(israeli_national_day(hd), None, "{y}-{m}-{d} should be None");
    }
}

#[test]
fn omer_count_spans_16_nisan_to_5_sivan() {
    use core_engine::calendar::{fixed_from_hebrew, HebrewDate};
    // Day 1 = 16 Nisan; day 49 = 5 Sivan; before/after = None. (5786, realm-invariant.)
    let omer = |month, day| {
        omer_day(HebrewDate {
            year: 5786,
            month,
            day,
        })
    };
    assert_eq!(omer(1, 16), Some(1));
    assert_eq!(omer(3, 5), Some(49));
    assert_eq!(omer(1, 15), None); // first day of Pesach — not yet counting
    assert_eq!(omer(3, 6), None); // Shavuos
                                  // Internal consistency: day N is N days after 16 Nisan.
    let d33 = hebrew_from_fixed(core_engine::calendar::RataDie(
        fixed_from_hebrew(HebrewDate {
            year: 5786,
            month: 1,
            day: 16,
        })
        .0 + 32,
    ));
    assert_eq!(omer_day(d33), Some(33)); // Lag BaOmer
}
