//! F3-class arithmetic tekufa + tal-u-matar (coupling #4, ADR core-domain/0016), validated against
//! `tests/fixtures/tekufa_vectors.csv`. The Shmuel branch is pinned to the universally-published
//! Tekufat Tishrei (≈ Oct 7) and tal-u-matar (Dec 4/5/6) dates — independent of our engine and
//! non-circular. Exact comparison (the tekufa is exact integer arithmetic; the dates are external).
//!
//! Run with output: `cargo test --test tekufa -- --nocapture`.

use core_engine::calendar::{fixed_from_hebrew, gregorian_from_fixed};
use core_engine::couplings::tal_umatar_start_date;
use core_engine::params::{TalUmatarBasis, TekufaMethod};
use core_engine::tekufa::{tekufa_civil, Season};

const FIXTURE: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/tekufa_vectors.csv"
);

fn season(s: &str) -> Season {
    match s {
        "Nisan" => Season::Nisan,
        "Tammuz" => Season::Tammuz,
        "Tishrei" => Season::Tishrei,
        "Tevet" => Season::Tevet,
        _ => panic!("unknown season {s}"),
    }
}
fn method(s: &str) -> TekufaMethod {
    match s {
        "shmuel" => TekufaMethod::Shmuel,
        "ravada" => TekufaMethod::RavAda,
        _ => panic!("unknown method {s}"),
    }
}
fn basis(s: &str) -> TalUmatarBasis {
    match s {
        "tekufabased" => TalUmatarBasis::TekufaBased,
        "fixed7cheshvan" => TalUmatarBasis::Fixed7Cheshvan,
        _ => panic!("unknown basis {s}"),
    }
}

#[test]
fn f3_tekufa_and_tal_umatar() {
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
        let year: i32 = a.parse().unwrap();
        let got = match kind {
            "tekufa_greg" => {
                let (rd, _, _, _) = tekufa_civil(year, season(b), method(c));
                let (y, m, d) = gregorian_from_fixed(rd);
                format!("{y:04}-{m:02}-{d:02}")
            }
            "tekufa_hm" => {
                let (_, h, mi, _) = tekufa_civil(year, season(b), method(c));
                format!("{h:02}:{mi:02}")
            }
            "talumatar" => {
                let date = tal_umatar_start_date(year, basis(b), method(c));
                let (y, m, d) = gregorian_from_fixed(fixed_from_hebrew(date));
                format!("{y:04}-{m:02}-{d:02}")
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
            "{} {kind:13} {a} {b:14} exp={expected:12} got={got}",
            if ok { "OK" } else { "!!" }
        ));
    }

    eprintln!("\n=== F3 tekufa + tal-u-matar (exact; published Oct-7 / Dec-4-5-6 sourced) ===");
    for r in &out {
        eprintln!("{r}");
    }
    eprintln!("\ntekufa/tal-u-matar: {pass} ok, {fail} fail");
    assert_eq!(fail, 0, "{fail} tekufa row(s) mismatched; see table above");
}
