# dont-derived-queries Specification Delta

## ADDED Requirements

### Requirement: Portable snapshot export
The system SHALL provide `dont export snapshot` as a read-only export that
dumps the live claims database as a portable, datom-level JSON snapshot
consumable by the WASM query module and other tooling. The snapshot SHALL
include every datom row (`[entity, attribute, value, tx, assert_bit]`) and the
stored `schema_version`, and SHALL be distinct from the existing `--eval`
aggregate export. A `--out <path>` option SHALL write the snapshot to a file;
the default SHALL emit the snapshot JSON to stdout.

#### Scenario: Snapshot export emits datom rows
- **WHEN** the caller runs `dont export snapshot --json` on an initialized
  project
- **THEN** the command returns `envelope_kind: "snapshot"`
- **AND** the payload contains the datom rows and the stored `schema_version`

#### Scenario: Snapshot round-trips through rehydration
- **WHEN** a snapshot produced by `dont export snapshot` is loaded into a
  fresh in-memory store via the WASM rehydration path (or an equivalent
  `Store::open_mem()` consumer)
- **THEN** `list_claims` and `list_terms` on the rehydrated store SHALL equal
  the source store's listings as sets (order-independent), after both sides
  are sorted by a stable key
- **AND** the snapshot SHALL include retracted datoms (`assert_bit = false`)
  so status transitions that retract an old value before asserting a new one
  rehydrate faithfully

#### Scenario: Snapshot written to a file
- **WHEN** the caller runs `dont export snapshot --out snapshot.json`
- **THEN** the snapshot JSON SHALL be written to `snapshot.json`
- **AND** stdout SHALL not contain the snapshot payload

#### Scenario: Snapshot export is read-only
- **WHEN** the caller runs `dont export snapshot` or `dont export snapshot
  --json` without any write flag
- **THEN** the command performs no writes to the project database or any
  project file other than an explicit `--out` target
