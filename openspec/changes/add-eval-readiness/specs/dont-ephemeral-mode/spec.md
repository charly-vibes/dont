## ADDED Requirements

> **Cross-reference:** `--no-persist` is specified as an extension to the universal flag set defined
> in `dont-cli-surface`. Its placement alongside `--json`, `--plain`, and `--strict` is governed by
> the conventions in that spec.

### Requirement: --no-persist universal flag for ephemeral invocations

The system SHALL accept `--no-persist` as a universal flag on every write-capable subcommand. When
`--no-persist` is set, the command SHALL validate the invocation (including hedge-rejection, dedup
checks, and rule evaluation) against the current read-only store state, return the same success or
validation-error envelope it would return if the write had occurred, and write no events to the
store. `--no-persist` SHALL be a no-op on read-only commands (`list`, `show`, `why`, `prime`,
`doctor`, `stats`, `export`, `vocab`, `trace`, `schema`).

#### Scenario: no-persist write command validates and succeeds without writing

- **WHEN** the caller runs `dont conclude "the service restarts cleanly" --no-persist --json`
- **AND** the invocation would succeed in normal mode
- **THEN** the command returns `envelope.ok: true` with the same payload shape as a normal conclude
- **AND** no event is recorded in the store

#### Scenario: no-persist write command validates and fails without writing

- **WHEN** the caller runs `dont trust <id> --reason "maybe" --no-persist --json`
- **AND** the reason string matches a configured hedge pattern
- **THEN** the command returns the hedge-rejection error envelope
- **AND** no event is recorded in the store

#### Scenario: no-persist dedup check fires without writing

- **WHEN** the caller runs `dont conclude <text> --no-persist --json`
- **AND** a claim with equivalent text already exists in the store
- **THEN** the command returns the duplicate-refused error envelope
- **AND** no event is recorded in the store

#### Scenario: no-persist is a no-op on read-only commands

- **WHEN** the caller runs `dont list --no-persist --json`
- **THEN** the flag is silently ignored and the command behaves identically to `dont list --json`

#### Scenario: no-persist does not hold a write lock

- **WHEN** the caller runs any write command with `--no-persist`
- **THEN** the tool acquires no write transaction on the store
- **AND** concurrent write commands are not blocked

### Requirement: --no-persist reflected in the response envelope

The system SHALL include a `ephemeral: true` field in the response envelope when `--no-persist` is
set on a write command, so that callers can programmatically distinguish ephemeral responses from
persisted ones. Read-only commands SHALL NOT include this field regardless of whether `--no-persist`
was passed.

#### Scenario: ephemeral flag in success envelope

- **WHEN** a write command succeeds with `--no-persist`
- **THEN** `envelope.ephemeral` is `true`

#### Scenario: ephemeral flag in error envelope

- **WHEN** a write command returns a validation error with `--no-persist`
- **THEN** `envelope.ephemeral` is `true`

#### Scenario: ephemeral field absent on read commands

- **WHEN** a read-only command is run with or without `--no-persist`
- **THEN** `envelope.ephemeral` is absent from the response

### Requirement: DONT_NO_PERSIST environment variable

The system SHALL honour the `DONT_NO_PERSIST=1` environment variable as equivalent to passing
`--no-persist` on every invocation in the process environment. This allows eval harnesses to set the
flag once for an entire session without modifying individual command invocations.

#### Scenario: DONT_NO_PERSIST=1 makes all writes ephemeral

- **WHEN** `DONT_NO_PERSIST=1` is set in the environment
- **AND** the caller runs any write command without an explicit `--no-persist` flag
- **THEN** the command behaves as if `--no-persist` were passed

#### Scenario: explicit --no-persist takes precedence

- **WHEN** `DONT_NO_PERSIST` is unset and `--no-persist` is passed explicitly
- **THEN** the command behaves in ephemeral mode

#### Scenario: DONT_NO_PERSIST with a value other than "1" is treated as unset

- **WHEN** `DONT_NO_PERSIST=0` (or any value other than the string `"1"`) is set in the environment
- **AND** `--no-persist` is not passed explicitly
- **THEN** the command behaves in normal (persisting) mode
- **AND** the environment variable is treated as if it were unset
