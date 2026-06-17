//! Parameter-vector + horizon-profile decode (ADR core-domain/0011/0018). Round-trips via a
//! test-only minicbor encoder (the shipped reader is decode-only/no-alloc), checks resolution to the
//! engine's typed knobs, the azimuth interpolation + binding check, and that malformed/bad input
//! yields a typed `DecodeError` (never a panic — the /0017 invariant, extended to the reader).

use core_engine::events::{Bound, Direction, ReadSpec};
use core_engine::optics::{HorizonMode, LimbReference, RefractionModel};
use core_engine::params::{Realm, TalUmatarBasis, TekufaMethod};
use core_engine::wire::{
    decode_horizon_profile, decode_parameter_vector, decode_read_spec, BoundWire, DecodeError,
    HorizonProfile, ParameterVector, ReadSpecWire, SCHEMA_VERSION,
};
use core_engine::Site;

fn valid_pv() -> ParameterVector {
    ParameterVector {
        schema_version: SCHEMA_VERSION,
        engine_selection: 0,
        horizon_mode: 2,     // terrain-profile
        refraction_model: 0, // standard-atmospheric
        refraction_coeff_micro: None,
        solar_position_reference: 0, // apparent
        solar_limb_reference: 0,     // upper
        locale_realm: 1,             // diaspora
        tal_umatar_basis: 0,         // tekufa-based
        tekufa_method: 0,            // Shmuel
        adar_anniversary_rule: 0,    // Adar II
        kiddush_levana_start: 0,     // three days
        kiddush_levana_end: 0,       // half month
        rounding_stringency: 2,      // nearest
        rounding_granularity: 1,     // minute
    }
}

#[test]
fn parameter_vector_round_trip_and_resolve() {
    let bytes = minicbor::to_vec(valid_pv()).expect("encode");
    let pv = decode_parameter_vector(&bytes).expect("decode");

    let optics = pv.resolve_optics().expect("resolve optics");
    assert_eq!(optics.horizon_mode, HorizonMode::TerrainProfile);
    assert_eq!(optics.horizon_refraction, RefractionModel::Bennett); // standard-atmospheric (/0013)
    assert_eq!(optics.depression_refraction, RefractionModel::None);
    assert_eq!(pv.realm().unwrap(), Realm::Diaspora);
    assert_eq!(pv.tal_umatar_basis().unwrap(), TalUmatarBasis::TekufaBased);
    assert_eq!(pv.tekufa_method().unwrap(), TekufaMethod::Shmuel);
    pv.check_fixed_behaviour()
        .expect("upper-limb + apparent are the engine defaults");
}

#[test]
fn parameter_vector_rejects_bad_header_and_unimplemented() {
    // Wrong schema / engine → typed errors.
    let mut pv = valid_pv();
    pv.schema_version = 999;
    assert_eq!(
        decode_parameter_vector(&minicbor::to_vec(&pv).unwrap()).unwrap_err(),
        DecodeError::Schema
    );
    let mut pv = valid_pv();
    pv.engine_selection = 7;
    assert_eq!(
        decode_parameter_vector(&minicbor::to_vec(&pv).unwrap()).unwrap_err(),
        DecodeError::Engine
    );
    // Real-but-unimplemented spec options are flagged, not faked (ADR-0018 finding).
    let mut pv = valid_pv();
    pv.refraction_model = 1; // meeus-noaa
    let decoded = decode_parameter_vector(&minicbor::to_vec(&pv).unwrap()).unwrap();
    assert_eq!(
        decoded.resolve_optics().unwrap_err(),
        DecodeError::Unimplemented
    );
    // /0020: the limb reference is now a resolved knob, NOT Unimplemented (it flipped here).
    let mut pv = valid_pv();
    pv.solar_limb_reference = 1; // center-limb
    let decoded = decode_parameter_vector(&minicbor::to_vec(&pv).unwrap()).unwrap();
    assert_eq!(
        decoded.resolve_optics().unwrap().limb,
        LimbReference::Center
    );
    decoded
        .check_fixed_behaviour()
        .expect("limb is no longer a fixed behaviour (/0020)");
    let mut pv = valid_pv();
    pv.solar_limb_reference = 9; // out of range
    let decoded = decode_parameter_vector(&minicbor::to_vec(&pv).unwrap()).unwrap();
    assert_eq!(decoded.resolve_optics().unwrap_err(), DecodeError::Range);
    // solar_position_reference (geometric-everywhere) remains the baked/unimplemented axis.
    let mut pv = valid_pv();
    pv.solar_position_reference = 1; // geometric
    let decoded = decode_parameter_vector(&minicbor::to_vec(&pv).unwrap()).unwrap();
    assert_eq!(
        decoded.check_fixed_behaviour().unwrap_err(),
        DecodeError::Unimplemented
    );
}

