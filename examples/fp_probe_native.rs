//! Native side of the FP-determinism check (ADR core-domain/0010). Prints, for a fixed set of
//! representative F1 reads, the inputs as exact f64 bit-patterns + the native `i64`-nanosecond
//! result. The node harness (`tools/fp_probe.mjs`) reconstructs the identical f64 inputs, runs
//! the SAME reads through the wasm build, and asserts EXACT cross-target equality.
//!
//! Inputs are bit-exact (hex of `f64::to_bits`) so the comparison cannot be polluted by decimal
//! round-trip. Reads include all four F1 read-types, does-not-occur, and a near-grazing
//! high-latitude case (the ~30 ns regime the ±1-min golden vectors structurally cannot see).

use core_engine::ffi::probe_zman_nanos;

fn main() {
    // (kind, lat, lon, elev_m, ref_jd_ut, angle_deg, label)
    let rows: &[(u32, f64, f64, f64, f64, f64, &str)] = &[
        (
            2,
            31.778,
            35.235,
            754.0,
            2_460_755.0,
            0.0,
            "jeru_netz_equinox",
        ),
        (
            3,
            31.778,
            35.235,
            754.0,
            2_460_755.0,
            0.0,
            "jeru_shkia_equinox",
        ),
        (
            0,
            31.778,
            35.235,
            754.0,
            2_460_755.0,
            16.1,
            "jeru_alot_16.1",
        ),
        (1, 31.778, 35.235, 754.0, 2_460_755.0, 8.5, "jeru_tzeit_8.5"),
        (4, 40.7128, -74.006, 10.0, 2_460_755.0, 0.0, "nyc_chatzot"),
        (2, 0.0, 0.0, 0.0, 2_460_755.0, 0.0, "equator_netz"),
        (
            5,
            31.778,
            35.235,
            754.0,
            2_460_755.0,
            0.0,
            "jeru_sofzman_shma_gra",
        ),
        (
            6,
            31.778,
            35.235,
            754.0,
            2_460_755.0,
            0.0,
            "jeru_sofzman_shma_mga",
        ),
        (
            0,
            51.5,
            -0.12,
            0.0,
            2_460_848.0,
            16.1,
            "london_june_alot_16.1_dno",
        ),
        // True near-grazing: lat 60, June, min night altitude ≈ −6.6°, so a setting crossing of
        // −6.5° barely exists — the ~30 ns sub-ULP regime the ±1-min vectors cannot see.
        (
            1,
            60.0,
            10.0,
            0.0,
            2_460_848.0,
            6.5,
            "lat60_june_tzeit_6.5_grazing",
        ),
        // F2 (lunar): moonrise/moonset (ref_jd = local-midnight day-start) + an apparent altitude;
        // F3 molad instant (ref_jd = year, angle = month). These exercise the ELP series, the
        // topocentric parallax, the moon crossing, and the molad UT projection.
        (
            7,
            31.778,
            35.2354,
            754.0,
            2_460_753.416_667,
            0.0,
            "jeru_moonrise",
        ),
        (
            8,
            31.778,
            35.2354,
            754.0,
            2_460_753.416_667,
            0.0,
            "jeru_moonset",
        ),
        (
            9,
            31.778,
            35.2354,
            754.0,
            2_460_755.208_333,
            0.0,
            "jeru_moon_alt",
        ),
        (10, 0.0, 0.0, 0.0, 5786.0, 7.0, "molad_tishrei_5786"),
        (10, 0.0, 0.0, 0.0, 5787.0, 7.0, "molad_tishrei_5787"),
        // Phase-3 couplings (ADR core-domain/0016): the tekufa UT projection (Shmuel + Rav Ada, both
        // through the shared molad projection at different magnitudes), the day-roll float path
        // (jd→RD floor + local-noon boundary + the cross-boundary comparison — incl. a far-longitude
        // near-00:00-UT case for the off-by-one risk), and the night-predicate sun altitude.
        (11, 0.0, 0.0, 0.0, 5787.0, 2.0, "tekufa_tishrei_5787_shmuel"),
        (12, 0.0, 0.0, 0.0, 5787.0, 2.0, "tekufa_tishrei_5787_ravada"),
        (13, 31.778, 35.2354, 754.0, 2_460_756.2, 0.0, "jeru_dayroll"),
        (
            13,
            1.35,
            172.98,
            0.0,
            2_460_756.49,
            0.0,
            "kiribati_dayroll_near_utc_midnight",
        ),
        (
            14,
            31.778,
            35.2354,
            754.0,
            2_460_755.208_333,
            0.0,
            "jeru_sun_alt",
        ),
    ];
    let emit = |kind: u32, lat: f64, lon: f64, elev: f64, ref_jd: f64, angle: f64, label: &str| {
        let nanos = probe_zman_nanos(kind, lat, lon, elev, ref_jd, angle);
        println!(
            "{},{:016x},{:016x},{:016x},{:016x},{:016x},{},{}",
            kind,
            lat.to_bits(),
            lon.to_bits(),
            elev.to_bits(),
            ref_jd.to_bits(),
            angle.to_bits(),
            nanos,
            label
        );
    };
    for &(kind, lat, lon, elev, ref_jd, angle, label) in rows {
        emit(kind, lat, lon, elev, ref_jd, angle, label);
    }

    // ── Determinism breadth grid (ADR core-domain/0017): sweep the float paths across latitudes,
    // longitudes, elevations, hemispheres and the full year so native==wasm is verified across the
    // input domain, not just the curated edge points. (Values aren't oracle-checked here — this gate
    // is purely cross-target bit-equality; correctness is the oracle suites.)
    let sites: &[(f64, f64, f64, &str)] = &[
        (31.778, 35.2354, 754.0, "jerusalem"),
        (40.7128, -74.006, 10.0, "nyc"),
        (0.0, 0.0, 0.0, "equator"),
        (60.0, 10.0, 0.0, "oslo60"),
        (22.30327, 98.50521, 1868.39, "highelev"), // the /0017 high-elevation crossing site
        (-33.8688, 151.2093, 58.0, "sydney"),
        (21.3069, -157.8583, 0.0, "honolulu"),
        (1.35, 172.98, 0.0, "kiribati"),
    ];
    // Five reference JDs spanning a year (≈ every 73 days through 2025-26); not tied to real zmanim.
    let day_jds: &[f64] = &[
        2_460_690.0,
        2_460_763.0,
        2_460_836.0,
        2_460_909.0,
        2_460_982.0,
    ];
    // (kind, angle) for the F1/F2/coupling float paths anchored at local noon (or the instant itself).
    let kinds: &[(u32, f64, &str)] = &[
        (0, 16.1, "alot"),
        (1, 8.5, "tzeit"),
        (2, 0.0, "netz"),
        (3, 0.0, "shkia"),
        (4, 0.0, "chatzot"),
        (5, 0.0, "sofzman_gra"),
        (6, 0.0, "sofzman_mga"),
        (9, 0.0, "moon_alt"),
        (13, 0.0, "dayroll"),
        (14, 0.0, "sun_alt"),
        (15, 0.0, "azimuth"),
        (16, 0.5, "terrain"), // synthetic +0.5° skyline crossing
    ];
    for &(lat, lon, elev, sname) in sites {
        for (di, &jd) in day_jds.iter().enumerate() {
            let ref_jd = jd - lon / 360.0; // local-noon anchor
            for &(kind, angle, kname) in kinds {
                let mut label = String::new();
                label.push_str("grid_");
                label.push_str(sname);
                label.push('_');
                label.push_str(kname);
                label.push('_');
                label.push((b'0' + di as u8) as char);
                emit(kind, lat, lon, elev, ref_jd, angle, &label);
            }
        }
    }
    // Year-based kinds (molad + both tekufa methods) across a span of Hebrew years.
    for y in 5780..5792 {
        emit(10, 0.0, 0.0, 0.0, y as f64, 7.0, "grid_molad");
        emit(11, 0.0, 0.0, 0.0, y as f64, 2.0, "grid_tekufa_shmuel");
        emit(12, 0.0, 0.0, 0.0, y as f64, 2.0, "grid_tekufa_ravada");
    }
}
