//! Patient endpoints: bundle ingest, listing, longitudinal timeline.

use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Deserialize;
use serde_json::{Value, json};

use crate::state::AppState;
use crate::store::StoreError;

impl From<StoreError> for Response {
    fn from(err: StoreError) -> Response {
        let message = err.to_string();
        let status = match err {
            StoreError::UnknownPatient(_) => StatusCode::NOT_FOUND,
            StoreError::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };
        (status, Json(json!({ "error": message }))).into_response()
    }
}

pub async fn ingest_bundle(
    State(state): State<AppState>,
    Json(bundle): Json<fhir_models::Bundle>,
) -> Response {
    let (mut patients, mut observations, mut conditions, mut medications) = (0, 0, 0, 0);
    for entry in &bundle.entry {
        let (p, o, c, m) = state.store.ingest(entry.resource.clone());
        patients += p;
        observations += o;
        conditions += c;
        medications += m;
    }
    let payload = json!({
        "bundle_type": bundle.bundle_type,
        "entries": bundle.entry.len(),
        "patients": patients,
        "observations": observations,
        "conditions": conditions,
        "medications": medications,
    });
    (StatusCode::CREATED, Json(payload)).into_response()
}

pub async fn list(State(state): State<AppState>) -> Json<Value> {
    let patients: Vec<Value> = state
        .store
        .patients()
        .into_iter()
        .map(|p| {
            json!({
                "id": p.id,
                "name": p.display_name(),
                "gender": p.gender,
                "birth_date": p.birth_date,
            })
        })
        .collect();
    Json(json!({ "patients": patients }))
}

pub async fn patient(State(state): State<AppState>, Path(patient_id): Path<String>) -> Response {
    match state.store.patient(&patient_id) {
        Ok(patient) => Json(json!({
            "id": patient.id,
            "name": patient.display_name(),
            "gender": patient.gender,
            "birth_date": patient.birth_date,
        }))
        .into_response(),
        Err(err) => err.into(),
    }
}

#[derive(Debug, Deserialize)]
pub struct ObservationQuery {
    /// Filter by LOINC code, e.g. 2160-0.
    pub loinc: Option<String>,
}

pub async fn observations(
    State(state): State<AppState>,
    Path(patient_id): Path<String>,
    Query(params): Query<ObservationQuery>,
) -> Response {
    // Validate the patient first (404 over empty list).
    if let Err(err) = state.store.patient(&patient_id) {
        return err.into();
    }
    let observations = state
        .store
        .observations(&patient_id, params.loinc.as_deref());
    let out: Vec<Value> = observations
        .iter()
        .map(|o| {
            json!({
                "loinc": o.loinc().map(|(code, display)| json!({"code": code, "display": display})),
                "effective": o.effective_date_time.map(|t| t.to_rfc3339()),
                "value": o.value_display(),
                "status": o.status,
            })
        })
        .collect();
    Json(json!({ "observations": out })).into_response()
}

/// Longitudinal timeline: every modeled event for a patient, merged and
/// sorted chronologically — the explorer's centerpiece.
pub async fn timeline(State(state): State<AppState>, Path(patient_id): Path<String>) -> Response {
    if let Err(err) = state.store.patient(&patient_id) {
        return err.into();
    }

    #[derive(serde::Serialize)]
    struct Event {
        when: Option<String>,
        kind: &'static str,
        label: String,
        detail: String,
    }

    let mut events: Vec<Event> = Vec::new();
    for observation in state.store.observations(&patient_id, None) {
        events.push(Event {
            when: observation.effective_date_time.map(|t| t.to_rfc3339()),
            kind: "observation",
            label: observation
                .loinc()
                .map(|(_, display)| display)
                .unwrap_or_else(|| {
                    observation
                        .code
                        .text
                        .clone()
                        .unwrap_or_else(|| "Observation".into())
                }),
            detail: observation.value_display(),
        });
    }
    for condition in state.store.conditions(&patient_id) {
        events.push(Event {
            when: condition.onset_date_time.map(|t| t.to_rfc3339()),
            kind: "condition",
            label: condition
                .code
                .text
                .clone()
                .unwrap_or_else(|| "Condition".into()),
            detail: condition
                .clinical_status
                .and_then(|s| s.text)
                .unwrap_or_else(|| "active".into()),
        });
    }
    for medication in state.store.medications(&patient_id) {
        events.push(Event {
            when: medication.effective_date_time.map(|t| t.to_rfc3339()),
            kind: "medication",
            label: medication
                .medication_codeable_concept
                .as_ref()
                .and_then(|c| c.text.clone())
                .unwrap_or_else(|| "Medication".into()),
            detail: medication.status.clone(),
        });
    }
    events.sort_by(|a, b| b.when.cmp(&a.when));

    Json(json!({
        "patient_id": patient_id,
        "events": events,
    }))
    .into_response()
}
