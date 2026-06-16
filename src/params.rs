//! The beginning of the ADR core-domain/0009 parameter-vector, as Rust types. Every halachic
//! convention is an explicit, inspectable **parameter** — the core resolves none. Serializable
//! enums (no_std, alloc-free; CBOR-ready per /0011) so a posek/community supplies the vector and
//! the zman is a pure, auditable, bit-reproducible function of (place, date, shita).

use crate::optics::{HorizonMode, RefractionModel};

/// Rounding stringency direction (halachic l'kula / l'chumra), composed with a read's
/// [`ObligationSense`] so the core derives the per-zman direction (never hard-codes "round down").
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Stringency {
    /// Lenient.
    Lehakel,
    /// Stringent.
    Lehachmir,
    /// Nearest.
    Nearest,
}

/// For a yahrzeit/anniversary that fell in Adar of a common year, which Adar to observe in a
/// **leap** year (ADR core-domain/0014). Halachic knob — the core resolves none.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum AdarAnniversaryRule {
    /// Adar II (Rema / most poskim) — the default.
    AdarII,
    /// Adar I.
    AdarI,
    /// Both (dual observance; the single-date API uses Adar II).
    Both,
}

/// When the Kiddush Levana window *opens* relative to the molad (ADR core-domain/0015). Halachic
/// knob — the core resolves none; genuinely contested between Ashkenaz and Sephardi practice.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum KiddushLevanaStart {
    /// 3 days (72 h) after the molad — Rema / common Ashkenaz. **Default.**
    ThreeDays,
    /// 7 days after the molad — many Sephardim / AriZal.
    SevenDays,
    /// From the molad moment itself (no waiting period).
    Molad,
}

/// When the Kiddush Levana window *closes* relative to the molad (ADR core-domain/0015). Halachic
/// knob — the core resolves none.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum KiddushLevanaEnd {
    /// Half a mean lunar month after the molad (molad + ½·synodic = 14d 18h 22m) — Rema. **Default.**
    HalfMonth,
    /// 15 full days after the molad.
    FifteenDays,
}

/// Whether a zman *opens* or *closes* an obligation — drives stringent rounding direction
/// (a closing zman rounds earlier to be stringent; an opening zman rounds later).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ObligationSense {
    /// Begins an obligation (stringent → round later).
    Opens,
    /// Ends an obligation (stringent → round earlier).
    Closes,
    /// Neither.
    Neutral,
}

/// The optics sub-vector consumed by the solar reads (ADR core-domain/0006/0013).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Optics {
    /// Refraction model for the **horizon** event (netz/shkia). Default Saemundsson.
    pub horizon_refraction: RefractionModel,
    /// Refraction model for **depression** shitot. Default `None` (geometric — classical/halachic).
    pub depression_refraction: RefractionModel,
    /// Sea-level vs visible vs terrain horizon.
    pub horizon_mode: HorizonMode,
}

impl Default for Optics {
    /// The ratified defaults (ADR core-domain/0013): apparent horizon (Saemundsson + visible dip),
    /// geometric depression. A community/posek may override any of these knobs.
    fn default() -> Self {
        Optics {
            horizon_refraction: RefractionModel::Bennett,
            depression_refraction: RefractionModel::None,
            horizon_mode: HorizonMode::Visible,
        }
    }
}