#[test]
fn read_spec_round_trip_all_variants() {
    // Construct each wire variant, encode (test-only minicbor encoder), decode, and assert it maps to
    // the expected engine ReadSpec. Values are chosen exact under the fixed-point quantization
    // (18° = 18_000_000 µ°; 1/4 = 0.25; −72 min = −72_000 milli-min) so the f64 compare is exact.
    let cases = [
        (
            ReadSpecWire::DepressionAngle {
                angle_microdeg: 18_000_000,
                dir: 0,
            },
            ReadSpec::DepressionAngle {
                angle_deg: 18.0,
                dir: Direction::Rising,
            },
        ),
        (
            ReadSpecWire::HorizonCrossing { dir: 1 },
            ReadSpec::HorizonCrossing {
                dir: Direction::Setting,
            },
        ),
        (ReadSpecWire::ExtremumMidpoint, ReadSpec::ExtremumMidpoint),
        (
            ReadSpecWire::Proportional {
                num: 1,
                den: 4,
                start: BoundWire::Netz,
                end: BoundWire::Shkia,
            },
            ReadSpec::Proportional {
                fraction: 0.25,
                start: Bound::Netz,
                end: Bound::Shkia,
            },
        ),
        (
            ReadSpecWire::FixedMinuteOffset {
                base: BoundWire::Netz,
                offset_milli_min: -72_000,
                seasonal_start: None,
                seasonal_end: None,
            },
            ReadSpec::FixedMinuteOffset {
                base: Bound::Netz,
                offset_min: -72.0,
                seasonal: None,
            },
        ),
        (
            ReadSpecWire::FixedMinuteOffset {
                base: BoundWire::Shkia,
                offset_milli_min: 72_000,
                seasonal_start: Some(BoundWire::Netz),
                seasonal_end: Some(BoundWire::Shkia),
            },
            ReadSpec::FixedMinuteOffset {
                base: Bound::Shkia,
                offset_min: 72.0,
                seasonal: Some((Bound::Netz, Bound::Shkia)),
            },
        ),
    ];
    for (wire, expected) in cases {
        let bytes = minicbor::to_vec(&wire).expect("encode read-spec");
        assert_eq!(
            decode_read_spec(&bytes).expect("decode read-spec"),
            expected
        );
    }
}

#[test]
fn read_spec_malformed_or_out_of_range_never_panics() {
    for bad in [&b""[..], &[0xFF][..], &[0x00][..], &[0x9F, 0xFF][..]] {
        assert!(decode_read_spec(bad).is_err());
    }
    // Bad direction discriminant → Range.
    let bytes = minicbor::to_vec(ReadSpecWire::HorizonCrossing { dir: 9 }).unwrap();
    assert_eq!(decode_read_spec(&bytes).unwrap_err(), DecodeError::Range);
    // Zero denominator → Range.
    let bytes = minicbor::to_vec(ReadSpecWire::Proportional {
        num: 1,
        den: 0,
        start: BoundWire::Netz,
        end: BoundWire::Shkia,
    })
    .unwrap();
    assert_eq!(decode_read_spec(&bytes).unwrap_err(), DecodeError::Range);
    // Half-specified seasonal span → Range.
    let bytes = minicbor::to_vec(ReadSpecWire::FixedMinuteOffset {
        base: BoundWire::Netz,
        offset_milli_min: 0,
        seasonal_start: Some(BoundWire::Netz),
        seasonal_end: None,
    })
    .unwrap();
    assert_eq!(decode_read_spec(&bytes).unwrap_err(), DecodeError::Range);
}

#[test]
fn parameter_vector_malformed_never_panics() {
    for bad in [&b""[..], &[0xFF][..], &[0xA1, 0x00][..], &[0x9F, 0xFF][..]] {
        // Must return Err, never panic.
        assert!(decode_parameter_vector(bad).is_err());
    }
}

/// Build a horizon profile bound to `site` with 4 evenly-spaced angle samples (milliarcminutes).
fn encode_profile(site: &Site, mam: [i32; 4]) -> Vec<u8> {
    let mut angles = Vec::new();
    for v in mam {
        angles.extend_from_slice(&v.to_le_bytes());
    }
    let hp = HorizonProfile {
        lat_microdeg: (site.lat_deg * 1.0e6) as i32,
        lon_microdeg: (site.lon_deg * 1.0e6) as i32,
        elev_mm: (site.elev_m * 1.0e3) as i32,
        dem_source: 1,
        dem_version: 2026,
        prov_refraction_model: 0,
        prov_refraction_coeff_micro: None,
        angles_mam: &angles,
    };
    minicbor::to_vec(&hp).expect("encode profile")
}

#[test]
fn horizon_profile_round_trip_interp_and_binding() {
    let site = Site {
        lat_deg: 31.778,
        lon_deg: 35.2354,
        elev_m: 754.0,
    };
    // Samples at azimuth 0,90,180,270 → 0, 0.1°, 0.2°, 0° (in milliarcminutes; 6000 mam = 0.1°).
    let bytes = encode_profile(&site, [0, 6000, 12000, 0]);
    let hp = decode_horizon_profile(&bytes).expect("decode profile");

    assert_eq!(hp.sample_count(), 4);
    let approx = |a: f64, b: f64| (a - b).abs() < 1e-9;
    assert!(approx(hp.horizon_angle_deg_at(0.0), 0.0));
    assert!(approx(hp.horizon_angle_deg_at(90.0), 0.1));
    assert!(approx(hp.horizon_angle_deg_at(180.0), 0.2));
    assert!(approx(hp.horizon_angle_deg_at(45.0), 0.05)); // interpolation
    assert!(approx(hp.horizon_angle_deg_at(360.0), 0.0)); // wraparound == 0°

    hp.check_binding(&site).expect("profile bound to this site");
    let elsewhere = Site {
        lat_deg: 40.0,
        lon_deg: -74.0,
        elev_m: 10.0,
    };
    assert_eq!(
        hp.check_binding(&elsewhere).unwrap_err(),
        DecodeError::Binding
    );
}

#[test]
fn horizon_profile_malformed_never_panics() {
    assert!(decode_horizon_profile(&[0xFF]).is_err());
    // A profile whose angle array isn't 4-byte aligned must be rejected, not panic on lookup.
    let site = Site {
        lat_deg: 0.0,
        lon_deg: 0.0,
        elev_m: 0.0,
    };
    let mut bytes = encode_profile(&site, [0, 0, 0, 0]);
    bytes.pop(); // corrupt length
    assert!(decode_horizon_profile(&bytes).is_err());
}
