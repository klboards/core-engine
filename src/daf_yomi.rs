//! **Daf Yomi** (Talmud Bavli) — the deterministic daily-page learning cycle (ADR core-domain/0022).
//!
//! A fixed **civil-day** cycle: exactly one daf per day, `DAF_YOMI_CYCLE_DAYS` (2711) days per cycle,
//! anchored at the 14th-cycle epoch (2020-01-05 = Berachos 2). Pure integer arithmetic on [`RataDie`]
//! — offline-forever, no floating point, native==wasm exact by construction. Periodic, so the modulo
//! resolves any date; the **modern masechta table** (Yerushalmi Shekalim = 22 daf) is correct for every
//! cycle from the 8th (1975-06-24) onward, which is all the device ever shows (present + future).
//!
//! Returns the masechta **index** + daf — *not* a name. Masechta names are localizable display content
//! (the management/content layer maps index → he/en/yi/…), keeping the engine label-free.

use crate::calendar::{fixed_from_gregorian, RataDie};

/// Days in one Bavli daf-yomi cycle (∑ of per-masechta day-counts).
pub const DAF_YOMI_CYCLE_DAYS: i64 = 2711;

/// Last daf of each masechta, in daf-yomi cycle order (index 0..39). Each masechta is learned from daf 2
/// to this value, so it occupies `last - 1` days; ∑(last − 1) = 2751 − 40 = [`DAF_YOMI_CYCLE_DAYS`].
/// Matches KosherJava's `blattPerMasechta` (modern table). The four end-masechtos Meilah/Kinnim/Tamid/
/// Middos share continuous Vilna-Shas pagination — handled by the display offsets in [`daf_yomi`].
const MASECHTA_LAST_DAF: [u16; 40] = [
    64, 157, 105, 121, 22, 88, 56, 40, 35, 31, 32, 29, 27, 122, 112, 91, 66, 49, 90, 82, 119, 119,
    176, 113, 24, 49, 76, 14, 120, 110, 142, 61, 34, 34, 28, 22, 4, 9, 5, 73,
];

/// A daf-yomi page: the masechta index (0..39, cycle order) and the displayed daf number.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct DafYomi {
    /// Masechta index in daf-yomi cycle order (0 = Berachos … 39 = Niddah).
    pub masechta: u8,
    /// Displayed daf number (≥ 2; Kinnim/Tamid/Middos continue Meilah's pagination).
    pub daf: u16,
}

/// The Bavli daf-yomi page for the civil day `rd` (ADR core-domain/0022). Deterministic, integer-only.
/// Correct for the **entire** daf-yomi era (cycle 1, 1923-09-11, onward): before the 8th cycle
/// (1975-06-24) Yerushalmi Shekalim was 13 daf (cycle = 2702 days); from the 8th cycle it is 22 (cycle =
/// 2711). The two eras are clean cycle boundaries, so each resolves by its own epoch + table.
pub fn daf_yomi(rd: RataDie) -> DafYomi {
    // The 8th-cycle start, when Shekalim moved 13 → 22 daf.
    let shekalim_change = fixed_from_gregorian(1975, 6, 24).0;
    let (epoch, shekalim_last) = if rd.0 >= shekalim_change {
        (fixed_from_gregorian(2020, 1, 5).0, 22u16) // modern: cycle-14 epoch, Shekalim 22
    } else {
        (fixed_from_gregorian(1923, 9, 11).0, 13u16) // historical: cycle-1 epoch, Shekalim 13
    };
    // Per-masechta last-daf with the era's Shekalim (index 4) substituted; cycle = ∑(last−1).
    let last_of = |i: usize| -> u16 {
        if i == 4 {
            shekalim_last
        } else {
            MASECHTA_LAST_DAF[i]
        }
    };
    let cycle: i64 = (0..MASECHTA_LAST_DAF.len())
        .map(|i| last_of(i) as i64 - 1)
        .sum();
    // 0-based day within the cycle. rem_euclid handles dates before the epoch too (periodic).
    let mut n = (rd.0 - epoch).rem_euclid(cycle);
    let mut i = 0usize;
    while i < MASECHTA_LAST_DAF.len() {
        let days = last_of(i) as i64 - 1; // learned daf 2..=last ⇒ last-1 days
        if n < days {
            // Internal daf = 2 + offset into the masechta; the Meilah-block masechtos continue Meilah's
            // pagination, so their *displayed* daf is shifted (Kinnim +21, Tamid +24, Middos +32).
            let display_shift: u16 = match i {
                36 => 21,
                37 => 24,
                38 => 32,
                _ => 0,
            };
            return DafYomi {
                masechta: i as u8,
                daf: n as u16 + 2 + display_shift,
            };
        }
        n -= days;
        i += 1;
    }
    // Unreachable: ∑(last−1) == DAF_YOMI_CYCLE_DAYS and n < that. Defensive fallback (no panic on device).
    DafYomi {
        masechta: 39,
        daf: 73,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn epoch_is_berachos_2() {
        // 2020-01-05 = cycle-14 start = Berachos (index 0) daf 2.
        let d = daf_yomi(fixed_from_gregorian(2020, 1, 5));
        assert_eq!(
            d,
            DafYomi {
                masechta: 0,
                daf: 2
            }
        );
    }

    #[test]
    fn day_counts_sum_to_cycle_length() {
        let total: i64 = MASECHTA_LAST_DAF.iter().map(|&d| d as i64 - 1).sum();
        assert_eq!(total, DAF_YOMI_CYCLE_DAYS);
    }

    #[test]
    fn boundary_berachos_to_shabbos() {
        // Berachos occupies 63 days (daf 2..64); Shabbos 2 starts on day 63 (2020-03-08).
        assert_eq!(
            daf_yomi(fixed_from_gregorian(2020, 3, 8)),
            DafYomi {
                masechta: 1,
                daf: 2
            }
        );
    }

    #[test]
    fn meilah_block_display_offsets() {
        // The Vilna-Shas continuous pagination (Hebcal-verified dates).
        assert_eq!(
            daf_yomi(fixed_from_gregorian(2027, 3, 14)),
            DafYomi {
                masechta: 36,
                daf: 24
            } // Kinnim 24 (+21)
        );
        assert_eq!(
            daf_yomi(fixed_from_gregorian(2027, 3, 20)),
            DafYomi {
                masechta: 37,
                daf: 30
            } // Tamid 30 (+24)
        );
        assert_eq!(
            daf_yomi(fixed_from_gregorian(2027, 3, 25)),
            DafYomi {
                masechta: 38,
                daf: 35
            } // Midos 35 (+32)
        );
    }
}
