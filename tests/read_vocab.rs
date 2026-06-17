//! First-order read-spec vocabulary (ADR core-domain/0020): the netz-definition limb axis and the
//! fixed/seasonal minute-offset read. Anchored to the engine itself (differential / arithmetic
//! identities), no external oracle — these assert *mechanism*, not a shita's correctness (/0019).

use core_engine::events::{read_instant, Bound, Direction, ReadSpec};
use core_engine::optics::{HorizonMode, LimbReference};
use core_engine::params::Optics;
use core_engine::time::jd_from_gregorian;
use core_engine::{AbsoluteInstant, Site};

const JERU: Site = Site {
    lat_deg: 31.778,
    lon_deg: 35.2354,
    elev_m: 754.0,
};

/// Local-noon-anchored UT reference JD for a civil date at Jerusalem (matches `tests/second_order.rs`).
fn ref_jd(y: i32, m: u32, d: f64) -> f64 {
    jd_from_gregorian(y, m, d) - JERU.lon_deg / 360.0
}
fn mins(a: AbsoluteInstant, b: AbsoluteInstant) -> f64 {
    (a.unix_nanos - b.unix_nanos) as f64 / 60.0e9
}
fn netz(optics: &Optics, rjd: f64) -> AbsoluteInstant {
    read_instant(
        &JERU,
        rjd,
        ReadSpec::HorizonCrossing {
            dir: Direction::Rising,
        },
        optics,
    )
    .expect("netz occurs")
}

/// Upper-limb netz (sun's first edge) is EARLIEST, lower-limb (whole disc up) LATEST; center between.
/// The upper↔lower gap is ~2·semidiameter of altitude (~0.53°) ≈ a couple of minutes near the horizon.
#[test]
fn limb_reference_orders_upper_before_center_before_lower() {
    let rjd = ref_jd(2026, 3, 20.5);
    let at = |limb| {
        netz(
            &Optics {
                horizon_mode: HorizonMode::Mishor,
                limb,
                ..Optics::default()
            },
            rjd,
        )
    };
    let upper = at(LimbReference::Upper);
    let center = at(LimbReference::Center);
    let lower = at(LimbReference::Lower);
    assert!(
        upper.unix_nanos < center.unix_nanos && center.unix_nanos < lower.unix_nanos,
        "netz ordering upper < center < lower (first-edge appears before whole disc)"
    );
    let gap = mins(lower, upper); // lower is later → positive
    assert!(
        (1.0..4.0).contains(&gap),
        "upper→lower netz gap ≈ 2·semidiameter near the horizon; Δ={gap:.2} min"
    );
    // Default optics (no explicit limb) must equal Upper — the byte-unchanged regression contract.
    let default = netz(
        &Optics {
            horizon_mode: HorizonMode::Mishor,
            ..Optics::default()
        },
        rjd,
    );
    assert_eq!(default.unix_nanos, upper.unix_nanos);
}

/// Fixed-minute offsets are literal clock minutes off the base bound: alot = netz − 72, tzeit = shkia + 72.
#[test]
fn fixed_minute_offset_is_literal_clock_minutes() {
    let rjd = ref_jd(2026, 3, 20.5);
    let opt = Optics::default();
    let netz_t = netz(&opt, rjd);
    let alot = read_instant(
        &JERU,
        rjd,
        ReadSpec::FixedMinuteOffset {
            base: Bound::Netz,
            offset_min: -72.0,
            seasonal: None,
        },
        &opt,
    )
    .expect("alot occurs");
    assert!(
        (mins(netz_t, alot) - 72.0).abs() < 1.0e-6,
        "fixed alot is exactly netz − 72 clock minutes"
    );

    let shkia = read_instant(
        &JERU,
        rjd,
        ReadSpec::HorizonCrossing {
            dir: Direction::Setting,
        },
        &opt,
    )
    .expect("shkia occurs");
    let tzeit = read_instant(
        &JERU,
        rjd,
        ReadSpec::FixedMinuteOffset {
            base: Bound::Shkia,
            offset_min: 72.0,
            seasonal: None,
        },
        &opt,
    )
    .expect("tzeit occurs");
    assert!(
        (mins(tzeit, shkia) - 72.0).abs() < 1.0e-6,
        "fixed tzeit is exactly shkia + 72 clock minutes"
    );
}

/// Seasonal (zmaniyos) minutes scale with the day length: ≈ fixed at the equinox (sha'ah ≈ 60 min),
/// materially larger in summer (longer day → longer sha'ah → more real minutes per zmaniyos minute).
#[test]
fn seasonal_minute_offset_scales_with_day_length() {
    let alot = |rjd, seasonal| {
        read_instant(
            &JERU,
            rjd,
            ReadSpec::FixedMinuteOffset {
                base: Bound::Netz,
                offset_min: -72.0,
                seasonal,
            },
            &Optics::default(),
        )
        .expect("alot occurs")
    };
    let span = Some((Bound::Netz, Bound::Shkia));

    // Equinox: day ≈ 12 h ⇒ sha'ah ≈ 60 min ⇒ 72 zmaniyos min ≈ 72 clock min.
    let eq = ref_jd(2026, 3, 20.5);
    let near = mins(alot(eq, None), alot(eq, span)); // |fixed − seasonal|, minutes
                                                     // ≈ equal — the small residual is the refraction/dip day being a touch over 12 h (sha'ah ≈ 61½ min).
    assert!(
        near.abs() < 2.5,
        "at the equinox seasonal ≈ fixed; Δ={near:.2} min"
    );

    // Summer solstice: day > 12 h ⇒ sha'ah > 60 ⇒ seasonal-72 is materially earlier than fixed-72.
    let su = ref_jd(2026, 6, 21.5);
    let far = mins(alot(su, None), alot(su, span));
    assert!(
        far > 5.0,
        "in summer the longer sha'ah makes seasonal alot notably earlier than fixed; Δ={far:.2} min"
    );
}

/// A does-not-occur base bound propagates through the offset read (polar day → no netz → None).
#[test]
fn offset_propagates_does_not_occur() {
    let svalbard = Site {
        lat_deg: 78.0,
        lon_deg: 15.0,
        elev_m: 0.0,
    };
    let rjd = jd_from_gregorian(2026, 6, 21.5) - svalbard.lon_deg / 360.0;
    assert!(
        read_instant(
            &svalbard,
            rjd,
            ReadSpec::HorizonCrossing {
                dir: Direction::Rising
            },
            &Optics::default()
        )
        .is_none(),
        "polar-day sanity: netz does not occur"
    );
    assert!(
        read_instant(
            &svalbard,
            rjd,
            ReadSpec::FixedMinuteOffset {
                base: Bound::Netz,
                offset_min: -72.0,
                seasonal: None,
            },
            &Optics::default()
        )
        .is_none(),
        "an offset off a does-not-occur base is itself does-not-occur"
    );
}
