//! Integration tests: router over seeded in-memory FHIR store.

use std::sync::Arc;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use serde_json::Value;
use tower::ServiceExt;

use fhir_api::routes;
use fhir_api::state::AppState;
use fhir_api::store::FhirStore;

fn seeded_store() -> FhirStore {
    let store = FhirStore::new();
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/demo_bundle.json"
    );
    let bundle: fhir_models::Bundle =
        serde_json::from_str(&std::fs::read_to_string(path).expect("fixture exists"))
            .expect("fixture is valid FHIR");
    for entry in &bundle.entry {
        store.ingest(entry.resource.clone());
    }
    store
}

async fn app() -> axum::Router {
    routes::router(AppState {
        store: Arc::new(seeded_store()),
    })
}

async fn get(path: &str) -> (StatusCode, Value) {
    let response = app()
        .await
        .oneshot(
            Request::get(path)
                .body(Body::empty())
                .expect("request builds"),
        )
        .await
        .expect("in-process request");
    let status = response.status();
    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body");
    (status, serde_json::from_slice(&bytes).expect("json"))
}

#[tokio::test]
async fn health_ok() {
    let (status, body) = get("/health").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["status"], "ok");
}

#[tokio::test]
async fn seeded_patients_listed() {
    let (status, body) = get("/api/v1/patients").await;
    assert_eq!(status, StatusCode::OK);
    let patients = body["patients"].as_array().expect("patients");
    assert_eq!(patients.len(), 2);
    assert_eq!(patients[0]["id"], "ada");
    assert_eq!(patients[0]["name"], "Ada Lovelace");
}

#[tokio::test]
async fn observations_filter_by_loinc() {
    let (status, all) = get("/api/v1/patients/ada/observations").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(all["observations"].as_array().expect("all").len(), 18);

    let (status, filtered) = get("/api/v1/patients/ada/observations?loinc=4548-4").await;
    assert_eq!(status, StatusCode::OK);
    let observations = filtered["observations"].as_array().expect("filtered");
    assert_eq!(observations.len(), 2);
    assert_eq!(observations[0]["loinc"]["code"], "4548-4");
}

#[tokio::test]
async fn observations_unknown_patient_404() {
    let (status, _) = get("/api/v1/patients/nobody/observations").await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn timeline_merges_events_chronologically() {
    let (status, body) = get("/api/v1/patients/ada/timeline").await;
    assert_eq!(status, StatusCode::OK);
    let events = body["events"].as_array().expect("events");
    // 18 observations + 1 condition + 1 medication
    assert_eq!(events.len(), 20);
    let kinds: Vec<&str> = events
        .iter()
        .map(|e| e["kind"].as_str().expect("kind"))
        .collect();
    assert!(kinds.contains(&"observation"));
    assert!(kinds.contains(&"condition"));
    assert!(kinds.contains(&"medication"));
    // Descending order (latest first).
    let times: Vec<&str> = events.iter().filter_map(|e| e["when"].as_str()).collect();
    let mut sorted = times.clone();
    sorted.sort();
    sorted.reverse();
    assert_eq!(times, sorted, "events must be sorted latest-first");
}

#[tokio::test]
async fn ingest_new_bundle_via_post() {
    let bundle = serde_json::json!({
        "resourceType": "Bundle", "type": "collection",
        "entry": [
            {"resource": {"resourceType": "Patient", "id": "zed",
                          "name": [{"given": ["Zed"], "family": "Test"}]}},
            {"resource": {"resourceType": "Observation", "status": "final",
                          "code": {"coding": [{"system": "http://loinc.org", "code": "8867-4", "display": "Heart rate"}]},
                          "subject": {"reference": "Patient/zed"},
                          "effectiveDateTime": "2026-10-01T10:00:00Z",
                          "valueQuantity": {"value": 71, "unit": "bpm"}}}
        ]
    });
    // One app instance: the POST must be visible to the follow-up GET.
    let app = app().await;
    let response = app
        .clone()
        .oneshot(
            Request::post("/api/v1/bundles")
                .header("content-type", "application/json")
                .body(Body::from(bundle.to_string()))
                .expect("request builds"),
        )
        .await
        .expect("in-process request");
    assert_eq!(response.status(), StatusCode::CREATED);
    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body");
    let body: Value = serde_json::from_slice(&bytes).expect("json");
    assert_eq!(body["entries"], 2);
    assert_eq!(body["patients"], 1);
    assert_eq!(body["observations"], 1);

    let response = app
        .oneshot(
            Request::get("/api/v1/patients/zed/timeline")
                .body(Body::empty())
                .expect("request builds"),
        )
        .await
        .expect("in-process request");
    assert_eq!(response.status(), StatusCode::OK);
    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body");
    let body: Value = serde_json::from_slice(&bytes).expect("json");
    assert_eq!(body["events"].as_array().expect("events").len(), 1);
}

#[tokio::test]
async fn invalid_bundle_rejected() {
    // The extractor rejects before the handler; the body is plain text.
    let response = app()
        .await
        .oneshot(
            Request::post("/api/v1/bundles")
                .header("content-type", "application/json")
                .body(Body::from("{\"foo\": 1}"))
                .expect("request builds"),
        )
        .await
        .expect("in-process request");
    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
}
