//! Binary entry: seed synthetic demo data, serve.

use std::sync::Arc;

use fhir_api::routes;
use fhir_api::state::AppState;
use fhir_api::store::FhirStore;
use tower_http::trace::TraceLayer;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,fhir_api=debug".into()),
        )
        .init();

    let store = Arc::new(FhirStore::new());

    let seed_path = std::env::var("APP_DEMO_BUNDLE")
        .unwrap_or_else(|_| "crates/api/tests/fixtures/demo_bundle.json".to_string());
    if let Ok(raw) = std::fs::read_to_string(&seed_path) {
        match serde_json::from_str::<fhir_models::Bundle>(&raw) {
            Ok(bundle) => {
                for entry in &bundle.entry {
                    store.ingest(entry.resource.clone());
                }
                tracing::info!(entries = bundle.entry.len(), "seeded demo bundle");
            }
            Err(err) => tracing::warn!(%err, "demo bundle failed to parse; starting empty"),
        }
    }

    let port: u16 = std::env::var("APP_PORT")
        .unwrap_or_else(|_| "8004".to_string())
        .parse()
        .expect("APP_PORT must be a valid port");

    let state = AppState { store };
    let app = routes::router(state).layer(TraceLayer::new_for_http());
    let listener = tokio::net::TcpListener::bind(("0.0.0.0", port)).await?;
    tracing::info!(%port, "fhir r4 explorer listening");
    axum::serve(listener, app).await?;
    Ok(())
}
