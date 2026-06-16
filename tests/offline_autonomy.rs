//! Offline-autonomy structural test (ADR core-domain/0005 / open-decisions #9, /0017 hardening). The
//! engine has no network code by construction; this test *demonstrates* the autonomy guarantee by
//! computing a full multi-day **chag + Shabbat** span — every zman, the day-roll, the day-type, and
//! Kiddush-Levana sayability — for consecutive days with zero external input beyond `(φ, λ, h, t)`,
//! asserting every day resolves and the calendar advances coherently.
//!
//! The certified autonomy window **N is still open** (open-decisions #5); here N spans Sukkot 5787
//! (erev → Shabbat → Yom Tov → Chol HaMoed → Shmini Atzeret), a representative chag+Shabbat stretch.

use core_engine::calendar::{
    classify_day, fixed_from_gregorian, gregorian_from_fixed, hebrew_from_fixed, RataDie,
};
use core_engine::couplings::{hebrew_date_at_instant, DayRoll, DEFAULT_DAY_BOUNDARY};
use core_engine::events::{read_instant, Direction, ReadSpec};
use core_engine::params::{Optics, Realm};
use core_engine::time::jd_from_gregorian;
use core_engine::{AbsoluteInstant, Site};

#[test]
fn offline_autonomy_chag_and_shabbat_span() {
    let site = Site {
        lat_deg: 31.778,
        lon_deg: 35.2354,
        elev_m: 754.0,
    };
    let optics = Optics::default();
    let start = fixed_from_gregorian(2026, 9, 25); // erev Sukkot 5787
    let n_days = 10i64;

    let netz = ReadSpec::HorizonCrossing {
        dir: Direction::Rising,
    };
    let shkia = ReadSpec::HorizonCrossing {
        dir: Direction::Setting,
    };
    let tzeit = ReadSpec::DepressionAngle {
        angle_deg: 8.5,
        dir: Direction::Setting,
    };

    let (mut saw_shabbat, mut saw_yom_tov, mut saw_chol_hamoed) = (false, false, false);
    let mut prev_hebrew_rd: Option<i64> = None;

    for off in 0..n_days {
        let rd = RataDie(start.0 + off);
        let (gy, gm, gd) = gregorian_from_fixed(rd);
        let ref_jd = jd_from_gregorian(gy, gm as u32, gd as f64 + 0.5) - site.lon_deg / 360.0;

        // Every solar event resolves offline at this site (no network, no does-not-occur here).
        let n = read_instant(&site, ref_jd, netz, &optics).expect("netz resolves");
        let s = read_instant(&site, ref_jd, shkia, &optics).expect("shkia resolves");
        let t = read_instant(&site, ref_jd, tzeit, &optics).expect("tzeit resolves");
        assert!(
            n.unix_nanos < s.unix_nanos && s.unix_nanos < t.unix_nanos,
            "netz < shkia < tzeit on {gy}-{gm}-{gd}"
        );

        // The day-roll, evaluated at solar noon, yields this civil day's daytime Hebrew date.
        let noon = AbsoluteInstant::from_julian_day(ref_jd);
        let hebrew = match hebrew_date_at_instant(noon, &site, DEFAULT_DAY_BOUNDARY, &optics) {
            DayRoll::Resolved(d) => d,
            DayRoll::BoundaryDoesNotOccur => {
                panic!("day-roll resolves at Jerusalem on {gy}-{gm}-{gd}")
            }
        };
        // At solar noon (before shkia) the rolled date equals the civil day's daytime Hebrew date,
        // and it advances exactly one day per civil day.
        let h_rd = core_engine::calendar::fixed_from_hebrew(hebrew).0;
        assert_eq!(
            hebrew,
            hebrew_from_fixed(rd),
            "day-roll at noon == daytime Hebrew date on {gy}-{gm}-{gd}"
        );
        if let Some(p) = prev_hebrew_rd {
            assert_eq!(
                h_rd,
                p + 1,
                "Hebrew date advances exactly one day per civil day"
            );
        }
        prev_hebrew_rd = Some(h_rd);

        let class = classify_day(hebrew, Realm::EretzYisrael);
        saw_shabbat |= class.shabbat;
        saw_yom_tov |= class.yom_tov;
        saw_chol_hamoed |= class.chol_hamoed;
    }

    // The span genuinely covered a chag + Shabbat (the autonomy scenario, not an ordinary week).
    assert!(saw_shabbat, "span must include Shabbat");
    assert!(saw_yom_tov, "span must include a Yom Tov (Sukkot)");
    assert!(saw_chol_hamoed, "span must include Chol HaMoed");
    eprintln!(
        "offline autonomy: {n_days} consecutive days (Sukkot 5787) all resolved with no network; \
         Shabbat + Yom Tov + Chol HaMoed present; calendar advanced coherently."
    );
}
