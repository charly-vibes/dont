## MODIFIED Requirements

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
