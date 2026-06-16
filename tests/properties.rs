//! Property / invariant tests (ADR core-domain/0017 hardening). Where the oracle vectors prove
//! correctness *at sampled points*, these prove **invariants that must hold across the input domain** —
//! catching whole bug classes example tests miss. Dependency-free, seeded (SplitMix64) → fully
//! reproducible, no flakiness. Pure-integer properties run 20k iterations; float-path ones 2k.

use core_engine::calendar::{
    classify_day, fixed_from_gregorian, fixed_from_hebrew, gregorian_from_fixed, hebrew_from_fixed,
    last_day_of_month, last_month_of_year, molad_chalakim, weekday_from_fixed, HebrewDate, RataDie,
    CHALAKIM_PER_MONTH,
};
use core_engine::couplings::{hebrew_date_at_instant, DayRoll, DEFAULT_DAY_BOUNDARY};
use core_engine::events::{read_jd, Direction, ReadSpec};
use core_engine::kiddush_levana::kiddush_levana_window;
use core_engine::params::{KiddushLevanaEnd, KiddushLevanaStart, Optics, Realm};
use core_engine::tekufa::{tekufa_chalakim, Season};
use core_engine::time::jd_from_gregorian;
use core_engine::{AbsoluteInstant, Site};

/// Dependency-free deterministic PRNG (SplitMix64) — reproducible across runs and platforms.
struct Rng(u64);
impl Rng {
    fn new(seed: u64) -> Self {
        Rng(seed)
    }
    fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }
    /// Uniform f64 in [lo, hi).
    fn f64_in(&mut self, lo: f64, hi: f64) -> f64 {
        let u = (self.next_u64() >> 11) as f64 / (1u64 << 53) as f64;
        lo + u * (hi - lo)
    }
    /// Uniform i64 in [lo, hi] inclusive.
    fn i64_in(&mut self, lo: i64, hi: i64) -> i64 {
        lo + (self.next_u64() % ((hi - lo + 1) as u64)) as i64
    }
}

const NS_PER_DAY: i64 = 86_400_000_000_000;

#[test]
fn prop_gregorian_round_trip() {
    let mut rng = Rng::new(0x6772_6567); // "greg"
    for _ in 0..20_000 {
        let rd = rng.i64_in(1, 1_460_000); // ≈ Gregorian years 1..4000
        let (y, m, d) = gregorian_from_fixed(RataDie(rd));
        // Components are in valid civil ranges, and the inverse recovers the RD exactly.
        assert!(
            (1..=12).contains(&m) && (1..=31).contains(&d),
            "rd {rd} → {y}-{m}-{d}"
        );
        assert_eq!(
            fixed_from_gregorian(y, m, d),
            RataDie(rd),
            "greg round-trip rd {rd}"
        );
    }
}

#[test]
fn prop_hebrew_round_trip_and_validity() {
    let mut rng = Rng::new(0x0002); // hebrew round-trip
    for _ in 0..20_000 {
        let rd = rng.i64_in(100_000, 1_400_000); // Hebrew years ≈ 3761..7760
        let h = hebrew_from_fixed(RataDie(rd));
        // Validity: month and day within the structural bounds of that Hebrew year.
        assert!(
            h.month >= 1 && h.month <= last_month_of_year(h.year),
            "rd {rd} → {:?} month out of range",
            h
        );
        assert!(
            h.day >= 1 && h.day <= last_day_of_month(h.year, h.month),
            "rd {rd} → {:?} day out of range",
            h
        );
        // Round-trip identity.
        assert_eq!(
            fixed_from_hebrew(h),
            RataDie(rd),
            "hebrew round-trip rd {rd}"
        );
    }
}

#[test]
fn prop_molad_interval_exactly_synodic() {
    let mut rng = Rng::new(0x0003); // molad interval
    for _ in 0..20_000 {
        let y = rng.i64_in(1, 9000) as i32;
        // Contiguous months Tishrei (7) → Cheshvan (8) within a year differ by exactly one synodic.
        assert_eq!(
            molad_chalakim(y, 8) - molad_chalakim(y, 7),
            CHALAKIM_PER_MONTH,
            "molad interval year {y}"
        );
        // Across the year-number roll: Adar(II)→next Nisan is also exactly one synodic.
        let last = last_month_of_year(y);
        assert_eq!(
            molad_chalakim(y, 1) - molad_chalakim(y, last),
            CHALAKIM_PER_MONTH,
            "molad interval across last→Nisan, year {y}"
        );
    }
}

