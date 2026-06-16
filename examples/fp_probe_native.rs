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
    ];
    for &(kind, lat, lon, elev, ref_jd, angle, label) in rows {
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
    }
}
