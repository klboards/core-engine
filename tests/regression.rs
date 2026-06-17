//! Golden regression snapshot (ADR core-domain/0017 hardening). A committed corpus of current engine
//! outputs across F1/F2/F3 + couplings; the test recomputes the grid and asserts **exact** equality
//! against the snapshot, so *any* code change that moves *any* value is caught. This guards against
//! silent **change**, not correctness (correctness is the oracle suites) — complementary to the
//! FP-determinism gate (which guards native==wasm) and the property suite (which guards invariants).
//!
//! Regenerate deliberately after an intended change: `BLESS_SNAPSHOT=1 cargo test --test regression`.

use core_engine::calendar::{
    classify_day, festival_date, hebrew_from_fixed, molad_chalakim, Festival, HebrewDate, RataDie,
};
use core_engine::couplings::{
    hebrew_date_at_instant, tal_umatar_start_date, DayRoll, DEFAULT_DAY_BOUNDARY,
};
use core_engine::events::{read_instant, terrain_horizon_crossing, Bound, Direction, ReadSpec};
use core_engine::kiddush_levana::kiddush_levana_window;
use core_engine::params::{Optics, Realm, TalUmatarBasis, TekufaMethod};
use core_engine::tekufa::{tekufa_civil, Season};
use core_engine::time::jd_from_gregorian;
use core_engine::wire::HorizonProfile;
use core_engine::{AbsoluteInstant, Site};

const SNAPSHOT: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/regression_snapshot.csv"
);

fn zr(r: Option<AbsoluteInstant>) -> String {
    r.map(|a| a.unix_nanos.to_string())
        .unwrap_or_else(|| "DNO".to_string())
}
fn hd(d: HebrewDate) -> String {
    format!("{}/{}/{}", d.year, d.month, d.day)
}
fn dc_flags(c: core_engine::calendar::DayClass) -> String {
    let mut p: Vec<&str> = Vec::new();
    for (b, n) in [
        (c.shabbat, "shabbat"),
        (c.yom_tov, "yom_tov"),
        (c.chol_hamoed, "chol_hamoed"),
        (c.erev, "erev"),
        (c.rosh_chodesh, "rosh_chodesh"),
        (c.fast_day, "fast_day"),
    ] {
        if b {
            p.push(n);
        }
    }
    if p.is_empty() {
        "none".into()
    } else {
        p.join("|")
    }
}

