# dont-envelope Specification

## Purpose
TBD - created by archiving change add-dont-envelope-specs. Update Purpose after archive.
## Requirements
### Requirement: Versioned output envelope

dont SHALL source its output envelope from `genesis::envelope` rather than a local `src/envelope.rs`. The deployed envelope_version `"0.2"` contract and all field semantics (`ok`, `envelope_kind`, `hints`, `warnings`) SHALL be preserved unchanged; genesis's module SHALL conform to this contract.

#### Scenario: envelope shape unchanged after adoption

- **WHEN** `dont prime --json` is run after adopting genesis
- **THEN** the emitted JSON SHALL have top-level keys `ok`, `envelope_version`, `cli_version`, `envelope_kind`, `data`, `warnings`, `hints`, `meta`
- **AND** `envelope_version` SHALL remain `"0.2"`
- **AND** no local `Envelope` struct SHALL remain in `src/envelope.rs`.

#### Scenario: envelope contains version and cli_version independently

- **WHEN** an error envelope is serialized
- **THEN** `data` SHALL contain the structured `ErrorResult` fields (`code`, `rule_name`, `remediation`)
- **AND** `envelope_version` and `cli_version` SHALL be independent top-level fields

#### Scenario: minor envelope version does not break existing parsers

- **GIVEN** `dont --version --json`
- **WHEN** the version envelope is emitted
- **THEN** it SHALL be a structured envelope with `ok: true`, `envelope_kind: "version"`, and `data.version` as a semver string
- **AND** parsers keyed on `envelope_version` SHALL continue to deserialize the envelope

#### Scenario: parsers do not branch on cli_version

- **GIVEN** a CLI error (e.g. `dont conclude "" --json`)
- **WHEN** the error envelope is emitted
- **THEN** it SHALL be a well-formed error envelope with `ok: false` and `envelope_kind: "error"`
- **AND** parsers SHALL NOT branch on `cli_version` to determine envelope structure

### Requirement: Boolean success discriminator
The system SHALL include an `ok` field in every envelope that is `true` for success and `false` for refusal or error, and SHALL set `envelope_kind` to `"error"` when `ok` is `false`.

#### Scenario: successful command produces ok true
- **WHEN** a command completes successfully
- **THEN** the envelope has `ok: true` and `envelope_kind` is the appropriate payload discriminator

#### Scenario: refused or errored command produces ok false
- **WHEN** a command is refused or encounters an error
- **THEN** the envelope has `ok: false` and `envelope_kind` is `"error"`

### Requirement: Typed payload discriminator
The system SHALL include an `envelope_kind` field that discriminates the shape of the `data` field, and SHALL define a canonical set of `envelope_kind` values for envelope version 0.2.

#### Scenario: envelope_kind matches the data payload type
- **WHEN** a command returns a claim payload
- **THEN** `envelope_kind` is `"claim"` and `data` contains the claim payload shape

#### Scenario: canonical envelope_kind values are enumerated
- **WHEN** envelope version is 0.2
- **THEN** the canonical `envelope_kind` values include at minimum: `claim`, `claims`, `term`, `term_list`, `event`, `events`, `spawn_request`, `spawn_requests`, `rule`, `rule_result`, `prime`, `why`, `doctor`, `examples`, `schema_doc`, `version`, `empty`, `error`

### Requirement: Forward-compatible envelope_kind parsing
The system SHALL require that parsers have a default branch for unknown `envelope_kind` values so that new payload types in future minor versions do not break existing parsers.

#### Scenario: parser encounters unknown envelope_kind
- **WHEN** a parser receives an envelope with an `envelope_kind` value not in its known set
- **THEN** the parser handles the envelope through a default branch rather than failing

### Requirement: Agent-addressed hints on success envelopes
The system SHALL include a `hints` key on every success envelope. The `hints` array contains ordered `{command, description}` pairs suggesting next actions for agents; it MAY be empty when no contextual action is applicable. Error envelopes SHALL NOT carry `hints`; agents SHALL use `remediation[]` inside the `ErrorResult` payload (see `dont-errors`) for next-action guidance. A success envelope that omits `hints` is non-conformant for producers. Parsers MAY treat a missing `hints` key as equivalent to `hints: []` for backward-compatibility with historical envelopes, and SHOULD surface a conformance warning when doing so.

#### Scenario: success envelope always carries hints key
- **WHEN** a command succeeds
- **THEN** the envelope includes a `hints` key whose value is an array (possibly empty) of `{command, description}` entries

