//! Fuzz-for-panic (ADR core-domain/0017 hardening). On the `no_std` device build a panic is
//! `loop {}` — a hung or rebooting board — so "never panics" is a **device-safety** property, not a
//! nicety. We hammer the C-ABI probe (`probe_zman_nanos`, the actual device/FFI surface) and the
//! readers with random + adversarial inputs (extreme lat/lon/elev, NaN/∞, out-of-range month/season,
//! wide years) and assert every call *returns* (a panic would fail the test).
//!
//! **Scope note (flagged, /0017):** time-domain inputs are bounded to sane ranges. The Hebrew
//! conversions (`hebrew_from_fixed`) use a correct-then-adjust search that is only bounded for
//! in-domain Rata Die; an absurd far-future/past instant could iterate for a long time. On-device
//! the clock is a trusted, bounded source, so this is not a live hazard — but input-domain guarding
//! (saturation/clamping at the boundary) is an explicit open robustness item, not silently assumed.

use core_engine::calendar::{
    classify_day, last_day_of_month, last_month_of_year, molad_chalakim, molad_civil, HebrewDate,
};
use core_engine::ffi::probe_zman_nanos;
use core_engine::kiddush_levana::kiddush_levana_window;
use core_engine::params::{KiddushLevanaEnd, KiddushLevanaStart, Realm};
use core_engine::tekufa::{tekufa_civil, Season};
use std::hint::black_box;

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
    fn f64_in(&mut self, lo: f64, hi: f64) -> f64 {
        lo + (self.next_u64() >> 11) as f64 / (1u64 << 53) as f64 * (hi - lo)
    }
    fn i64_in(&mut self, lo: i64, hi: i64) -> i64 {
        lo + (self.next_u64() % ((hi - lo + 1) as u64)) as i64
    }
    /// Occasionally returns an adversarial f64 (extreme / NaN / ±∞), else a value in [lo, hi].
    fn adversarial(&mut self, lo: f64, hi: f64) -> f64 {
        match self.next_u64() % 16 {
            0 => f64::NAN,
            1 => f64::INFINITY,
            2 => f64::NEG_INFINITY,
            3 => 1.0e300,
            4 => -1.0e300,
            _ => self.f64_in(lo, hi),
        }
    }
}

#[test]
fn fuzz_probe_jd_kinds_never_panic() {
    let mut rng = Rng::new(0x66757a7a); // "fuzz"
    let mut finite = 0u64;
    for _ in 0..25_000 {
        // JD-based kinds (incl. 13 day-roll, 14 sun-altitude); sane JD range avoids the unbounded
        // Hebrew search noted above, while lat/lon/elev/angle are fully adversarial.
        let kind = rng.i64_in(0, 18) as u32; // 15..18 are undefined → DOES_NOT_OCCUR
        let lat = rng.adversarial(-95.0, 95.0);
        let lon = rng.adversarial(-200.0, 200.0);
        let elev = rng.adversarial(-500.0, 9000.0);
        let ref_jd = rng.f64_in(2_300_000.0, 2_600_000.0); // ≈ 1500..2900 CE
        let angle = rng.adversarial(-30.0, 120.0);
        let r = probe_zman_nanos(kind, lat, lon, elev, ref_jd, angle);
        black_box(r);
        if r != i64::MIN {
            finite += 1;
        }
    }
    // Sanity: not everything degenerated to the sentinel (the harness is exercising real paths).
    assert!(
        finite > 0,
        "expected some finite results across the fuzz run"
    );
}

#[test]
fn fuzz_probe_year_kinds_never_panic() {
    let mut rng = Rng::new(0x79656172); // "year"
    for _ in 0..50_000 {
        // Year-based kinds: 10 molad (angle=month), 11/12 tekufa (angle=season ordinal). Years and
        // month/season indices include out-of-range values; all are O(1) arithmetic (no search).
        let kind = [10u32, 11, 12][(rng.next_u64() % 3) as usize];
        let year = rng.i64_in(-5000, 15000) as f64;
        let idx = rng.adversarial(-5.0, 20.0); // month (kind 10) or season ordinal (11/12)
        black_box(probe_zman_nanos(kind, 0.0, 0.0, 0.0, year, idx));
    }
}

#[test]
fn fuzz_calendar_readers_never_panic() {
    let mut rng = Rng::new(0x63616c); // "cal"
    for _ in 0..50_000 {
        let year = rng.i64_in(-2000, 12000) as i32;
        // Deliberately allow out-of-range month/day (0, 13/14, 30/31) to probe the match arms.
        let month = rng.i64_in(0, 15) as u8;
        let day = rng.i64_in(0, 31) as u8;
        let date = HebrewDate { year, month, day };
        black_box(classify_day(date, Realm::EretzYisrael));
        black_box(classify_day(date, Realm::Diaspora));
        black_box(molad_chalakim(year, month));
        black_box(molad_civil(year, month));
        black_box(last_month_of_year(year));
        black_box(last_day_of_month(year, month));
        for s in [
            Season::Nisan,
            Season::Tammuz,
            Season::Tishrei,
            Season::Tevet,
        ] {
            black_box(tekufa_civil(
                year,
                s,
                core_engine::params::TekufaMethod::Shmuel,
            ));
        }
        // Valid (year, month) for the KL window (month within the year's range).
        let m = rng.i64_in(1, last_month_of_year(year) as i64) as u8;
        black_box(kiddush_levana_window(
            year,
            m,
            KiddushLevanaStart::ThreeDays,
            KiddushLevanaEnd::HalfMonth,
        ));
    }
}

#[test]
fn fuzz_decoders_never_panic() {
    use core_engine::wire::{decode_horizon_profile, decode_parameter_vector, decode_read_spec};
    // Random byte sequences into the CBOR readers (the device/FFI intake surface). A malformed
    // artifact must yield a typed DecodeError, never a panic (the /0017 invariant — a no_std panic
    // is a hung device). The shipped reader is no-alloc; minicbor decodes without the heap.
    let mut rng = Rng::new(0x6465_636f); // "deco"
    let mut buf = [0u8; 96];
    for _ in 0..100_000 {
        let len = (rng.next_u64() % (buf.len() as u64 + 1)) as usize;
        for b in buf.iter_mut().take(len) {
            *b = (rng.next_u64() & 0xff) as u8;
        }
        let bytes = &buf[..len];
        black_box(decode_parameter_vector(bytes).is_ok());
        black_box(decode_horizon_profile(bytes).is_ok());
        // The read-spec decoder is intake too (/0020, +/0021 OffsetMinutes bound) — must never panic.
        black_box(decode_read_spec(bytes).is_ok());
    }
}
