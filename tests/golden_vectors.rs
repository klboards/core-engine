//! F1 oracle validation against `tests/fixtures/golden_vectors.csv` (Wolfram-verified).
//!
//! In scope: `def_type ∈ {depression_angle, horizon_crossing, extremum_midpoint, proportional}`.
//! `calendar_f3` rows are SKIPPED (F3, a later pass).
//!
//! tz/DST rendering and the civil-day (+1) label are done HERE, in the harness — the edge, per
//! ADR core-domain/0007. The core emits only timezone-free `Option<AbsoluteInstant>`.
//!
//! Run with output: `cargo test --test golden_vectors -- --nocapture`.

use chrono::{DateTime, Datelike, Duration, TimeZone, Timelike};
use chrono_tz::Tz;
use core_engine::params::Optics;
use core_engine::reads::{proportional_span_days, read_instant, Bound, Direction, ReadSpec};
use core_engine::{time, AbsoluteInstant, Site};

/// PROVISIONAL oracle tolerance. The CSV is minute-rounded → ±1 min this pass. The final
/// sub-second oracle tolerance is an OPEN question owned by ADR core-domain/0003.
const ORACLE_TOLERANCE_SECS: i64 = 60;

const FIXTURE: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/golden_vectors.csv"
);

/// MGA day bounds use the fixture's own alot/tzeitRT depression (−16.1°); set via the
/// `proportional_day_bounds` knob (data), not a code branch.
const MGA_DEG: f64 = 16.1;

enum Target {
    Instant(ReadSpec),
    Duration { start: Bound, end: Bound },
}

/// Parse the depression angle out of e.g. "sun_center=-16.1deg" → 16.1 (magnitude).
fn parse_angle(parameter: &str) -> Option<f64> {
    let eq = parameter.find('=')?;
    let rest = &parameter[eq + 1..];
    let end = rest.find("deg").unwrap_or(rest.len());
    rest[..end].trim().parse::<f64>().ok().map(f64::abs)
}

fn target_for(zman_key: &str, def_type: &str, parameter: &str) -> Option<Target> {
    match def_type {
        "depression_angle" => {
            let angle_deg = parse_angle(parameter)?;
            let dir = if zman_key.starts_with("tzeit") {
                Direction::Setting
            } else {
                Direction::Rising
            };
            Some(Target::Instant(ReadSpec::DepressionAngle {
                angle_deg,
                dir,
            }))
        }
        "horizon_crossing" => {
            let dir = match zman_key {
                "netz" => Direction::Rising,
                "shkia" => Direction::Setting,
                _ => return None,
            };
            Some(Target::Instant(ReadSpec::HorizonCrossing { dir }))
        }
        "extremum_midpoint" => Some(Target::Instant(ReadSpec::ExtremumMidpoint)),
        "proportional" => match zman_key {
            "shaah_zmanit_gra" => Some(Target::Duration {
                start: Bound::Netz,
                end: Bound::Shkia,
            }),
            "shaah_zmanit_mga" => Some(Target::Duration {
                start: Bound::Depression {
                    angle_deg: MGA_DEG,
                    dir: Direction::Rising,
                },
                end: Bound::Depression {
                    angle_deg: MGA_DEG,
                    dir: Direction::Setting,
                },
            }),
            "sof_zman_shma_gra" => Some(Target::Instant(ReadSpec::Proportional {
                fraction: 0.25,
                start: Bound::Netz,
                end: Bound::Shkia,
            })),
            "sof_zman_shma_mga" => Some(Target::Instant(ReadSpec::Proportional {
                fraction: 0.25,
                start: Bound::Depression {
                    angle_deg: MGA_DEG,
                    dir: Direction::Rising,
                },
                end: Bound::Depression {
                    angle_deg: MGA_DEG,
                    dir: Direction::Setting,
                },
            })),
            _ => None,
        },
        _ => None,
    }
}

/// ("HH:MM" or "HH:MM+1d") → ((hh, mm), civil-day offset).
fn parse_expected(s: &str) -> ((u32, u32), i64) {
    let (core, off) = match s.find("+1d") {
        Some(p) => (&s[..p], 1),
        None => (s, 0),
    };
    let core = core.trim();
    let mut it = core.split(':');
    let hh = it.next().unwrap().parse().unwrap();
    let mm = it.next().unwrap().parse().unwrap();
    ((hh, mm), off)
}

fn render_hhmm(ai: AbsoluteInstant, tz: Tz) -> String {
    let dt = DateTime::from_timestamp_nanos(ai.unix_nanos).with_timezone(&tz);
    format!("{:02}:{:02}", dt.hour(), dt.minute())
}

