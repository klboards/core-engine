//! TerrainProfile activation (ADR core-domain/0004 + /0018, the visible-sunrise moat). Differential
//! properties — no external oracle needed: a flat-zero profile reproduces the sea-level (Mishor)
//! horizon; a higher horizon angle delays sunrise / advances sunset; and an angle placed only at the
//! sunrise azimuth shifts netz but NOT shkia — proving the crossing target is genuinely
//! azimuth-dependent (the whole point of a terrain skyline).

use core_engine::events::{
    read_instant, read_instant_with_horizon, terrain_horizon_crossing, Bound, Direction, ReadSpec,
};
use core_engine::optics::HorizonMode;
use core_engine::params::Optics;
use core_engine::time::jd_from_gregorian;
use core_engine::wire::HorizonProfile;
use core_engine::{AbsoluteInstant, Site};

const MAM_HALF_DEG: i32 = 30_000; // 0.5° in milliarcminutes

fn pack(mam: &[i32]) -> Vec<u8> {
    let mut v = Vec::with_capacity(mam.len() * 4);
    for &x in mam {
        v.extend_from_slice(&x.to_le_bytes());
    }
    v
}

fn mk(angles: &[u8]) -> HorizonProfile<'_> {
    HorizonProfile {
        lat_microdeg: 32_000_000,
        lon_microdeg: 35_000_000,
        elev_mm: 0,
        dem_source: 1,
        dem_version: 1,
        prov_refraction_model: 0,
        prov_refraction_coeff_micro: None,
        angles_mam: angles,
    }
}

fn secs(a: AbsoluteInstant, b: AbsoluteInstant) -> f64 {
    (a.unix_nanos - b.unix_nanos) as f64 / 1.0e9
}

#[test]
fn terrain_profile_activation() {
    let site = Site {
        lat_deg: 32.0,
        lon_deg: 35.0,
        elev_m: 0.0,
    };
    let optics = Optics::default();
    let mishor = Optics {
        horizon_mode: HorizonMode::Mishor,
        ..Optics::default()
    };
    // Equinox: the Sun rises ~due east (az ≈ 90) and sets ~due west (az ≈ 270).
    let ref_jd = jd_from_gregorian(2026, 3, 20.5) - site.lon_deg / 360.0;
    let rise = |hp: &HorizonProfile| {
        terrain_horizon_crossing(&site, ref_jd, Direction::Rising, &optics, hp).unwrap()
    };
    let set = |hp: &HorizonProfile| {
        terrain_horizon_crossing(&site, ref_jd, Direction::Setting, &optics, hp).unwrap()
    };

    let zero = pack(&[0i32; 360]);
    let hp0 = mk(&zero);

    // (1) A flat-zero skyline reproduces the sea-level (Mishor) horizon crossing.
    let netz_mishor = read_instant(
        &site,
        ref_jd,
        ReadSpec::HorizonCrossing {
            dir: Direction::Rising,
        },
        &mishor,
    )
    .unwrap();
    assert!(
        secs(rise(&hp0), netz_mishor).abs() < 2.0,
        "flat-zero terrain ≈ Mishor netz (Δ {:.2}s)",
        secs(rise(&hp0), netz_mishor)
    );

    // (2) A uniform +0.5° skyline delays sunrise and advances sunset (the Sun must climb higher).
    let const_half = pack(&[MAM_HALF_DEG; 360]);
    let hpc = mk(&const_half);
    assert!(
        secs(rise(&hpc), rise(&hp0)) > 60.0,
        "uniform +0.5° → later sunrise"
    );
    assert!(
        secs(set(&hp0), set(&hpc)) > 60.0,
        "uniform +0.5° → earlier sunset"
    );

    // (3) +0.5° ONLY in the east sector (az 45–135) shifts netz but leaves shkia ≈ flat — the target
    // is azimuth-dependent (the moat). A uniform profile could not produce this asymmetry.
    let mut east = [0i32; 360];
    for (az, a) in east.iter_mut().enumerate() {
        if (45..=135).contains(&az) {
            *a = MAM_HALF_DEG;
        }
    }
    let east_bytes = pack(&east);
    let hpe = mk(&east_bytes);
    assert!(
        secs(rise(&hpe), rise(&hp0)) > 60.0,
        "east skyline delays sunrise"
    );
    assert!(
        secs(set(&hpe), set(&hp0)).abs() < 2.0,
        "east skyline leaves sunset ≈ flat (Δ {:.2}s) — azimuth-dependent",
        secs(set(&hpe), set(&hp0))
    );
}

