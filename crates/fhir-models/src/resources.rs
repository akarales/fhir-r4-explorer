//! FHIR R4 resources used across the portfolio: Patient, Observation,
//! Condition, MedicationStatement, and Bundle (transport).
//!
//! Conventions: LOINC codes for lab observations, SNOMED CT for
//! conditions, RxNorm for medications. `extra` maps keep unknown fields
//! lossless.

use std::collections::BTreeMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Typed passthrough for fields we do not model.
pub type Extra = BTreeMap<String, serde_json::Value>;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Reference {
    #[serde(rename = "reference", skip_serializing_if = "Option::is_none")]
    pub reference: Option<String>,
    #[serde(rename = "display", skip_serializing_if = "Option::is_none")]
    pub display: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Coding {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CodeableConcept {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub coding: Vec<Coding>,
    #[serde(rename = "text", skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
}

impl CodeableConcept {
    /// First code with its system, e.g. ("http://loinc.org", "2160-0").
    pub fn first_code(&self) -> Option<(&str, &str)> {
        self.coding.iter().find_map(|c| {
            c.code
                .as_deref()
                .map(|code| (c.system.as_deref().unwrap_or(""), code))
        })
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Patient {
    /// Vestigial: the BundleResource tag owns `resourceType` in transit.
    #[serde(skip)]
    resource_type: String,
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<Vec<HumanName>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub birth_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gender: Option<String>,
    #[serde(flatten)]
    pub extra: Extra,
}

impl Patient {
    pub fn display_name(&self) -> String {
        self.name
            .as_ref()
            .and_then(|names| names.first())
            .map(|n| {
                let family = n.family.as_deref().unwrap_or("");
                let given = n.given.first().map(String::as_str).unwrap_or("");
                format!("{given} {family}").trim().to_string()
            })
            .unwrap_or_else(|| self.id.clone())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HumanName {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub given: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub family: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Observation {
    /// Vestigial: the BundleResource tag owns `resourceType` in transit.
    #[serde(skip)]
    resource_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    pub status: String,
    pub code: CodeableConcept,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subject: Option<Reference>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub effective_date_time: Option<DateTime<Utc>>,
    /// valueQuantity | valueString | valueBoolean — scaffold models the
    /// common ones; the rest pass through `extra`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value_quantity: Option<Quantity>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value_string: Option<String>,
    #[serde(flatten)]
    pub extra: Extra,
}

impl Observation {
    pub fn value_display(&self) -> String {
        if let Some(quantity) = &self.value_quantity {
            return match (&quantity.value, &quantity.unit) {
                (Some(value), Some(unit)) => format!("{value} {unit}"),
                (Some(value), None) => format!("{value}"),
                _ => "—".to_string(),
            };
        }
        if let Some(value) = &self.value_string {
            return value.clone();
        }
        "—".to_string()
    }

    /// LOINC code + display when present.
    pub fn loinc(&self) -> Option<(String, String)> {
        self.code
            .coding
            .iter()
            .find(|c| c.system.as_deref() == Some("http://loinc.org"))
            .map(|c| {
                (
                    c.code.clone().unwrap_or_default(),
                    c.display.clone().unwrap_or_default(),
                )
            })
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Quantity {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unit: Option<String>,
    #[serde(rename = "code", skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Condition {
    /// Vestigial: the BundleResource tag owns `resourceType` in transit.
    #[serde(skip)]
    resource_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub clinical_status: Option<CodeableConcept>,
    pub code: CodeableConcept,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subject: Option<Reference>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub onset_date_time: Option<DateTime<Utc>>,
    #[serde(flatten)]
    pub extra: Extra,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MedicationStatement {
    /// Vestigial: the BundleResource tag owns `resourceType` in transit.
    #[serde(skip)]
    resource_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub medication_codeable_concept: Option<CodeableConcept>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subject: Option<Reference>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub effective_date_time: Option<DateTime<Utc>>,
    #[serde(flatten)]
    pub extra: Extra,
}

/// Bundle transport: typed entries over any FHIR resource we model.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Bundle {
    #[serde(rename = "resourceType", skip_serializing)]
    resource_type: String,
    #[serde(rename = "type")]
    pub bundle_type: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub entry: Vec<BundleEntry>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BundleEntry {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub full_url: Option<String>,
    pub resource: BundleResource,
}

/// Polymorphic entry resource — exactly the types we ingest.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "resourceType")]
pub enum BundleResource {
    Patient(Patient),
    Observation(Observation),
    Condition(Condition),
    MedicationStatement(MedicationStatement),
}

impl BundleResource {
    pub fn patient_reference(&self) -> Option<String> {
        match self {
            BundleResource::Patient(p) => Some(format!("Patient/{}", p.id)),
            BundleResource::Observation(o) => o.subject.as_ref().and_then(|s| s.reference.clone()),
            BundleResource::Condition(c) => c.subject.as_ref().and_then(|s| s.reference.clone()),
            BundleResource::MedicationStatement(m) => {
                m.subject.as_ref().and_then(|s| s.reference.clone())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn patient_round_trip_lossless() {
        let raw = serde_json::json!({
            "resourceType": "Patient",
            "id": "example",
            "gender": "male",
            "birthDate": "1974-12-25",
            "name": [{"given": ["Peter"], "family": "Chalmers"}],
            "unmodeledField": {"nested": [1, 2, 3]}
        });
        let patient: Patient = serde_json::from_value(raw.clone()).expect("valid patient");
        assert_eq!(patient.display_name(), "Peter Chalmers");
        assert!(
            patient.extra.contains_key("unmodeledField"),
            "unknown fields survive"
        );
        let round: serde_json::Value = serde_json::to_value(&patient).expect("serializes");
        assert_eq!(round["id"], "example");
    }

    #[test]
    fn observation_extracts_loinc_and_value() {
        let raw = serde_json::json!({
            "resourceType": "Observation",
            "status": "final",
            "code": {
                "coding": [
                    {"system": "http://loinc.org", "code": "2160-0", "display": "Cholesterol"}
                ],
                "text": "Cholesterol"
            },
            "subject": {"reference": "Patient/example"},
            "effectiveDateTime": "2026-09-01T08:30:00Z",
            "valueQuantity": {"value": 182, "unit": "mg/dL"}
        });
        let observation: Observation = serde_json::from_value(raw).expect("valid observation");
        let (code, display) = observation.loinc().expect("loinc present");
        assert_eq!(code, "2160-0");
        assert_eq!(display, "Cholesterol");
        assert_eq!(observation.value_display(), "182 mg/dL");
        assert_eq!(
            observation.subject.and_then(|s| s.reference).as_deref(),
            Some("Patient/example")
        );
    }

    #[test]
    fn bundle_tagged_entries_deserialize() {
        let raw = serde_json::json!({
            "resourceType": "Bundle",
            "type": "collection",
            "entry": [
                {"fullUrl": "urn:uuid:1", "resource": {
                    "resourceType": "Patient", "id": "p1", "name": [{"given": ["Ada"], "family": "Lovelace"}]
                }},
                {"fullUrl": "urn:uuid:2", "resource": {
                    "resourceType": "Observation", "status": "final",
                    "code": {"coding": [{"system": "http://loinc.org", "code": "8867-4"}]},
                    "subject": {"reference": "Patient/p1"},
                    "valueQuantity": {"value": 72, "unit": "bpm"}
                }}
            ]
        });
        let bundle: Bundle = serde_json::from_value(raw).expect("valid bundle");
        assert_eq!(bundle.entry.len(), 2);
        assert!(matches!(
            bundle.entry[0].resource,
            BundleResource::Patient(_)
        ));
        assert_eq!(
            bundle.entry[1].resource.patient_reference().as_deref(),
            Some("Patient/p1")
        );
    }
}
