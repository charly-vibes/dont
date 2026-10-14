---
id: dont.envelope
kind: intent
statement: "WHEN the migrated delta is elaborated, THE author SHALL replace this scaffold statement with the real requirement."
---

## Constraints

| id | kind | expr | traces_to |
|----|------|------|-----------|
| scaffold_constraint | invariant | `true` | [[dont.envelope]] |

## Model

### States

- `draft`

### Transitions

| id | from | to | guard |
|----|------|----|-------|
| scaffold_transition | draft | draft | [[dont.envelope.scaffold_constraint]] |

## Properties

| id | kind | derives_from | generator | predicate |
|----|------|--------------|-----------|-----------|
| scaffold_property | unit | [[dont.envelope.scaffold_constraint]] | `todo()` | `true` |

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

## Requirements

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
