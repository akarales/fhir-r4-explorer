//! In-memory FHIR store: patients, observations, conditions, medication
//! statements — indexed by patient. The Store abstraction mirrors
//! ZAP_RUNTIME's `store/mod.rs`; the Postgres/sqlx backend is Phase 1 of
//! the build plan (resources land as JSONB + indexed columns).

use std::collections::BTreeMap;
use std::sync::{Mutex, RwLock};

use fhir_models::{BundleResource, Condition, MedicationStatement, Observation, Patient};

#[derive(Debug, thiserror::Error)]
pub enum StoreError {
    #[error("unknown patient: {0}")]
    UnknownPatient(String),
    #[error("store error: {0}")]
    Internal(String),
}

#[derive(Default)]
pub struct FhirStore {
    patients: RwLock<BTreeMap<String, Patient>>,
    // patient id -> (taken_at, resource) — sorted on read
    observations: Mutex<BTreeMap<String, Vec<Observation>>>,
    conditions: Mutex<BTreeMap<String, Vec<Condition>>>,
    medications: Mutex<BTreeMap<String, Vec<MedicationStatement>>>,
}

impl FhirStore {
    pub fn new() -> Self {
        Self::default()
    }

    /// Ingest any modeled bundle resource; returns (patients, observations,
    /// conditions, medications) counts.
    pub fn ingest(&self, resource: BundleResource) -> (usize, usize, usize, usize) {
        let patient_ref = resource.patient_reference();
        let patient_id = patient_ref
            .as_deref()
            .and_then(|r| r.strip_prefix("Patient/"))
            .map(|id| id.to_string());

        match resource {
            BundleResource::Patient(patient) => {
                let id = patient.id.clone();
                self.patients
                    .write()
                    .expect("patients lock")
                    .insert(id, patient);
                (1, 0, 0, 0)
            }
            BundleResource::Observation(observation) => {
                let id = patient_id.clone().unwrap_or_default();
                self.observations
                    .lock()
                    .expect("observations lock")
                    .entry(id)
                    .or_default()
                    .push(observation);
                (0, 1, 0, 0)
            }
            BundleResource::Condition(condition) => {
                let id = patient_id.clone().unwrap_or_default();
                self.conditions
                    .lock()
                    .expect("conditions lock")
                    .entry(id)
                    .or_default()
                    .push(condition);
                (0, 0, 1, 0)
            }
            BundleResource::MedicationStatement(medication) => {
                let id = patient_id.clone().unwrap_or_default();
                self.medications
                    .lock()
                    .expect("medications lock")
                    .entry(id)
                    .or_default()
                    .push(medication);
                (0, 0, 0, 1)
            }
        }
    }

    pub fn patients(&self) -> Vec<Patient> {
        self.patients
            .read()
            .expect("patients lock")
            .values()
            .cloned()
            .collect()
    }

    pub fn patient(&self, id: &str) -> Result<Patient, StoreError> {
        self.patients
            .read()
            .expect("patients lock")
            .get(id)
            .cloned()
            .ok_or_else(|| StoreError::UnknownPatient(id.to_string()))
    }

    /// Observations for a patient, chronological; optional LOINC filter.
    pub fn observations(&self, patient_id: &str, loinc: Option<&str>) -> Vec<Observation> {
        let observations = self
            .observations
            .lock()
            .expect("observations lock")
            .get(patient_id)
            .cloned()
            .unwrap_or_default();
        let mut filtered: Vec<Observation> = match loinc {
            Some(code) => observations
                .into_iter()
                .filter(|o| {
                    o.loinc()
                        .map(|(obs_code, _)| obs_code == code)
                        .unwrap_or(false)
                })
                .collect(),
            None => observations,
        };
        filtered.sort_by_key(|o| o.effective_date_time);
        filtered
    }

    pub fn conditions(&self, patient_id: &str) -> Vec<Condition> {
        let mut conditions = self
            .conditions
            .lock()
            .expect("conditions lock")
            .get(patient_id)
            .cloned()
            .unwrap_or_default();
        conditions.sort_by_key(|c| c.onset_date_time);
        conditions
    }

    pub fn medications(&self, patient_id: &str) -> Vec<MedicationStatement> {
        let mut medications = self
            .medications
            .lock()
            .expect("medications lock")
            .get(patient_id)
            .cloned()
            .unwrap_or_default();
        medications.sort_by_key(|m| m.effective_date_time);
        medications
    }
}
