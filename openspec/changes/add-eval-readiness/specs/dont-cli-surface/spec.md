## MODIFIED Requirements

> **Change:** Adds `--no-persist` to the universal flag set. Full behavioural specification is in
> `dont-ephemeral-mode`.

### Requirement: Universal flags on every subcommand

The system SHALL accept `--no-persist` as an additional universal flag alongside `--help`,
`--version`, `--json`, `--plain`, `--author`, and `--direct`. `--no-persist` SHALL be parsed before
subcommand-specific flags and SHALL be accepted on every subcommand without error. It activates
ephemeral mode on write-capable commands (as defined in `dont-ephemeral-mode`) and is a no-op on
read-only commands (`list`, `show`, `why`, `prime`, `doctor`, `stats`, `export`, `vocab`, `trace`,
`schema`).

#### Scenario: no-persist flag is accepted on all subcommands without error

- **WHEN** the caller appends `--no-persist` to any subcommand invocation
- **THEN** the flag is parsed without error regardless of whether the subcommand is write-capable
  or read-only

#### Scenario: no-persist appears in help and completions

- **WHEN** `dont help <cmd>` or the shell-completion generator is queried for any subcommand
- **THEN** `--no-persist` appears in the flag listing for that subcommand