#[test]
fn f1_golden_vectors() {
    let data = std::fs::read_to_string(FIXTURE).expect("fixture present (prereq)");
    let mut lines = data.lines();
    lines.next(); // header

    let (mut pass, mut fail, mut skipped) = (0u32, 0u32, 0u32);
    let mut out: Vec<String> = Vec::new();

    for line in lines {
        if line.trim().is_empty() {
            continue;
        }
        let f: Vec<&str> = line.splitn(12, ',').collect();
        if f.len() < 11 {
            continue;
        }
        let (lat, lon, elev, tzname, date, zman_key, def_type, parameter, expected, status) =
            (f[1], f[2], f[3], f[4], f[5], f[6], f[7], f[8], f[9], f[10]);
        if def_type == "calendar_f3" {
            skipped += 1;
            continue;
        }

        let site = Site {
            lat_deg: lat.parse().unwrap(),
            lon_deg: lon.parse().unwrap(),
            elev_m: elev.parse().unwrap(),
        };
        let tz: Tz = tzname.parse().unwrap();
        let base = chrono::NaiveDate::parse_from_str(date, "%Y-%m-%d").unwrap();
        let anchor = tz
            .with_ymd_and_hms(base.year(), base.month(), base.day(), 12, 0, 0)
            .single()
            .unwrap();
        let ref_jd = time::jd_from_unix_secs(anchor.timestamp() as f64);
        let label = format!("{} {} {}", f[0], date, zman_key);

        let (ok, engine_str, resid, hypo) = match target_for(zman_key, def_type, parameter) {
            None => (
                false,
                "UNMAPPED".to_string(),
                String::new(),
                "unmapped in-scope token",
            ),
            Some(Target::Duration { start, end }) => {
                let exp_min = expected.trim_end_matches("min").trim().parse::<f64>().ok();
                match (
                    proportional_span_days(&site, ref_jd, start, end, &Optics::default()),
                    exp_min,
                ) {
                    (Some(span_days), Some(em)) => {
                        let got = span_days * 120.0; // (span/12 days) → minutes
                        let ok = (got - em).abs() <= ORACLE_TOLERANCE_SECS as f64 / 60.0;
                        let resid = format!("{:+.0}s", (got - em) * 60.0);
                        (
                            ok,
                            format!("{:.1}min", got),
                            resid,
                            if ok { "" } else { "duration/algorithm" },
                        )
                    }
                    _ => (
                        false,
                        "does-not-occur".to_string(),
                        String::new(),
                        "bound does-not-occur",
                    ),
                }
            }
            Some(Target::Instant(spec)) => {
                let engine = read_instant(&site, ref_jd, spec, &Optics::default());
                if status == "absent" {
                    match engine {
                        None => (true, "does-not-occur".to_string(), String::new(), ""),
                        Some(ai) => (false, render_hhmm(ai, tz), String::new(), "expected absent"),
                    }
                } else {
                    match engine {
                        None => (
                            false,
                            "does-not-occur".to_string(),
                            String::new(),
                            "expected instant (root-find/algorithm)",
                        ),
                        Some(ai) => {
                            let ((hh, mm), off) = parse_expected(expected);
                            let exp_date = base + Duration::days(off);
                            let exp_ts = tz
                                .with_ymd_and_hms(
                                    exp_date.year(),
                                    exp_date.month(),
                                    exp_date.day(),
                                    hh,
                                    mm,
                                    0,
                                )
                                .single()
                                .unwrap()
                                .timestamp();
                            let eng_secs = ai.unix_nanos.div_euclid(1_000_000_000);
                            let resid_s = eng_secs - exp_ts; // + = engine later than oracle
                            let within = resid_s.abs() <= ORACLE_TOLERANCE_SECS;
                            let eng_dt =
                                DateTime::from_timestamp_nanos(ai.unix_nanos).with_timezone(&tz);
                            let day_ok = eng_dt.date_naive() == exp_date;
                            let ok = within && day_ok;
                            let hypo = if ok {
                                ""
                            } else if !day_ok {
                                "tz-rendering / civil-day"
                            } else if def_type == "horizon_crossing" {
                                "dip/refraction magnitude"
                            } else {
                                "algorithm / root-find"
                            };
                            (ok, render_hhmm(ai, tz), format!("{resid_s:+}s"), hypo)
                        }
                    }
                }
            }
        };

        if ok {
            pass += 1;
        } else {
            fail += 1;
        }
        let exp_disp = if expected.is_empty() {
            "(absent)"
        } else {
            expected
        };
        out.push(format!(
            "{:<3} {:<34} {:<18} exp={:<10} eng={:<12} {:<8} {}",
            if ok { "OK" } else { "!!" },
            label,
            status,
            exp_disp,
            engine_str,
            resid,
            hypo
        ));
    }

    eprintln!("\n=== F1 golden-vector per-row results (tolerance ±{ORACLE_TOLERANCE_SECS}s, core-domain/0003) ===");
    for r in &out {
        eprintln!("{r}");
    }
    eprintln!(
        "\nF1 golden vectors: {pass} ok, {fail} fail, {skipped} skipped (calendar_f3 — next pass)"
    );

    assert_eq!(
        fail, 0,
        "{fail} in-scope F1 row(s) missed the oracle; see table above"
    );
}
