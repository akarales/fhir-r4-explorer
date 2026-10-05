# AGENTS.md

Build, test, and verification commands for the FHIR R4 Data Explorer.

## Tooling

- **Rust: cargo** — repo root · **JS/TS: pnpm** — `frontend/`

## Rust

```bash
cargo run -p fhir-api            # :8004 (seeded demo bundle)
cargo test --workspace -q        # 10 tests, no infra
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --check
```

## Frontend

```bash
pnpm install && pnpm dev    # :5173 → :8004
pnpm build                 # type check + build
```

## Environment

| Variable | Default | Notes |
|----------|---------|-------|
| `APP_PORT` | `8004` | 8000-8003 taken by ZAP_AGI / apps #1-#2 |
| `APP_DEMO_BUNDLE` | `crates/api/tests/fixtures/demo_bundle.json` | synthetic seed |

## Conventions

- Conventional commits; hygiene hook strips AI attribution
- FHIR structs live ONLY in `crates/fhir-models` — apps #2/#6 reuse the
  crate rather than redefining resources
- New resources: model the fields you use, `#[serde(flatten)] extra`
  passthrough for the rest; never lose data on round-trip
- Code systems: LOINC (observations), SNOMED CT (conditions), RxNorm (meds)
- `cargo test` needs no infra; store is in-memory until the Phase 1
  Postgres backend
