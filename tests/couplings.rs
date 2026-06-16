//! Couplings #1 (day-roll) and #3 (full Kiddush Levana), ADR core-domain/0016. These are validated by
//! their **defining differential properties** (no external oracle value needed for correctness): the
//! day rolls exactly at the boundary instant; Kiddush Levana reduces to moon-up once window∩night
//! hold. One day-roll anchor is cross-checked against MyZmanim (Jerusalem 2026-12-04 = 24 Kislev, the
//! roll into 25 Kislev at shkia). FP-determinism of the float paths is covered by `ffi` kinds 13/14.

use core_engine::calendar::{
    fixed_from_gregorian, gregorian_from_fixed, hebrew_from_fixed, molad_instant, HebrewDate,
    RataDie,
};
use core_engine::couplings::{
    hebrew_date_at_instant, kiddush_levana_interval_on_night, kiddush_levana_sayable_at, DayRoll,
    DEFAULT_DAY_BOUNDARY,
};
use core_engine::events::{read_instant, Direction, ReadSpec};
use core_engine::kiddush_levana::moon_visible;
use core_engine::params::{KiddushLevanaEnd, KiddushLevanaStart, Optics};
use core_engine::time::{jd_from_gregorian, jd_from_unix_secs};
use core_engine::{AbsoluteInstant, Site};

const NS_PER_MIN: i64 = 60_000_000_000;
const NS_PER_DAY: i64 = 86_400_000_000_000;
const JD_AT_RD0: f64 = 1_721_424.5;

fn jerusalem() -> Site {
    Site {
        lat_deg: 31.778,
        lon_deg: 35.2354,
        elev_m: 754.0,
    }
}

/// Local-noon UT anchor for a civil Gregorian date (mirrors the private helper in `couplings`).
fn local_noon_jd(y: i32, m: u8, d: u8, lon_deg: f64) -> f64 {
    jd_from_gregorian(y, m as u32, d as f64 + 0.5) - lon_deg / 360.0
}

#[test]
fn day_roll_at_shkia() {
    let site = jerusalem();
    let optics = Optics::default();
    // Jerusalem civil 2026-12-04 (= 24 Kislev 5787 by day, MyZmanim). The Hebrew day rolls to 25
    // Kislev at shkia; just before → 24 Kislev, just after → 25 Kislev.
    let ref_jd = local_noon_jd(2026, 12, 4, site.lon_deg);
    let shkia = read_instant(
        &site,
        ref_jd,
        ReadSpec::HorizonCrossing {
            dir: Direction::Setting,
        },
        &optics,
    )
    .expect("Jerusalem has a sunset");

    let rd = fixed_from_gregorian(2026, 12, 4);
    let before = AbsoluteInstant {
        unix_nanos: shkia.unix_nanos - NS_PER_MIN,
    };
    let after = AbsoluteInstant {
        unix_nanos: shkia.unix_nanos + NS_PER_MIN,
    };
    assert_eq!(
        hebrew_date_at_instant(before, &site, DEFAULT_DAY_BOUNDARY, &optics),
        DayRoll::Resolved(hebrew_from_fixed(rd)),
        "before shkia → daytime Hebrew date (24 Kislev)"
    );
    assert_eq!(
        hebrew_date_at_instant(after, &site, DEFAULT_DAY_BOUNDARY, &optics),
        DayRoll::Resolved(hebrew_from_fixed(RataDie(rd.0 + 1))),
        "after shkia → rolled Hebrew date (25 Kislev)"
    );
    // Anchor: 24 Kislev = 5787/9/24, 25 Kislev = 5787/9/25 (MyZmanim).
    assert_eq!(
        hebrew_from_fixed(rd),
        HebrewDate {
            year: 5787,
            month: 9,
            day: 24
        }
    );
}

#[test]
fn day_roll_polar_does_not_occur() {
    let optics = Optics::default();
    // 80°N at midsummer: the Sun never sets → the boundary does-not-occur → the roll is undefined,
    // surfaced (never guessed). The high-latitude fallback policy is an open gate (ADR-0016).
    let polar = Site {
        lat_deg: 80.0,
        lon_deg: 0.0,
        elev_m: 0.0,
    };
    let t = AbsoluteInstant::from_julian_day(jd_from_gregorian(2026, 6, 21.5));
    assert_eq!(
        hebrew_date_at_instant(t, &polar, DEFAULT_DAY_BOUNDARY, &optics),
        DayRoll::BoundaryDoesNotOccur
    );
}

#[test]
fn kiddush_levana_clauses() {
    let site = jerusalem();
    let optics = Optics::default();
    let (year, month) = (5786, 8); // Cheshvan 5786
    let night_dep = 8.5_f64;
    let (start, end) = (KiddushLevanaStart::ThreeDays, KiddushLevanaEnd::HalfMonth);
    let say = |t: AbsoluteInstant| {
        kiddush_levana_sayable_at(t, year, month, &site, night_dep, start, end, &optics)
    };

    let (open, close) = core_engine::kiddush_levana::kiddush_levana_window(year, month, start, end);

    // (a) Before the window opens and after it closes → never sayable.
    assert!(!say(AbsoluteInstant {
        unix_nanos: open.unix_nanos - NS_PER_DAY
    }));
    assert!(!say(AbsoluteInstant {
        unix_nanos: close.unix_nanos + NS_PER_DAY
    }));

    // Build a local-noon and a late-evening instant ~5 days after the molad (inside the window).
    let molad_jd = jd_from_unix_secs(molad_instant(year, month).unix_nanos as f64 / 1.0e9);
    let day5_jd = molad_jd + 5.0;
    let rd = (day5_jd - JD_AT_RD0).floor() as i64;
    let (gy, gm, gd) = gregorian_from_fixed(RataDie(rd));
    let noon_jd = local_noon_jd(gy, gm, gd, site.lon_deg);
    let noon = AbsoluteInstant::from_julian_day(noon_jd);
    let night = AbsoluteInstant::from_julian_day(noon_jd + 0.4); // ≈ 21:36 local — well after shkia

    // (b) Daytime inside the window → not sayable (night clause fails), regardless of the moon.
    assert!(
        !say(noon),
        "local noon inside the window must not be sayable (it is day)"
    );

    // (c) At a night instant inside the window, sayable reduces exactly to moon-up (the conjunction).
    let moon_up = moon_visible(noon_jd + 0.4, &site, &optics);
    assert!(
        open.unix_nanos <= night.unix_nanos && night.unix_nanos <= close.unix_nanos,
        "night instant must be inside the window (precondition)"
    );
    assert_eq!(
        say(night),
        moon_up,
        "window∩night hold → sayable == moon-up"
    );

    // (d) interval_on_night: Some on a night inside the window, None on a night before the molad.
    assert!(
        kiddush_levana_interval_on_night(
            noon_jd, year, month, &site, night_dep, start, end, &optics
        )
        .is_some(),
        "a night ~5 days after molad intersects the window"
    );
    assert!(
        kiddush_levana_interval_on_night(
            noon_jd - 10.0,
            year,
            month,
            &site,
            night_dep,
            start,
            end,
            &optics
        )
        .is_none(),
        "a night ~5 days before molad is outside the window"
    );
}
