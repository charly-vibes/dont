# dont-cli-surface Deltas

## MODIFIED Requirements

### Requirement: Universal flags on every subcommand
The system SHALL accept the following flags on every subcommand, parsed before subcommand-specific flags and never conflicting with them: `--help` (`-h`), `--version`, `--json` (`-j`), `--plain`, `--author <id>` (`-a`), and `--direct`. Universal flags SHALL be recognised in any position relative to positional arguments (GNU-style interspersed parsing).

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

### Requirement: No short-flag conflicts
The system SHALL ensure that subcommand-specific short flags do not collide with the universal short flags (`-h`, `-j`, `-a`), and SHALL document the full short-flag mapping via `dont help <cmd>` and the shell-completion generator. The widely reused per-command short flags are `-r` for `--reason` (on `trust`, `undoubt`, `ignore`) and `-e` for `--evidence` (on `flag`).

#### Scenario: help and completions document short flags
- **WHEN** `dont help <cmd>` or the shell-completion generator is queried for a subcommand
- **THEN** the output includes the complete short-flag mapping for that subcommand

## ADDED Requirements

### Requirement: Entity ID resolution and ergonomics
The system SHALL resolve entity identifiers provided as arguments by checking against internal ULIDs (with optional `claim:`/`term:` prefixes), registered CURIEs for coined terms, and unique ULID prefixes.

The system SHALL apply the following resolution priority to resolve ambiguity:
1. **Full ULID Match**: Exact match against a prefixed or unprefixed ULID.
2. **CURIE Match**: Exact match against a registered term CURIE.
3. **Short-ID Prefix**: Unique prefix match against a ULID.

If an identifier is ambiguous within a priority level (e.g. matches multiple CURIEs or multiple ULID prefixes), the command SHALL be refused with a `multiple-matches` error.

#### Scenario: lookup by CURIE
- **WHEN** `dont show WB:P001` is invoked
- **THEN** the system resolves `WB:P001` to its internal `term:ULID` and displays the term

#### Scenario: lookup by short ULID
- **WHEN** `dont show 01KQZ` is invoked and only one entity starts with that prefix
- **THEN** the system resolves the prefix to the full entity ID and proceeds

#### Scenario: kind-prefixing disambiguates short IDs
- **WHEN** `dont show claim:01KQZ` is invoked
- **THEN** only claim entities are searched for the prefix match

#### Scenario: ambiguous identifier produces error
- **WHEN** an identifier matches multiple entities
- **THEN** the command exits `1` with an error listing the ambiguous matches

### Requirement: Stdin ID piping support
The system SHALL support `-` as an identifier argument to read a newline-delimited list of entity IDs or CURIEs from stdin. This allows bulk operations across commands like `show`, `trust`, `flag`, `undoubt`, `ignore`, and `reopen`.

#### Scenario: bulk show via stdin
- **WHEN** `echo "claim:1\nclaim:2" | dont show -` is invoked
- **THEN** the system displays each entity in sequence
- **AND** exits non-zero if any single resolution fails (unless a `--force` flag is specified)
