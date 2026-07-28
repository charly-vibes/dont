# dont-wasm-query Specification Delta

## ADDED Requirements

### Requirement: WASM query module embeds the in-memory Cozo engine
The system SHALL provide a WebAssembly module compiled from the `dont` library
that embeds CozoDB's in-memory engine (`DbInstance::new("mem", ...)`) with no
filesystem, file-lock, or native-SQLite dependency, so that read-only claims
queries can execute in a browser.

#### Scenario: WASM module builds without native storage
- **WHEN** the WASM artifact is built for `wasm32-unknown-unknown`
- **THEN** the build SHALL succeed with `cozo` `features=["wasm"]` and the
  `getrandom` `js` feature enabled
- **AND** the artifact SHALL not require `storage-sqlite`, `storage-rocksdb`,
  or `storage-sled`

#### Scenario: WASM module opens an in-memory store
- **WHEN** the WASM module initializes a store
- **THEN** it SHALL use the Cozo `mem` engine
- **AND** it SHALL NOT attempt to acquire a file lock, read a `tx.seq` sidecar,
  or perform filesystem I/O

### Requirement: Snapshot rehydration from a portable datom dump
The system SHALL rehydrate an in-memory store from a portable JSON snapshot
consisting of datom rows (`[entity, attribute, value, tx, assert_bit]`),
including retracted datoms (`assert_bit = false`) so the rehydrated `datoms`
relation is equivalent to the source. Rehydration SHALL go through a public
`import_datoms` entry point on the `dont` library (the existing internal
`put_datoms` path is reused, not re-exposed as a private surface). The system
SHALL refuse a snapshot whose `schema_version` does not match the module's
compiled expectation.

#### Scenario: Snapshot rehydrates into a queryable store
- **WHEN** the WASM module is given a valid snapshot
- **THEN** it SHALL bulk-insert the datom rows via the public `import_datoms`
  entry point into a fresh in-memory store
- **AND** the rehydrated `datoms` relation SHALL include retracted datoms
  (`assert_bit = false`), making it equivalent to the source relation
- **AND** subsequent `list_claims` / `list_terms` SHALL return data equal to
  the source store at export time

#### Scenario: Repeated load_snapshot resets the store
- **WHEN** the WASM module is given a snapshot while its in-memory store is
  already populated (e.g. the caller invokes `load_snapshot` a second time
  without re-initialising the module)
- **THEN** the module SHALL reset the in-memory store to empty before
  rehydrating, rather than merging the new datoms onto the existing state
- **AND** no partial hybrid state SHALL be left queryable

#### Scenario: Schema-version mismatch is refused
- **WHEN** the WASM module is given a snapshot whose `schema_version` differs
  from its compiled expectation
- **THEN** rehydration SHALL fail with an error envelope
- **AND** no partial state SHALL be left queryable

### Requirement: Shipped rules execute in the WASM module
The system SHALL expose a `run_shipped_rule(name)` FFI that evaluates any of
the shipped rules from the `dont-rule-engine` capability against the
rehydrated in-memory store and returns the matches as JSON, using the same
rule code path as the native CLI.

#### Scenario: Shipped rule runs against rehydrated data
- **WHEN** the caller invokes `run_shipped_rule("ungrounded")` after loading a
  snapshot containing a claim with an unresolved CURIE dependency
- **THEN** the result SHALL contain a match naming that claim
- **AND** the result SHALL equal the matches produced by the native CLI's
  rule evaluation on the same data

#### Scenario: Unknown shipped rule name is rejected
- **WHEN** the caller invokes `run_shipped_rule("not-a-real-rule")`
- **THEN** the result SHALL be an error envelope indicating the rule is unknown

### Requirement: Ad-hoc Datalog query execution is read-only
The system SHALL expose a `run_datalog(script)` FFI that executes a Cozo
Datalog query against the rehydrated in-memory store and returns the result
rows as JSON. The FFI SHALL execute the script in Cozo's immutable
(read-only) mode, so any script Cozo would classify as mutable is rejected
and cannot corrupt the in-memory database.

#### Scenario: Read-only Datalog query returns rows
- **WHEN** the caller invokes `run_datalog` with a `?[...] :=` query
- **THEN** the result SHALL contain the matching rows as JSON
- **AND** the in-memory store SHALL be unchanged

#### Scenario: Mutable Datalog script is rejected
- **WHEN** the caller invokes `run_datalog` with a script Cozo would execute
  as mutable (e.g. one containing a `:put`, `:rm`, `:delete`, `:create`, or
  `:replace` operation)
- **THEN** the result SHALL be an error envelope indicating the script is not
  read-only
- **AND** the in-memory store SHALL be unchanged

### Requirement: FFI returns JSON envelopes
Every WASM FFI entry point SHALL return a JSON string conforming to the
`dont-envelope` envelope shape (the canonical `Envelope` struct: `ok`,
`envelope_version`, `cli_version`, `envelope_kind`, `data`, `warnings`,
`hints`, `ephemeral`, `meta`), so callers handle success and error uniformly
with the CLI convention.

#### Scenario: Successful FFI call returns an envelope
- **WHEN** any FFI call succeeds
- **THEN** the returned JSON SHALL have `ok: true` and a populated `data`
  field

#### Scenario: Failed FFI call returns an error envelope
- **WHEN** any FFI call fails (e.g. bad snapshot, unknown rule, rejected
  script)
- **THEN** the returned JSON SHALL have `ok: false` and an error payload the
  caller can inspect
