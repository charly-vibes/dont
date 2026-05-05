## ADDED Requirements

This capability defines the shell-facing surface of the `dont` CLI. Exit-code semantics are defined in `dont-errors`; the JSON envelope contract is defined in `dont-envelope`.

### Requirement: Universal flags on every subcommand
The system SHALL accept the following flags on every subcommand, parsed before subcommand-specific flags and never conflicting with them: `--help` (`-h`), `--version`, `--json` (`-j`), `--plain`, `--author <id>` (`-a`), and `--direct`. Universal flags SHALL be recognised in any position relative to positional arguments (GNU-style interspersed parsing).

#### Scenario: help flag prints command help on stdout and exits 0
- **WHEN** any subcommand is invoked with `--help` or `-h`
- **THEN** the output is written to stdout (not stderr), matches `dont help <this-cmd>`, and the process exits `0`

#### Scenario: version flag prints versions and exits 0
- **WHEN** `--version` is invoked without `--json`
- **THEN** the output includes `cli_version` and `envelope_version` on stdout and the process exits `0`

#### Scenario: version flag with json emits version envelope
- **WHEN** `--version --json` is invoked
- **THEN** the envelope has `envelope_kind: "version"` with `data` containing `cli_version` and `envelope_version`, and the process exits `0`

#### Scenario: json flag emits structured envelope
- **WHEN** any subcommand is invoked with `--json` or `-j`
- **THEN** stdout contains only the JSON envelope per the `dont-envelope` contract and human logging moves to stderr

#### Scenario: plain flag suppresses formatting
- **WHEN** any subcommand is invoked with `--plain`
- **THEN** output is uncoloured and unformatted, but hints and warnings are still included in human-readable form

#### Scenario: json wins over plain when both are set
- **WHEN** both `--json` and `--plain` are provided
- **THEN** `--json` takes precedence

#### Scenario: author flag overrides identity
- **WHEN** a subcommand is invoked with `--author <id>` or `-a <id>`
- **THEN** the author string for that invocation is the provided value instead of the default derived from `$DONT_AUTHOR` or `$USER`

#### Scenario: direct flag opts out of harness detection
- **WHEN** a subcommand is invoked with `--direct`
- **THEN** harness mode detection is bypassed and the command runs in direct mode

### Requirement: No short-flag conflicts
The system SHALL ensure that subcommand-specific short flags do not collide with the universal short flags (`-h`, `-j`, `-a`), and SHALL document the full short-flag mapping via `dont help <cmd>` and the shell-completion generator. The widely reused per-command short flags are `-r` for `--reason` (on `trust`, `ignore`) and `-e` for `--evidence` (on `dismiss`).

#### Scenario: subcommand short flag does not conflict with universal flags
- **WHEN** a subcommand defines a short flag
- **THEN** the short flag is not `-h`, `-j`, or `-a`

#### Scenario: help and completions document short flags
- **WHEN** `dont help <cmd>` or the shell-completion generator is queried for a subcommand
- **THEN** the output includes the complete short-flag mapping for that subcommand

### Requirement: Colour and terminal awareness
The system SHALL colour human-mode output by default when stdout is a terminal, SHALL honour the `NO_COLOR` convention (disabling colour when `NO_COLOR` is set to any non-empty value, stdout is not a terminal, or `--plain` is passed), and SHALL support `CLICOLOR_FORCE=1` to force colour even when stdout is redirected. Explicit `--plain` takes precedence over `CLICOLOR_FORCE`.

#### Scenario: colour enabled on terminal stdout
- **WHEN** stdout is a terminal and `NO_COLOR` is not set and `--plain` is not passed
- **THEN** human-mode output includes ANSI colour codes

#### Scenario: colour disabled when stdout is not a terminal
- **WHEN** stdout is redirected to a file or pipe and `NO_COLOR` is not set and `--plain` is not passed and `CLICOLOR_FORCE` is not set
- **THEN** output contains no ANSI colour codes

#### Scenario: colour disabled when NO_COLOR is set
- **WHEN** `NO_COLOR` is set to a non-empty value
- **THEN** output contains no ANSI colour codes

#### Scenario: colour forced with CLICOLOR_FORCE
- **WHEN** `CLICOLOR_FORCE=1` is set and stdout is redirected
- **THEN** output includes ANSI colour codes despite redirection

#### Scenario: plain flag overrides CLICOLOR_FORCE
- **WHEN** both `--plain` and `CLICOLOR_FORCE=1` are set
- **THEN** output contains no ANSI colour codes

