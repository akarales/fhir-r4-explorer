//! Shared state.

use std::sync::Arc;

use crate::store::FhirStore;

#[derive(Clone)]
pub struct AppState {
    pub store: Arc<FhirStore>,
}
