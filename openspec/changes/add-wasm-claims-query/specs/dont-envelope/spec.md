# dont-envelope Specification Delta

## ADDED Requirements

### Requirement: Snapshot envelope kind
The `EnvelopeKind` enumeration SHALL include a `Snapshot` variant (serialized as
`"snapshot"`) used by the `dont export snapshot` command and by the WASM query
module's `load_snapshot` result, so snapshot operations carry a distinct,
self-describing envelope discriminator alongside the existing kinds.

#### Scenario: Snapshot export uses the snapshot kind
- **WHEN** the caller runs `dont export snapshot --json` on an initialized
  project
- **THEN** the response `envelope_kind` SHALL be `"snapshot"`

#### Scenario: WASM load_snapshot returns the snapshot kind
- **WHEN** the WASM `load_snapshot` FFI successfully rehydrates a store
- **THEN** the returned envelope `envelope_kind` SHALL be `"snapshot"`
- **AND** `ok` SHALL be `true`
