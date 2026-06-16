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
