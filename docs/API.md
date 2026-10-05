# API Reference

Base URL: `http://localhost:8004`.

## Health

```bash
curl localhost:8004/health
```

```json
{ "status": "ok", "version": "0.1.0" }
```

## Ingest a Bundle

```bash
curl -X POST localhost:8004/api/v1/bundles \
  -H 'content-type: application/json' \
  -d '{
    "resourceType": "Bundle", "type": "collection",
    "entry": [
      { "resource": { "resourceType": "Patient", "id": "zed",
                      "name": [{ "given": ["Zed"], "family": "Test" }] } },
      { "resource": { "resourceType": "Observation", "status": "final",
          "code": { "coding": [{ "system": "http://loinc.org", "code": "8867-4",
                                 "display": "Heart rate" }] },
          "subject": { "reference": "Patient/zed" },
          "effectiveDateTime": "2026-10-01T10:00:00Z",
          "valueQuantity": { "value": 71, "unit": "bpm" } } }
    ]
  }'
```

```json
{ "bundle_type": "collection", "entries": 2, "patients": 1,
  "observations": 1, "conditions": 0, "medications": 0 }
```

Modeled resources only (Patient, Observation, Condition,
MedicationStatement). Unknown fields inside resources pass through
untouched.

## Patient listing / detail

```bash
curl localhost:8004/api/v1/patients
curl localhost:8004/api/v1/patients/ada
```

```json
{ "patients": [ { "id": "ada", "name": "Ada Lovelace",
                  "gender": "female", "birth_date": "1975-12-10" } ] }
```

## Observations (LOINC filter)

```bash
curl 'localhost:8004/api/v1/patients/ada/observations?loinc=4548-4'
```

```json
{ "observations": [ {
  "loinc": { "code": "4548-4", "display": "Hemoglobin A1c" },
  "effective": "2026-01-05T09:00:00+00:00",
  "value": "5.8 %", "status": "final" } ] }
```

## Longitudinal timeline

```bash
curl localhost:8004/api/v1/patients/ada/timeline
```

```json
{ "patient_id": "ada", "events": [
  { "when": "2026-10-05T09:00:00+00:00", "kind": "observation",
    "label": "Cholesterol", "detail": "203.5 mg/dL" },
  { "when": "2023-03-01T09:00:00+00:00", "kind": "medication",
    "label": "Albuterol", "detail": "active" } ] }
```

Events are merged across kinds and sorted latest-first.

## Errors

| Status | Meaning |
|--------|---------|
| `404` | unknown patient |
| `422` | body isn't a modeled FHIR Bundle (extractor rejection — plain-text error body) |
