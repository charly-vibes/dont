## ADDED Requirements

### Requirement: Versioned output envelope
The system SHALL wrap all machine-parseable output in a JSON envelope that carries an `envelope_version` field independent of the CLI binary version, and SHALL guarantee that minor envelope versions add fields but MUST NOT remove or rename existing fields. The `envelope_version` starts at `"0.2"` (not `"0.1"`) because the v0.2 spec already committed to this envelope shape. Parsers MUST NOT branch on `cli_version`; it exists for troubleshooting only.

#### Scenario: envelope contains version and CLI version independently
- **WHEN** a command produces JSON output
- **THEN** the envelope contains an `envelope_version` field for the envelope schema version and a `cli_version` field for the binary's semver, and the two version strings are independent of each other

#### Scenario: minor envelope version does not break existing parsers
- **WHEN** a new minor envelope version is released
- **THEN** the new version adds fields but MUST NOT remove or rename fields that existed in the prior minor version

#### Scenario: parsers do not branch on cli_version
- **WHEN** a parser receives an envelope with a `cli_version` field
- **THEN** the parser does not use `cli_version` for feature detection or compatibility branching

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
The system SHALL include a `hints` key on every success envelope. The `hints` array contains ordered `{command, description}` pairs suggesting next actions for agents; it MAY be empty when no contextual action is applicable. Error envelopes SHALL NOT carry `hints`; agents SHALL use `remediation[]` inside the `ErrorResult` payload (see `dont-errors`) for next-action guidance.

#### Scenario: success envelope always carries hints key
- **WHEN** a command succeeds
- **THEN** the envelope includes a `hints` key whose value is an array (possibly empty) of `{command, description}` entries

#### Scenario: error envelopes do not carry hints
- **WHEN** a command fails
- **THEN** the envelope does not include a `hints` key; agents use `remediation[]` inside the `ErrorResult` data payload

#### Scenario: absent hints key treated as empty
- **WHEN** a parser receives a success envelope without a `hints` key
- **THEN** the parser treats it as equivalent to `hints: []`

### Requirement: Rule warnings on non-refusing conditions
The system SHALL include a `warnings` key on every envelope. Each entry SHALL have the shape `{rule_name: string, entity_id?: string, message: string, suggested_remediation?: string}`. Warnings capture non-refusing rule flags, malformed-but-non-blocking inputs, and liveness stale signals.

#### Scenario: warning attached for non-refusing rule flag
- **WHEN** a command succeeds but a non-refusing rule condition was triggered
- **THEN** the envelope includes a `warnings` entry with `rule_name` and `message`, and optionally `entity_id` and `suggested_remediation`

#### Scenario: warnings may appear on error envelopes
- **WHEN** a command is refused but a non-refusing warning also fired during the operation
- **THEN** the error envelope includes the non-refusing warning in `warnings[]` alongside `ok: false`

### Requirement: Execution metadata
The system SHALL include a `meta` object on every envelope carrying `duration_ms` (non-negative integer, milliseconds), `tx` (non-negative integer transaction ID for mutations, `null` for read-only commands), and `request_id` (string spawn request ID when resolving a pending spawn, otherwise `null`). `meta.tx` values MUST fit in the range [1, 2^53-1] when present; parsers MUST NOT assume they fit in a 32-bit integer.

#### Scenario: mutating command includes transaction ID
- **WHEN** a command writes to the store
- **THEN** the envelope's `meta.tx` contains a non-negative integer transaction ID

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