#[test]
fn prop_tekufa_spacing_exact_shmuel() {
    use core_engine::params::TekufaMethod::Shmuel;
    const SEASON_CH: i64 = 9_467_280 / 4; // Shmuel year (chalakim) / 4
    let mut rng = Rng::new(0x74656b75); // "teku"
    for _ in 0..20_000 {
        let y = rng.i64_in(1, 9000) as i32;
        assert_eq!(
            tekufa_chalakim(y, Season::Tammuz, Shmuel) - tekufa_chalakim(y, Season::Nisan, Shmuel),
            SEASON_CH,
            "Nisan→Tammuz spacing, year {y}"
        );
        assert_eq!(
            tekufa_chalakim(y, Season::Tevet, Shmuel) - tekufa_chalakim(y, Season::Tishrei, Shmuel),
            SEASON_CH,
            "Tishrei→Tevet spacing, year {y}"
        );
    }
}

#[test]
fn prop_weekday_consistent() {
    let mut rng = Rng::new(0x776b); // "wk"
    for _ in 0..20_000 {
        let rd = rng.i64_in(-500_000, 1_500_000);
        let w = weekday_from_fixed(RataDie(rd));
        assert!(w <= 6, "weekday out of range at rd {rd}");
        assert_eq!(
            weekday_from_fixed(RataDie(rd + 1)),
            (w + 1) % 7,
            "weekday advances by 1, rd {rd}"
        );
    }
}

#[test]
fn prop_classify_day_invariants() {
    let mut rng = Rng::new(0x636c61737379); // "classy"
    for _ in 0..20_000 {
        let y = rng.i64_in(5700, 5900) as i32;
        let m = rng.i64_in(1, last_month_of_year(y) as i64) as u8;
        let d = rng.i64_in(1, last_day_of_month(y, m) as i64) as u8;
        let date = HebrewDate {
            year: y,
            month: m,
            day: d,
        };
        for realm in [Realm::EretzYisrael, Realm::Diaspora] {
            let c = classify_day(date, realm); // must not panic
            let rd = fixed_from_hebrew(date);
            assert_eq!(
                c.shabbat,
                weekday_from_fixed(rd) == 6,
                "shabbat⟺Saturday {date:?}"
            );
            assert_eq!(
                c.rosh_chodesh,
                d == 30 || (d == 1 && m != 7),
                "rosh_chodesh rule {date:?}"
            );
            if c.chol_hamoed {
                assert!(m == 1 || m == 7, "chol_hamoed only Nisan/Tishrei {date:?}");
            }
        }
    }
}

#[test]
fn prop_kiddush_levana_window_well_formed() {
    let mut rng = Rng::new(0x6b6c); // "kl"
    for _ in 0..20_000 {
        let y = rng.i64_in(5700, 5900) as i32;
        let m = rng.i64_in(1, last_month_of_year(y) as i64) as u8;
        let (open, close) = kiddush_levana_window(
            y,
            m,
            KiddushLevanaStart::ThreeDays,
            KiddushLevanaEnd::HalfMonth,
        );
        assert!(
            open.unix_nanos < close.unix_nanos,
            "KL window non-empty {y}/{m}"
        );
        let span = close.unix_nanos - open.unix_nanos;
        // molad+3d .. molad+14d18h22m → span ≈ 11d18h; bound generously.
        assert!(
            span > 10 * NS_PER_DAY && span < 13 * NS_PER_DAY,
            "KL span sane {y}/{m}: {span} ns"
        );
    }
}

