# Architecture

## Crates

```
fhir-r4-explorer/
├── crates/fhir-models/   # serde FHIR R4 structs (reused by apps #2/#6 later)
└── crates/api/           # axum service: FhirStore, routes, demo seed
```

## The serde model design

FHIR's full spec is enormous — this crate models **exactly the four
resources the portfolio uses**, with two rules:

1. **Strict where it matters**: identifiers, codings, value choices,
   subject references — typed fields with helpers (`loinc()`,
   `value_display()`, `patient_reference()`).
2. **Lossless everywhere else**: every struct carries
   `#[serde(flatten)] extra: BTreeMap<String, Value>` — unknown fields
   survive ingestion and round-trip untouched. A test ingests a resource
   with a nested unmodeled field and asserts it survives.

`BundleResource` is a `#[serde(tag = "resourceType")]` enum so one POST
ingests a mixed Bundle. Design note: serde's tagged enums **consume** the
tag, so the inner structs' vestigial `resource_type` field is
`#[serde(skip)]` — the enum owns `resourceType` in transit.

## Store (crates/api/src/store.rs)

In-memory `FhirStore` (RwLock/Mutex BTreeMaps): patients by id;
observations, conditions, and medications indexed by patient with
chronological reads. The Store abstraction mirrors ZAP Runtime's
`store/mod.rs`; the Postgres backend (JSONB + indexed columns) is Phase 1
of the roadmap and will become the shared reference persistence for the
biomarker and patient-360 apps.

## Timeline

`GET /patients/{id}/timeline` merges all modeled events per patient and
sorts latest-first — every event carries `when`, `kind`
(observation/condition/medication), `label`, and `detail` so the client
renders one list without per-source requests.

## Code systems

- Observations: **LOINC** (`http://loinc.org`) — the filter key
- Conditions: **SNOMED CT**
- Medications: **RxNorm**

## Design decisions

| Decision | Why |
|----------|-----|
| Hand-written serde models, not a generated full R4 set | The full spec is ~150 resources of mostly-unused surface; four resources + passthrough cover the portfolio's needs with zero codegen maintenance |
| Timeline merged server-side | One endpoint, one render pass — the client never re-derives ordering |
| In-memory store for the scaffold | CI stays DB-free; the Postgres store is the Phase 1 deliverable |
