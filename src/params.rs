//! The beginning of the ADR core-domain/0009 parameter-vector, as Rust types. Every halachic
//! convention is an explicit, inspectable **parameter** — the core resolves none. Serializable
//! enums (no_std, alloc-free; CBOR-ready per /0011) so a posek/community supplies the vector and
//! the zman is a pure, auditable, bit-reproducible function of (place, date, shita).

use crate::optics::{HorizonMode, LimbReference, RefractionModel};

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

/// Israel vs diaspora (`locale.realm`, spec §1.D / ADR core-domain/0001). A **provisioned input**,
/// never derived by the core: the geographic Eretz-Yisrael boundary (Bamidbar 34; contested edges
/// such as Eilat / Aleppo) is set at provisioning. Gates Yom Tov Sheni (coupling #2) and the
/// tal-u-matar basis (coupling #4). Halachic/locale knob — the core resolves none; no default.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Realm {
    /// Eretz Yisrael — one festival day; tal-u-matar fixed at 7 Cheshvan.
    EretzYisrael,
    /// Diaspora — Yom Tov Sheni shel Galuyot; tal-u-matar = 60th day after Tekufat Tishrei.
    Diaspora,
}

/// Which rule starts *tal u-matar* / *she'elat geshamim* (`tal_umatar.basis`, spec §1.D). Halachic
/// knob — the core resolves none; no default (realm normally selects it: EY → `Fixed7Cheshvan`,
/// diaspora → `TekufaBased`).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TalUmatarBasis {
    /// 60th day after (the chosen method's) Tekufat Tishrei — the diaspora rule.
    TekufaBased,
    /// Fixed 7 Cheshvan — the Eretz-Yisrael rule.
    Fixed7Cheshvan,
}

/// Which arithmetic tekufa construct to use (`tekufa.method`, spec §1.D). **Finding (ADR
/// core-domain/0016):** both values are pure *calendar arithmetic* (F3-class), NOT astronomy — so
/// the spec's "F1class.tekufa" label is imprecise; only a future true-astronomical-equinox method
/// would be F1-class. Halachic knob — the core resolves none; no default.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TekufaMethod {
    /// Shmuel: solar year = 365¼ d exactly (Julian); season = 91d 7h 30m. The tal-u-matar default.
    Shmuel,
    /// Rav Ada bar Ahava: solar year = the 19-year Metonic mean (235 synodic months / 19).
    RavAda,
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
    /// Which solar limb defines netz/shkia — the netz-definition dispute (ADR core-domain/0020).
    /// Default `Upper` (the sun first appears); a `±semidiameter` shift, a mechanism knob not policy.
    pub limb: LimbReference,
}

impl Default for Optics {
    /// The ratified defaults (ADR core-domain/0013): apparent horizon (Saemundsson + visible dip),
    /// geometric depression. A community/posek may override any of these knobs.
    fn default() -> Self {
        Optics {
            horizon_refraction: RefractionModel::Bennett,
            depression_refraction: RefractionModel::None,
            horizon_mode: HorizonMode::Visible,
            limb: LimbReference::Upper,
        }
    }
}