/// Regression for the high-elevation horizon-crossing bug found by `prop_zman_ordering_non_polar`
/// (ADR core-domain/0017): at 1868 m the dip target (~−1.66°) sits below the horizon, where Bennett
/// refraction is non-monotonic and used to fake an ascending crossing during the evening descent —
/// returning an evening "sunrise" ~0.71 d before local noon. The geometric-slope gate fixes it; netz
/// must now land in the morning (~0.29 d before local noon), ascending.
#[test]
fn regression_high_elevation_netz_is_morning() {
    let site = Site {
        lat_deg: 22.303_273_750_725_31,
        lon_deg: 98.505_209_917_186_28,
        elev_m: 1868.3894126853472,
    };
    let optics = Optics::default();
    let ref_jd = jd_from_gregorian(2017, 6, 15.5) - site.lon_deg / 360.0;
    let netz = read_jd(
        &site,
        ref_jd,
        ReadSpec::HorizonCrossing {
            dir: Direction::Rising,
        },
        &optics,
    )
    .expect("sunrise occurs");
    let before_noon = ref_jd - netz;
    assert!(
        (0.20..0.40).contains(&before_noon),
        "netz must be a morning crossing (~0.29 d before local noon), got {before_noon:.4} d \
         (the pre-fix bug returned ~0.71 d — an evening setting limb)"
    );
}

#[test]
fn prop_zman_ordering_non_polar() {
    let optics = Optics::default();
    let mut rng = Rng::new(0x7a6d616e); // "zman"
    let alot = ReadSpec::DepressionAngle {
        angle_deg: 16.1,
        dir: Direction::Rising,
    };
    let netz = ReadSpec::HorizonCrossing {
        dir: Direction::Rising,
    };
    let chatzot = ReadSpec::ExtremumMidpoint;
    let shkia = ReadSpec::HorizonCrossing {
        dir: Direction::Setting,
    };
    let tzeit = ReadSpec::DepressionAngle {
        angle_deg: 8.5,
        dir: Direction::Setting,
    };
    for _ in 0..2_000 {
        // |lat| < 45: all of alot..tzeit occur year-round, so ordering is total.
        let site = Site {
            lat_deg: rng.f64_in(-45.0, 45.0),
            lon_deg: rng.f64_in(-180.0, 180.0),
            elev_m: rng.f64_in(0.0, 2000.0),
        };
        let y = rng.i64_in(2000, 2060) as i32;
        let m = rng.i64_in(1, 12) as u8;
        let d = rng.i64_in(1, 28) as u8;
        let ref_jd = jd_from_gregorian(y, m as u32, d as f64 + 0.5) - site.lon_deg / 360.0;
        let r = |s| read_jd(&site, ref_jd, s, &optics);
        if let (Some(a), Some(n), Some(c), Some(s), Some(t)) =
            (r(alot), r(netz), r(chatzot), r(shkia), r(tzeit))
        {
            assert!(
                a < n && n < c && c < s && s < t,
                "zman order violated at {site:?} {y}-{m}-{d}: alot {a} netz {n} chatzot {c} shkia {s} tzeit {t}"
            );
        }
    }
}

#[test]
fn prop_day_roll_advances_one_per_day() {
    let optics = Optics::default();
    let mut rng = Rng::new(0x726f6c6c); // "roll"
    for _ in 0..2_000 {
        // Non-polar: exactly one boundary per solar day, so the Hebrew RD advances by ≈ N over N days
        // (±1 absorbs the sunset-drift margin near a boundary). Also strictly monotone in time.
        let site = Site {
            lat_deg: rng.f64_in(-55.0, 55.0),
            lon_deg: rng.f64_in(-180.0, 180.0),
            elev_m: 0.0,
        };
        // A base instant somewhere in 2020..2050 (unix seconds).
        let base_secs = rng.i64_in(1_577_836_800, 2_524_608_000);
        let t0 = AbsoluteInstant {
            unix_nanos: base_secs * 1_000_000_000,
        };
        let n = rng.i64_in(1, 60);
        let t1 = AbsoluteInstant {
            unix_nanos: t0.unix_nanos + n * NS_PER_DAY,
        };
        if let (DayRoll::Resolved(d0), DayRoll::Resolved(d1)) = (
            hebrew_date_at_instant(t0, &site, DEFAULT_DAY_BOUNDARY, &optics),
            hebrew_date_at_instant(t1, &site, DEFAULT_DAY_BOUNDARY, &optics),
        ) {
            let diff = fixed_from_hebrew(d1).0 - fixed_from_hebrew(d0).0;
            assert!(diff >= 0, "Hebrew date never goes backward ({diff})");
            assert!(
                (diff - n).abs() <= 1,
                "≈ one Hebrew day per solar day: n={n} diff={diff}"
            );
        }
    }
}