/// The moat reaching the deadlines: a `TerrainProfile` GRA proportional zman (sof-zman-shma = ¼ of the
/// netz→shkia day) is reckoned from the SAME terrain netz/shkia as the displayed event — not from the
/// sea-level bounds. Proves the fix (`read_instant_with_horizon` threading the profile into the
/// proportional path) and that `None` stays the byte-frozen scalar fallback.
#[test]
fn terrain_proportional_uses_terrain_bounds() {
    let site = Site {
        lat_deg: 32.0,
        lon_deg: 35.0,
        elev_m: 0.0,
    };
    let terrain = Optics {
        horizon_mode: HorizonMode::TerrainProfile,
        ..Optics::default()
    };
    let mishor = Optics {
        horizon_mode: HorizonMode::Mishor,
        ..Optics::default()
    };
    let ref_jd = jd_from_gregorian(2026, 3, 20.5) - site.lon_deg / 360.0;

    // A +0.5° east-sector skyline → a later visible netz (a "Har HaZeisim" in miniature).
    let mut east = [0i32; 360];
    for (az, a) in east.iter_mut().enumerate() {
        if (45..=135).contains(&az) {
            *a = MAM_HALF_DEG;
        }
    }
    let east_bytes = pack(&east);
    let hp = mk(&east_bytes);

    // GRA sof-zman-shma = netz + ¼·(shkia − netz).
    let gra_shma = ReadSpec::Proportional {
        fraction: 0.25,
        start: Bound::Netz,
        end: Bound::Shkia,
    };
    let t_netz = terrain_horizon_crossing(&site, ref_jd, Direction::Rising, &terrain, &hp).unwrap();
    let t_shkia =
        terrain_horizon_crossing(&site, ref_jd, Direction::Setting, &terrain, &hp).unwrap();
    let shma_terrain =
        read_instant_with_horizon(&site, ref_jd, gra_shma, &terrain, Some(&hp)).unwrap();

    // (1) CONSISTENCY — the proportional day uses the terrain netz/shkia (not mishor).
    let from_netz = secs(shma_terrain, t_netz);
    let quarter_day = 0.25 * secs(t_shkia, t_netz);
    assert!(
        (from_netz - quarter_day).abs() < 2.0,
        "GRA shma sits ¼ into the TERRAIN day (Δ {:.2}s)",
        from_netz - quarter_day
    );

    // (2) The fix actually changed the deadline: terrain shma ≠ mishor shma (the moat reaches it).
    let shma_mishor = read_instant(&site, ref_jd, gra_shma, &mishor).unwrap();
    assert!(
        secs(shma_terrain, shma_mishor).abs() > 30.0,
        "terrain GRA shma differs from the sea-level one (Δ {:.2}s)",
        secs(shma_terrain, shma_mishor)
    );

    // (3) Back-compat: a None horizon is the byte-frozen scalar path even in TerrainProfile mode.
    let shma_none = read_instant_with_horizon(&site, ref_jd, gra_shma, &terrain, None).unwrap();
    let shma_scalar = read_instant(&site, ref_jd, gra_shma, &terrain).unwrap();
    assert_eq!(
        shma_none.unix_nanos, shma_scalar.unix_nanos,
        "None horizon == scalar read (no behavioural drift)"
    );
}