### Requirement: Stdin piping for bulk entity operations
The system SHALL accept entity IDs from stdin when the ID argument is `-`, reading one ID per line, emitting one envelope per line in `--json` mode (NDJSON), and processing each line as an independent command invocation with its own transaction boundary. Lines SHALL be stripped of leading and trailing ASCII whitespace (including `\r`) before validation. A line that is empty after stripping SHALL be silently skipped. Invalid IDs produce an error envelope for that line and processing continues.

#### Scenario: entity IDs read from stdin
- **WHEN** a command receives `-` as the entity ID argument
- **THEN** it reads one entity ID per line from stdin and processes each independently

#### Scenario: stdin mode emits NDJSON
- **WHEN** a stdin-piped command runs with `--json`
- **THEN** each processed entity produces one envelope per line on stdout

#### Scenario: invalid ID in stdin produces per-line error
- **WHEN** an invalid entity ID appears in stdin
- **THEN** the command emits an error envelope for that line and continues processing remaining lines

#### Scenario: whitespace and CR/LF handled on stdin lines
- **WHEN** a stdin line contains leading/trailing whitespace or a trailing `\r`
- **THEN** the whitespace is stripped before validation

#### Scenario: stdin exit code is highest severity across all lines
- **WHEN** a stdin run completes with mixed results
- **THEN** the exit code follows the `dont-errors` contract with highest-severity-wins precedence: `4` > `3` > `2` > `1` > `0`

### Requirement: Stdin-accepting verbs
The system SHALL support stdin piping for the following verbs: `show`, `why`, `trust` (with `--reason`), `dismiss` (with `--evidence`), `ignore` (with `--reason`), `lock`, `reopen`, `verify-evidence`. The system SHALL NOT support stdin piping for `conclude` or `define` (which take domain content, not just IDs), nor for `list` or `vocab` (which are sources, not sinks). Verbs not yet specified by a capability spec (`show`, `why`, `list`, `vocab`) are listed here for completeness and will be defined in future capabilities.

#### Scenario: show accepts stdin IDs
- **WHEN** `dont show -` is invoked with entity IDs on stdin
- **THEN** each entity is looked up and its envelope emitted

#### Scenario: conclude rejects stdin
- **WHEN** `dont conclude -` is invoked
- **THEN** the command exits `2` with a `usage` error; it does not read from stdin

### Requirement: Shell completion generation
The system SHALL provide `dont completions <shell>` that prints a shell-completion script for `bash`, `zsh`, `fish`, `powershell`, and `elvish` in each shell's native format, covering subcommands, universal flags, subcommand flags, and enum-valued flags (including at minimum status values, mode values, and shell names). The `completions` subcommand SHALL ignore `--json` and always emit the raw shell script to stdout.

#### Scenario: completions generate valid shell script
- **WHEN** `dont completions fish` is invoked
- **THEN** stdout contains a valid fish completion script with `complete -c` entries

#### Scenario: completions for unknown shell exits with usage error
- **WHEN** `dont completions <unknown-shell>` is invoked
- **THEN** the process exits `2` with a `usage` error listing supported shell names

#### Scenario: dynamic entity ID completion with bounded latency
- **WHEN** a completion script queries the local store for entity IDs matching a prefix
- **THEN** the query completes within a 10ms timeout or exits with no dynamic candidates rather than blocking the shell prompt

### Requirement: Help surface structure
The system SHALL provide `dont help` and `dont help <cmd>` as the primary agent-addressed help surface, `dont help --tutorial` for the walkthrough, `dont help --howto <topic>` for how-to guides, and `dont help --topics` to list available tutorials and how-to guides. Help output follows a Diataxis structure: tutorial, how-to, and reference modalities.

#### Scenario: bare help lists available commands
- **WHEN** `dont help` is invoked with no arguments
- **THEN** the output lists all available subcommands with brief descriptions

#### Scenario: command help shows usage and flags
- **WHEN** `dont help <cmd>` is invoked
- **THEN** the output is written to stdout and includes the command's usage line, flags (including short flags), and a description

#### Scenario: tutorial help prints the walkthrough
- **WHEN** `dont help --tutorial` is invoked
- **THEN** the output prints the canonical tutorial walkthrough

#### Scenario: topics lists available help entries
- **WHEN** `dont help --topics` is invoked
- **THEN** the output lists all available tutorials and how-to guides
