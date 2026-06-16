//! Newtypes that make the apparent-vs-geometric and unit confusions **unrepresentable**
//! (ADR core-domain/0013). Refraction (`optics`) is the ONLY way to turn a `GeometricAltitude`
//! into an `ApparentAltitude`; the root-finder works on whichever the read calls for.

/// Geometric (airless / true) solar altitude in degrees — what `geometry` computes directly.
/// Depression-angle shitot are defined against this (refraction-independent; ADR core-domain/0013).
#[derive(Clone, Copy, PartialEq, PartialOrd, Debug)]
pub struct GeometricAltitude(pub f64);

/// Apparent (refraction-corrected) solar altitude in degrees — `geometric + refraction`.
/// netz/shkia (the visible horizon event) are defined against this.
#[derive(Clone, Copy, PartialEq, PartialOrd, Debug)]
pub struct ApparentAltitude(pub f64);

impl GeometricAltitude {
    /// The geometric altitude in degrees.
    #[inline]
    pub fn deg(self) -> f64 {
        self.0
    }
}

impl ApparentAltitude {
    /// The apparent altitude in degrees.
    #[inline]
    pub fn deg(self) -> f64 {
        self.0
    }
}
