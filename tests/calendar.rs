//! F3 (Hebrew calendar) validation against `tests/fixtures/calendar_vectors.csv` — values sourced
//! from Wolfram "Jewish" calendar and cross-checked against Hebcal (ADR core-domain/0014).
//! Exact integer comparison (no tolerance; F3 is fixed arithmetic).
//!
//! Run with output: `cargo test --test calendar -- --nocapture`.

use core_engine::calendar::{
    festival_date, fixed_from_gregorian, gregorian_from_fixed, hebrew_from_fixed,
    hebrew_year_length, is_hebrew_leap_year, molad_chalakim, molad_civil, molad_instant, yahrzeit,
    Festival, HebrewDate, RataDie, CHALAKIM_PER_MONTH,
};
use core_engine::kiddush_levana::kiddush_levana_window;
use core_engine::params::{AdarAnniversaryRule, KiddushLevanaEnd, KiddushLevanaStart};

const FIXTURE: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/calendar_vectors.csv"
);

fn parse_greg(s: &str) -> (i32, u8, u8) {
    let p: Vec<&str> = s.split('-').collect();
    (
        p[0].parse().unwrap(),
        p[1].parse().unwrap(),
        p[2].parse().unwrap(),
    )
}
fn parse_heb(s: &str) -> HebrewDate {
    let p: Vec<&str> = s.split('/').collect();
    HebrewDate {
        year: p[0].parse().unwrap(),
        month: p[1].parse().unwrap(),
        day: p[2].parse().unwrap(),
    }
}
fn fest(s: &str) -> Festival {
    match s {
        "RoshHashanah" => Festival::RoshHashanah,
        "YomKippur" => Festival::YomKippur,
        "Sukkot" => Festival::Sukkot,
        "Pesach" => Festival::Pesach,
        "Shavuot" => Festival::Shavuot,
        "Purim" => Festival::Purim,
        "Chanukah" => Festival::Chanukah,
        _ => panic!("unknown festival {s}"),
    }
}
fn rule(s: &str) -> AdarAnniversaryRule {
    match s {
        "AdarII" => AdarAnniversaryRule::AdarII,
        "AdarI" => AdarAnniversaryRule::AdarI,
        "Both" => AdarAnniversaryRule::Both,
        _ => panic!("unknown rule {s}"),
    }
}
fn gstr(rd: RataDie) -> String {
    let (y, m, d) = gregorian_from_fixed(rd);
    format!("{y:04}-{m:02}-{d:02}")
}
fn parse_ym(s: &str) -> (i32, u8) {
    let p: Vec<&str> = s.split('/').collect();
    (p[0].parse().unwrap(), p[1].parse().unwrap())
}

#[test]
fn f3_calendar_vectors() {
    let data = std::fs::read_to_string(FIXTURE).expect("fixture present");
    let mut lines = data.lines();
    lines.next(); // header
    let (mut pass, mut fail) = (0u32, 0u32);
    let mut out: Vec<String> = Vec::new();

    for line in lines {
        if line.trim().is_empty() {
            continue;
        }
        let f: Vec<&str> = line.splitn(6, ',').collect();
        let (kind, a, b, c, expected) = (f[0], f[1], f[2], f[3], f[4]);
        let got = match kind {
            "greg2heb" => {
                let (y, m, d) = parse_greg(a);
                let h = hebrew_from_fixed(fixed_from_gregorian(y, m, d));
                format!("{}/{}/{}", h.year, h.month, h.day)
            }
            "leap" => format!("{}", is_hebrew_leap_year(a.parse().unwrap())),
            "len" => format!("{}", hebrew_year_length(a.parse().unwrap())),
            "festival" => gstr(festival_date(a.parse().unwrap(), fest(b))),
            "yahrzeit" => gstr(yahrzeit(parse_heb(a), b.parse().unwrap(), rule(c))),
            "molad_ch" => {
                let (y, m) = parse_ym(a);
                format!("{}", molad_chalakim(y, m))
            }
            _ => "UNKNOWN-KIND".to_string(),
        };
        let ok = got == expected;
        if ok {
            pass += 1;
        } else {
            fail += 1;
        }
        out.push(format!(
            "{} {kind:9} {a:12} exp={expected:12} got={got}",
            if ok { "OK" } else { "!!" }
        ));
    }

    eprintln!("\n=== F3 calendar vectors (exact; Wolfram + Hebcal sourced) ===");
    for r in &out {
        eprintln!("{r}");
    }
    eprintln!("\nF3: {pass} ok, {fail} fail");
    assert_eq!(
        fail, 0,
        "{fail} F3 calendar row(s) mismatched; see table above"
    );
}

/// Molad: the canonical **BaHaRaD** anchor (5h 204 chalakim into Monday night) + the exact synodic
/// interval (765433 chalakim) — together these pin every molad. Then the Kiddush Levana window
/// offsets under the default knobs (ADR core-domain/0015).
#[test]
fn f3_molad_and_kiddush_levana() {
    // BaHaRaD: molad of Tishrei AM 1, in the mean-time-frame civil rendering, is 23:11 + 6 chalakim
    // (= 6 PM + 5h + 204 chalakim — the classic "2-5-204", Monday-night halachic day).
    let (_, h, mi, ch) = molad_civil(1, 7);
    assert_eq!((h, mi, ch), (23, 11, 6), "BaHaRaD time-of-day mismatch");

    // Exact synodic interval between consecutive molads (Tishrei → Cheshvan, contiguous numbering),
    // across several years incl. a leap year — must be exactly 765433 chalakim by construction.
    for y in [5785, 5786, 5787] {
        assert_eq!(
            molad_chalakim(y, 8) - molad_chalakim(y, 7),
            CHALAKIM_PER_MONTH,
            "synodic interval must be exactly 765433 chalakim (year {y})"
        );
    }

    // Kiddush Levana window: earliest = molad + 3 days; latest = molad + ½ synodic (Rema defaults).
    let molad = molad_instant(5786, 7).unix_nanos;
    let (e0, e1) = kiddush_levana_window(
        5786,
        7,
        KiddushLevanaStart::ThreeDays,
        KiddushLevanaEnd::HalfMonth,
    );
    let npd: f64 = 86_400.0 * 1.0e9;
    let half_days = CHALAKIM_PER_MONTH as f64 / 2.0 / 25_920.0;
    assert_eq!(
        e0.unix_nanos,
        molad + (3.0 * npd).round() as i64,
        "KL earliest = molad + 3 days"
    );
    assert_eq!(
        e1.unix_nanos,
        molad + (half_days * npd).round() as i64,
        "KL latest = molad + half synodic month"
    );
    assert!(e1.unix_nanos > e0.unix_nanos, "KL window must be non-empty");
    eprintln!(
        "molad/KL: BaHaRaD 23:11+6ch ✓, interval 765433 ✓, KL window [molad+3d, molad+½month] ✓"
    );
}
