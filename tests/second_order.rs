//! Second-order layering demonstration (ADR core-domain/0019). Proves that named poskim positions are
//! reachable as **parameter use over the UNCHANGED first-order engine** — no core knob, no per-posek
//! code path (ADR-0002). These are the substitutions a second-order (preset/management) layer performs;
//! the engine just supplies the mechanism. If any required a core edit, the ruling would be wrong.

use core_engine::events::{read_instant, Direction, ReadSpec};
use core_engine::optics::HorizonMode;
use core_engine::params::Optics;
use core_engine::time::jd_from_gregorian;
use core_engine::{AbsoluteInstant, Site};

fn netz(site: &Site, optics: &Optics) -> AbsoluteInstant {
    // Jerusalem, equinox; the second-order layer picks the date/site/optics — the engine is unchanged.
    let ref_jd = jd_from_gregorian(2026, 3, 20.5) - site.lon_deg / 360.0;
    read_instant(
        site,
        ref_jd,
        ReadSpec::HorizonCrossing {
            dir: Direction::Rising,
        },
        optics,
    )
    .expect("netz occurs")
}
fn mins(a: AbsoluteInstant, b: AbsoluteInstant) -> f64 {
    (a.unix_nanos - b.unix_nanos) as f64 / 60.0e9
}

/// Sh"A HaRav / Igrot Moshe — "reckon everyone as if at Jerusalem's ~800 m" — is just **passing
/// `elev = 800`** to the existing Visible read. No core change; netz comes out earlier (higher vantage
/// sees the sun sooner), the few-minutes shift the poskim discuss.
#[test]
fn fixed_reference_altitude_is_just_an_elevation_substitution() {
    let jeru = |elev| Site {
        lat_deg: 31.778,
        lon_deg: 35.2354,
        elev_m: elev,
    };
    let sea = netz(
        &jeru(0.0),
        &Optics {
            horizon_mode: HorizonMode::Mishor,
            ..Optics::default()
        },
    );
    let at800 = netz(
        &jeru(800.0),
        &Optics {
            horizon_mode: HorizonMode::Visible,
            ..Optics::default()
        },
    );
    let earlier = mins(sea, at800); // positive ⇒ 800 m netz is earlier
    assert!(
        (1.0..8.0).contains(&earlier),
        "elev=800 (Sh\"A HaRav) yields an earlier netz via the unchanged engine; Δ={earlier:.2} min"
    );
}

/// ChaiTables' "radius of influence" (earliest visible sunrise among a community's high points) is just
/// a **`min` over several engine reads** — a second-order aggregation, not engine logic.
#[test]
fn community_earliest_is_just_min_over_reads() {
    let pts = [
        Site {
            lat_deg: 31.78,
            lon_deg: 35.23,
            elev_m: 720.0,
        },
        Site {
            lat_deg: 31.79,
            lon_deg: 35.22,
            elev_m: 826.0,
        }, // a higher vantage → earliest
        Site {
            lat_deg: 31.77,
            lon_deg: 35.24,
            elev_m: 700.0,
        },
    ];
    let opt = Optics {
        horizon_mode: HorizonMode::Visible,
        ..Optics::default()
    };
    let times: Vec<i64> = pts.iter().map(|s| netz(s, &opt).unix_nanos).collect();
    let community = *times.iter().min().unwrap(); // the second-order "earliest in area"
    let highest = netz(&pts[1], &opt).unix_nanos; // the 826 m point
    assert_eq!(
        community, highest,
        "earliest-in-area == the highest vantage's netz"
    );
}

/// mishor vs (sea-level-at-altitude / visible) is a `horizon_mode` choice — the most basic posek axis,
/// already a parameter (built). Mishor ignores elevation; Visible applies the dip → they differ.
#[test]
fn mishor_vs_visible_is_a_horizon_mode_parameter() {
    let site = Site {
        lat_deg: 31.778,
        lon_deg: 35.2354,
        elev_m: 754.0,
    };
    let mishor = netz(
        &site,
        &Optics {
            horizon_mode: HorizonMode::Mishor,
            ..Optics::default()
        },
    );
    let visible = netz(
        &site,
        &Optics {
            horizon_mode: HorizonMode::Visible,
            ..Optics::default()
        },
    );
    assert!(
        mins(mishor, visible) > 0.5,
        "Visible (elevation dip) is earlier than Mishor (sea-level) — a parameter, not a code path"
    );
}
