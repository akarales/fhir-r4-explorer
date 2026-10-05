//! Router assembly.

pub mod health;
pub mod patients;

use axum::Router;
use axum::routing::{get, post};

use crate::state::AppState;

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health::health))
        .route("/api/v1/bundles", post(patients::ingest_bundle))
        .route("/api/v1/patients", get(patients::list))
        .route("/api/v1/patients/{patient_id}", get(patients::patient))
        .route(
            "/api/v1/patients/{patient_id}/timeline",
            get(patients::timeline),
        )
        .route(
            "/api/v1/patients/{patient_id}/observations",
            get(patients::observations),
        )
        .with_state(state)
}
