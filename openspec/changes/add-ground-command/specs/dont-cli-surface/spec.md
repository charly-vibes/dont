## ADDED Requirements
### Requirement: Ground is exposed as a standard subcommand
The system SHALL expose `ground` through the standard command surfaces used for other verbs. Bare help, command-specific help, and shell completions MUST list `ground` as an available subcommand with its statement-plus-evidence purpose.

#### Scenario: bare help lists ground
- **WHEN** the caller runs `dont help`
- **THEN** the available subcommands include `ground` with a brief description of one-shot claim grounding

#### Scenario: completions include ground
- **WHEN** the caller generates shell completions
- **THEN** the completion script includes `ground` among the available subcommands

### Requirement: Ground does not participate in stdin ID bulk mode
The system SHALL treat `ground` like `conclude` and `define` for stdin bulk semantics: it takes domain content rather than an entity ID sink and therefore SHALL NOT read entity IDs from stdin via `-`.

#### Scenario: ground rejects stdin ID bulk mode
- **WHEN** `dont ground -` is invoked
- **THEN** the command exits with a `usage` error rather than reading entity IDs from stdin
