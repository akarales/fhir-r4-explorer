# Development Guide

Machine-facing commands live in [AGENTS.md](../AGENTS.md).

## Prerequisites

Rust 1.96, cargo · pnpm 11 / Node 24 (frontend).

## Daily loop

```bash
cargo run -p fhir-api        # :8004, seeded with the synthetic demo bundle
cd frontend && pnpm dev      # :5173 → /api proxied
cargo test --workspace -q   # 10 tests — no infra
cargo clippy --workspace --all-targets -- -D warnings
```

The demo seed (`crates/api/tests/fixtures/demo_bundle.json`) contains two
synthetic patients with quarterly labs, a rising cholesterol series,
SNOMED/RxNorm-coded condition and medication, and 26 total entries.

## Testing notes

- Model tests assert the lossless round-trip (unmodeled fields survive),
  LOINC extraction, and tagged-enum Bundle deserialization
- API tests run the router in-process; the ingest test uses ONE app
  instance for its POST-then-GET flow (fresh apps per request would drop
  state — a lesson learned here)

## Gotchas learned here

- **Serde tagged enums consume the tag**: `#[serde(tag = "resourceType")]`
  strips the tag from inner payloads, so inner `resource_type` fields are
  `#[serde(skip)]` and the enum owns the field in transit
- **axum extractor rejections return plain text**, not JSON — error
  tests assert status, not a JSON body
- Port 8004 (8000–8003 are taken on this machine)

## Extending

Add a resource: model the fields you use in
`crates/fhir-models/src/resources.rs`, add the enum variant, extend the
store, and update the timeline mapper — with a round-trip test. The
crate is designed to be reused by the biomarker and patient-360 apps;
keep it dependency-light (serde only).

## Conventions

Conventional commits; hygiene hook strips AI attribution. FHIR structs
live ONLY in `fhir-models` — never duplicate resources in app crates.
