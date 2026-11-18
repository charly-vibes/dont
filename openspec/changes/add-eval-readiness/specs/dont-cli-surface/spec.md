## MODIFIED Requirements

> **Change:** Adds `--no-persist` to the universal flag set. Full behavioural specification is in
> `dont-ephemeral-mode`.

### Requirement: Universal flags on every subcommand

The system SHALL accept the following flags on every subcommand, parsed before subcommand-specific
flags and never conflicting with them: `--help` (`-h`), `--version`, `--json` (`-j`), `--plain`,
`--author <id>` (`-a`), `--direct`, and (as of this change) `--no-persist`. Universal flags SHALL
be recognised in any position relative to positional arguments (GNU-style interspersed parsing).
`--no-persist` activates ephemeral mode on write-capable commands (as defined in
`dont-ephemeral-mode`) and is a no-op on read-only commands (`list`, `show`, `why`, `prime`,
`doctor`, `stats`, `export`, `vocab`, `trace`, `schema`).

#### Scenario: author flag overrides identity

- **WHEN** a subcommand is invoked with `--author <id>` or `-a <id>`
- **THEN** the author string for that invocation is the provided value instead of the default derived from `$DONT_AUTHOR` or `$USER`

#### Scenario: json flag emits structured envelope

- **WHEN** any subcommand is invoked with `--json` or `-j`
- **THEN** stdout contains only the JSON envelope per the `dont-envelope` contract and human logging moves to stderr

#### Scenario: human mode is default

- **WHEN** a subcommand is invoked without the `--json` flag
- **THEN** the output is human-readable text on stdout
- **AND** it follows the ANSI colour settings from terminal awareness

#### Scenario: plain flag suppresses formatting

- **WHEN** a subcommand is invoked with `--plain`
- **THEN** the output is human-readable text but without ANSI colours or terminal-specific formatting (e.g. for logging to a file)

#### Scenario: direct flag bypasses harness

- **WHEN** a subcommand is invoked with `--direct`
- **THEN** the tool behaves as if `DONT_DIRECT=1` is set, ignoring any parent harness presence (e.g. skipping extra harness-facing hints or envelopes)

#### Scenario: no-persist flag is accepted on all subcommands without error

- **WHEN** the caller appends `--no-persist` to any subcommand invocation
- **THEN** the flag is parsed without error regardless of whether the subcommand is write-capable
  or read-only

#### Scenario: no-persist appears in help and completions

- **WHEN** `dont help <cmd>` or the shell-completion generator is queried for any subcommand
- **THEN** `--no-persist` appears in the flag listing for that subcommand