/// The deterministic grid — the single source of truth for both blessing and checking.
fn rows() -> Vec<(String, String)> {
    let mut out: Vec<(String, String)> = Vec::new();
    let o = Optics::default();
    let sites: &[(f64, f64, f64, &str)] = &[
        (31.778, 35.2354, 754.0, "jeru"),
        (40.7128, -74.006, 10.0, "nyc"),
        (-33.8688, 151.2093, 58.0, "sydney"),
        (22.30327, 98.50521, 1868.39, "highelev"),
    ];
    let dates: &[(i32, u8, u8)] = &[(2026, 3, 20), (2026, 6, 21), (2026, 12, 21)];
    let reads: &[(&str, ReadSpec)] = &[
        (
            "netz",
            ReadSpec::HorizonCrossing {
                dir: Direction::Rising,
            },
        ),
        (
            "shkia",
            ReadSpec::HorizonCrossing {
                dir: Direction::Setting,
            },
        ),
        ("chatzot", ReadSpec::ExtremumMidpoint),
        (
            "alot16",
            ReadSpec::DepressionAngle {
                angle_deg: 16.1,
                dir: Direction::Rising,
            },
        ),
        (
            "tzeit85",
            ReadSpec::DepressionAngle {
                angle_deg: 8.5,
                dir: Direction::Setting,
            },
        ),
        (
            "shma_gra",
            ReadSpec::Proportional {
                fraction: 0.25,
                start: Bound::Netz,
                end: Bound::Shkia,
            },
        ),
    ];
    for &(lat, lon, elev, sn) in sites {
        let site = Site {
            lat_deg: lat,
            lon_deg: lon,
            elev_m: elev,
        };
        for &(y, m, d) in dates {
            let ref_jd = jd_from_gregorian(y, m as u32, d as f64 + 0.5) - lon / 360.0;
            for (rn, spec) in reads {
                out.push((
                    format!("f1.{sn}.{y}{m:02}{d:02}.{rn}"),
                    zr(read_instant(&site, ref_jd, *spec, &o)),
                ));
            }
        }
    }
    // F3: Hebrew conversion, molad, classification, festivals.
    for rd in [730000i64, 738000, 739000, 740000, 745000] {
        out.push((format!("f3.heb.{rd}"), hd(hebrew_from_fixed(RataDie(rd)))));
    }
    for (y, m) in [(5786, 7), (5787, 7), (5787, 1), (5788, 8)] {
        out.push((
            format!("f3.molad.{y}.{m}"),
            molad_chalakim(y, m).to_string(),
        ));
    }
    for (y, m, d) in [(5787, 7, 16), (5787, 7, 10), (5787, 9, 25), (5786, 1, 16)] {
        let date = HebrewDate {
            year: y,
            month: m,
            day: d,
        };
        for (realm, rn) in [(Realm::EretzYisrael, "ey"), (Realm::Diaspora, "dia")] {
            out.push((
                format!("f3.class.{y}.{m}.{d}.{rn}"),
                dc_flags(classify_day(date, realm)),
            ));
        }
    }
    for (f, fname) in [
        (Festival::RoshHashanah, "rh"),
        (Festival::Pesach, "pesach"),
        (Festival::Chanukah, "chanukah"),
    ] {
        out.push((
            format!("f3.fest.5787.{fname}"),
            festival_date(5787, f).0.to_string(),
        ));
    }
    // Tekufa.
    for y in [5786, 5787, 5788] {
        for (s, sn) in [(Season::Nisan, "nisan"), (Season::Tishrei, "tishrei")] {
            for (meth, mn) in [(TekufaMethod::Shmuel, "shm"), (TekufaMethod::RavAda, "ada")] {
                let (rd, h, mi, ch) = tekufa_civil(y, s, meth);
                out.push((
                    format!("tekufa.{y}.{sn}.{mn}"),
                    format!("{}/{}/{}/{}", rd.0, h, mi, ch),
                ));
            }
        }
    }
    // Couplings: tal-u-matar dates, KL window, day-roll.
    for y in [5786, 5787, 5788] {
        for (b, bn) in [
            (TalUmatarBasis::TekufaBased, "tk"),
            (TalUmatarBasis::Fixed7Cheshvan, "fx"),
        ] {
            out.push((
                format!("c.talumatar.{y}.{bn}"),
                hd(tal_umatar_start_date(y, b, TekufaMethod::Shmuel)),
            ));
        }
    }
    for (y, m) in [(5786, 8), (5787, 7), (5787, 9)] {
        let (open, close) = kiddush_levana_window(
            y,
            m,
            core_engine::params::KiddushLevanaStart::ThreeDays,
            core_engine::params::KiddushLevanaEnd::HalfMonth,
        );
        out.push((
            format!("c.kl.{y}.{m}"),
            format!("{}..{}", open.unix_nanos, close.unix_nanos),
        ));
    }
    let jeru = Site {
        lat_deg: 31.778,
        lon_deg: 35.2354,
        elev_m: 754.0,
    };
    for secs in [1_765_000_000i64, 1_766_000_000, 1_767_000_000] {
        let t = AbsoluteInstant {
            unix_nanos: secs * 1_000_000_000,
        };
        let r = match hebrew_date_at_instant(t, &jeru, DEFAULT_DAY_BOUNDARY, &o) {
            DayRoll::Resolved(d) => hd(d),
            DayRoll::BoundaryDoesNotOccur => "DNO".into(),
        };
        out.push((format!("c.dayroll.{secs}"), r));
    }
    // TerrainProfile crossings (coupling-free /0018 moat path): a constant +0.3° synthetic skyline
    // (72 samples, 18000 milliarcminutes) at two sites — locks the azimuth-dependent terrain solver.
    let mut angles: Vec<u8> = Vec::new();
    for _ in 0..72 {
        angles.extend_from_slice(&18_000i32.to_le_bytes());
    }
    for &(lat, lon, sn) in &[(31.778, 35.2354, "jeru"), (40.7128, -74.006, "nyc")] {
        let site = Site {
            lat_deg: lat,
            lon_deg: lon,
            elev_m: 0.0,
        };
        let hp = HorizonProfile {
            lat_microdeg: (lat * 1.0e6) as i32,
            lon_microdeg: (lon * 1.0e6) as i32,
            elev_mm: 0,
            dem_source: 1,
            dem_version: 1,
            prov_refraction_model: 0,
            prov_refraction_coeff_micro: None,
            angles_mam: &angles,
        };
        let ref_jd = jd_from_gregorian(2026, 6, 21.5) - lon / 360.0;
        for (dir, dn) in [(Direction::Rising, "netz"), (Direction::Setting, "shkia")] {
            out.push((
                format!("terrain.{sn}.{dn}"),
                zr(terrain_horizon_crossing(
                    &site,
                    ref_jd,
                    dir,
                    &Optics::default(),
                    &hp,
                )),
            ));
        }
    }
    out
}

#[test]
fn regression_snapshot_matches() {
    let rows = rows();
    if std::env::var("BLESS_SNAPSHOT").is_ok() {
        let body: String = rows.iter().map(|(k, v)| format!("{k},{v}\n")).collect();
        std::fs::write(SNAPSHOT, format!("id,value\n{body}")).expect("write snapshot");
        eprintln!("BLESSED {} rows → {SNAPSHOT}", rows.len());
        return;
    }
    let data = std::fs::read_to_string(SNAPSHOT)
        .expect("snapshot present (regenerate with BLESS_SNAPSHOT=1)");
    let mut expected = std::collections::HashMap::new();
    for line in data.lines().skip(1) {
        if let Some((k, v)) = line.split_once(',') {
            expected.insert(k.to_string(), v.to_string());
        }
    }
    let (mut pass, mut fail) = (0u32, 0u32);
    for (k, v) in &rows {
        match expected.get(k) {
            Some(e) if e == v => pass += 1,
            Some(e) => {
                fail += 1;
                eprintln!("!! {k}: snapshot={e} got={v}");
            }
            None => {
                fail += 1;
                eprintln!("!! {k}: missing from snapshot");
            }
        }
    }
    assert_eq!(rows.len(), expected.len(), "row-count drift vs snapshot");
    eprintln!(
        "regression snapshot: {pass} ok, {fail} fail / {} rows",
        rows.len()
    );
    assert_eq!(
        fail, 0,
        "{fail} value(s) drifted from the committed snapshot"
    );
}