#### Scenario: error envelopes do not carry hints
- **WHEN** a command fails
- **THEN** the envelope does not include a `hints` key; agents use `remediation[]` inside the `ErrorResult` data payload

#### Scenario: missing hints on success is non-conformant for producers
- **WHEN** a producer emits a success envelope without a `hints` key
- **THEN** that envelope is non-conformant with this specification

#### Scenario: parser tolerates historical envelope missing hints
- **WHEN** a parser receives a historical success envelope that omits `hints`
- **THEN** the parser may treat it as `hints: []`
- **AND** the parser surfaces a conformance warning

### Requirement: Rule warnings on non-refusing conditions
The system SHALL include a `warnings` key on every envelope. Each entry SHALL have the shape `{rule_name: string, entity_id?: string, message: string, suggested_remediation?: string}`. Warnings capture non-refusing rule flags, malformed-but-non-blocking inputs, and liveness stale signals.

#### Scenario: warning attached for non-refusing rule flag
- **WHEN** a command succeeds but a non-refusing rule condition was triggered
- **THEN** the envelope includes a `warnings` entry with `rule_name` and `message`, and optionally `entity_id` and `suggested_remediation`

#### Scenario: warnings may appear on error envelopes
- **WHEN** a command is refused but a non-refusing warning also fired during the operation
- **THEN** the error envelope includes the non-refusing warning in `warnings[]` alongside `ok: false`

### Requirement: Execution metadata
The system SHALL include a `meta` object on every envelope carrying `duration_ms` (non-negative integer, milliseconds), `tx` (transaction ID for mutations, `null` for read-only commands), and `request_id` (string spawn request ID when resolving a pending spawn, otherwise `null`). `meta.tx` MUST be `null` on read-only commands. When present on mutating commands, `meta.tx` MUST be an integer in the range [1, 2^53-1]; parsers MUST NOT assume it fits in a 32-bit integer.

#### Scenario: mutating command includes transaction ID
- **WHEN** a command writes to the store
- **THEN** the envelope's `meta.tx` contains an integer in the range [1, 2^53-1]

#### Scenario: read-only command has null transaction ID
- **WHEN** a command is read-only
- **THEN** the envelope's `meta.tx` is `null`

#### Scenario: spawn resolution populates request_id
- **WHEN** a command resolves a pending spawn request
- **THEN** `meta.request_id` is set to the spawn request's ID

### Requirement: JSON-only stdout in json mode
The system SHALL emit only the JSON envelope on stdout in `--json` mode and SHALL route human-readable logging to stderr. The `--json` flag's behaviour on the CLI surface is defined in `dont-cli-surface`.

#### Scenario: json mode produces only the envelope on stdout
- **WHEN** a command is invoked with `--json`
- **THEN** stdout contains exactly the JSON envelope and no other output
- **AND** any human-readable logging is written to stderr

### Requirement: Entity ID representation in envelopes
The system SHALL format claim identifiers as `claim:<ULID>`, term identifiers as `term:<ULID>`, and spawn request identifiers as `spawn:<ULID>`, producing lexicographically sortable, timestamp-embedded identifiers.

#### Scenario: claim ID is prefixed ULID
- **WHEN** a new claim is created
- **THEN** its identifier has the format `claim:<ULID>` where the ULID is lexicographically sortable and embeds a timestamp

#### Scenario: spawn request ID is prefixed ULID
- **WHEN** a new spawn request is created
- **THEN** its identifier has the format `spawn:<ULID>`

### Requirement: Naming and format conventions
The system SHALL use lower-kebab-case for rule names, event kinds, and status values, and RFC 3339 UTC with whole-second precision (no fractional seconds) for timestamps. Validity tuples SHALL be two-element arrays `[RFC3339-timestamp, boolean]`.

#### Scenario: rule names are lower-kebab-case
- **WHEN** a rule name appears in output
- **THEN** it is formatted as lower-kebab-case

#### Scenario: timestamps are RFC 3339 UTC whole-second
- **WHEN** a timestamp appears in output
- **THEN** it is formatted as RFC 3339 in UTC with whole-second precision (e.g. `2026-04-18T14:20:00Z`)

#### Scenario: validity tuple is a two-element array
- **WHEN** a validity value appears in output
- **THEN** it is a two-element array `[timestamp, boolean]` where `timestamp` follows the RFC 3339 UTC convention
