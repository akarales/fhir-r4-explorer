//! fhir-models — hand-defined FHIR R4 serde models.
//!
//! Philosophy: model exactly the resources this ecosystem uses, strictly.
//! FHIR's full spec is enormous; these structs enforce the fields we rely
//! on (identifiers, codings, value choices) and pass the rest through
//! untouched via `#[serde(flatten)] extra` — lossless round-trips.

pub mod resources;

pub use resources::*;

#[derive(Debug, thiserror::Error)]
pub enum FhirError {
    #[error("invalid FHIR resource: {0}")]
    Invalid(String),
}
