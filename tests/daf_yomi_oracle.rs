//! Daf-yomi differential vs **Hebcal** (ADR core-domain/0022). Hebcal is an independent daf-yomi
//! implementation (same algorithm family as KosherJava); the committed fixture is its output over a
//! date grid covering every tricky masechta — Shekalim, the Meilah/Kinnim/Tamid/Middos continuous-
//! pagination end-block, and Niddah. The oracle is a build/test reference, never shipped; the result is
//! exact (integer daf-yomi has no tolerance).

use core_engine::calendar::fixed_from_gregorian;
use core_engine::daf_yomi::daf_yomi;

const FIXTURE: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/daf_yomi_vectors.csv"
);

#[test]
fn daf_yomi_vs_hebcal() {
    let data = std::fs::read_to_string(FIXTURE).expect("daf-yomi fixture present");
    let (mut pass, mut fail) = (0u32, 0u32);
    for line in data.lines().skip(1) {
        if line.trim().is_empty() {
            continue;
        }
        let f: Vec<&str> = line.split(',').collect();
        let dp: Vec<&str> = f[0].split('-').collect();
        let (y, m, d): (i32, u8, u8) = (
            dp[0].parse().unwrap(),
            dp[1].parse().unwrap(),
            dp[2].parse().unwrap(),
        );
        let want_masechta: u8 = f[1].parse().unwrap();
        let want_daf: u16 = f[2].parse().unwrap();
        let got = daf_yomi(fixed_from_gregorian(y, m, d));
        if got.masechta == want_masechta && got.daf == want_daf {
            pass += 1;
        } else {
            fail += 1;
            eprintln!(
                "!! {} ({}): got (m{},d{}) want (m{},d{})",
                f[0], f[3], got.masechta, got.daf, want_masechta, want_daf
            );
        }
    }
    eprintln!("daf-yomi vs Hebcal: {pass} ok, {fail} fail");
    assert_eq!(fail, 0, "{fail} daf-yomi date(s) diverged from Hebcal");
}
