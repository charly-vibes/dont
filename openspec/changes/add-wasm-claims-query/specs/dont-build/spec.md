# dont-build Specification Delta

## ADDED Requirements

### Requirement: WASM query artifact built alongside the native binary
The system SHALL produce a WebAssembly `cdylib` artifact compiled from the
`dont` library for the `wasm32-unknown-unknown` target, in addition to the
native single binary. The WASM artifact SHALL be built in continuous
integration and staged as a static asset under a content-hashed or
version-tagged path for the documentation site. The native single-binary
distribution SHALL remain unchanged by the addition of the WASM target.

The `dont` library `Cargo.toml` SHALL make the Cozo storage-backend and WASM
features independently selectable so the same library source compiles to
both the native binary (with `storage-sqlite` / `storage-sqlite-src`) and the
WASM artifact (with `cozo` `features=["wasm"]` and the `getrandom` `js`
feature) via target-gated or optional features, without editing the manifest
per target.

#### Scenario: CI builds the WASM artifact
- **WHEN** CI runs for a pull request or push to the default branch
- **THEN** the workflow builds the WASM artifact for `wasm32-unknown-unknown`
- **AND** the job fails if the WASM build fails

#### Scenario: Native binary is unaffected by the WASM target
- **WHEN** the native `dont` binary is built
- **THEN** the WASM-only dependencies (`wasm-bindgen`, `cozo` `wasm` feature,
  `getrandom` `js` feature) SHALL NOT be linked into the native binary
- **AND** the native binary retains its single-binary, embedded-SQLite storage
  characteristics

#### Scenario: WASM artifact is staged for the docs site
- **WHEN** the documentation site is built
- **THEN** the built WASM artifact is available as a static asset under a
  content-hashed or version-tagged path under the docs source
- **AND** the claims-query page can load it

#### Scenario: WASM artifact stays within a size budget
- **WHEN** the WASM artifact is built and optimised (e.g. with `wasm-opt -Oz`)
- **THEN** the gzipped artifact SHALL be smaller than 5 MB
- **AND** exceeding the budget SHALL fail CI or trigger a documented trim
  follow-up rather than shipping unbounded size
