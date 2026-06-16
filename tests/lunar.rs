//! F2 (lunar geometry) validation against `tests/fixtures/lunar_vectors.csv` (Wolfram-sourced).
//!
//! Two row kinds:
//! - `alt`   — apparent topocentric lunar altitude at a fixed UTC instant vs Wolfram `MoonPosition`
//!   (the primary, fully-independent test: it exercises the ELP-2000/82 series + topocentric
//!   parallax + refraction). Tolerance in **degrees**.
//! - `moonrise`/`moonset` — the rise/set UTC time. The oracle time is Wolfram's apparent-altitude
//!   curve scanned to the same horizon target the engine uses (`−(semidiameter + dip)` at the dipped
//!   horizon); Wolfram supplies the independent *position*, so agreement validates the event *time*,
//!   not the definition (the Phase-0 reciprocal pattern). Tolerance in **seconds**, tied to /0003.
//!
//! tz/UTC handling is in the harness (the edge, ADR core-domain/0007); the core stays tz-free.
//! Run with output: `cargo test --test lunar -- --nocapture`.

use chrono::DateTime;
use core_engine::events::{moon_rise_set, Direction};
use core_engine::lunar::moon_altitude_deg;
use core_engine::optics::RefractionModel;
use core_engine::params::Optics;
use core_engine::time::jd_from_unix_secs;
use core_engine::units::GeometricAltitude;
use core_engine::{AbsoluteInstant, Site};

const FIXTURE: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/lunar_vectors.csv"
);

fn parse_utc_secs(s: &str) -> i64 {
    // "YYYY-MM-DDTHH:MM:SS" as UTC.
    DateTime::parse_from_rfc3339(&format!("{s}Z"))
        .unwrap_or_else(|_| panic!("bad UTC datetime {s}"))
        .timestamp()
}

#[test]
fn f2_lunar_vectors() {
    let data = std::fs::read_to_string(FIXTURE).expect("fixture present");
    let mut lines = data.lines();
    lines.next(); // header
    let (mut pass, mut fail) = (0u32, 0u32);
    let mut out: Vec<String> = Vec::new();

    for line in lines {
        if line.trim().is_empty() {
            continue;
        }
        let f: Vec<&str> = line.splitn(9, ',').collect();
        let (kind, site_name, lat, lon, elev, arg_utc, expected, tol) =
            (f[0], f[1], f[2], f[3], f[4], f[5], f[6], f[7]);
        let site = Site {
            lat_deg: lat.parse().unwrap(),
            lon_deg: lon.parse().unwrap(),
            elev_m: elev.parse().unwrap(),
        };
        let tol: f64 = tol.parse().unwrap();

        let (ok, got_str, resid) = match kind {
            "alt" => {
                let jd = jd_from_unix_secs(parse_utc_secs(arg_utc) as f64);
                let geo = moon_altitude_deg(jd, &site);
                let app = RefractionModel::Bennett
                    .apparent(GeometricAltitude(geo))
                    .deg();
                let exp: f64 = expected.parse().unwrap();
                let r = app - exp;
                (
                    r.abs() <= tol,
                    format!("{app:.5}deg"),
                    format!("{:+.5}deg ({:+.0}\")", r, r * 3600.0),
                )
            }
            "moonrise" | "moonset" => {
                let day_start = jd_from_unix_secs(parse_utc_secs(arg_utc) as f64);
                let dir = if kind == "moonrise" {
                    Direction::Rising
                } else {
                    Direction::Setting
                };
                let exp_ts = parse_utc_secs(expected);
                match moon_rise_set(&site, day_start, dir, &Optics::default()) {
                    Some(ai) => {
                        let eng = ai.unix_nanos.div_euclid(1_000_000_000);
                        let r = eng - exp_ts;
                        (r.abs() <= tol as i64, render_utc(ai), format!("{r:+}s"))
                    }
                    None => (false, "does-not-occur".into(), String::new()),
                }
            }
            _ => (false, "UNKNOWN-KIND".into(), String::new()),
        };

        if ok {
            pass += 1;
        } else {
            fail += 1;
        }
        out.push(format!(
            "{} {kind:8} {site_name:9} exp={expected:20} got={got_str:22} {resid}  // {}",
            if ok { "OK" } else { "!!" },
            f[8]
        ));
    }

    eprintln!("\n=== F2 lunar vectors (Wolfram MoonPosition; alt in deg, rise/set in s) ===");
    for r in &out {
        eprintln!("{r}");
    }
    eprintln!("\nF2: {pass} ok, {fail} fail");
    assert_eq!(
        fail, 0,
        "{fail} F2 lunar row(s) missed the oracle; see table above"
    );
}

fn render_utc(ai: AbsoluteInstant) -> String {
    DateTime::from_timestamp_nanos(ai.unix_nanos)
        .format("%Y-%m-%dT%H:%M:%S")
        .to_string()
}
