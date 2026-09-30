# dont-harness-cli Specification

## Purpose

Defines the invocation contract between `dont` and its callers — human operators, CI
pipelines, and LLM harnesses. Covers project directory resolution (`DONT_DIR`), config
loading, the `--help` / `after_help` surface, and the subprocess protocol that external
tools use when spawning `dont` as a child process.

## Requirements

### Requirement: Project directory resolution

The system SHALL resolve the active project directory through a deterministic two-step
lookup. First it MUST check the `DONT_DIR` environment variable; if set to a non-empty
value, that path is used as the project directory without walking the filesystem. If
`DONT_DIR` is absent or empty, the system MUST walk upward from the current working
directory until it finds a `.dont/` directory with a `config.toml` inside it, or exhaust
the filesystem hierarchy. A walk that finds no `.dont/` directory MUST produce a
`config-missing` error with exit code `3`.

#### Scenario: DONT_DIR overrides filesystem walk

- **WHEN** `DONT_DIR` is set to a non-empty path before `dont` is invoked
- **THEN** the system uses that path as the project directory without walking the current working directory upward

#### Scenario: filesystem walk finds nearest ancestor

- **WHEN** `DONT_DIR` is unset and the current working directory is a subdirectory of a project
- **THEN** the system walks upward and uses the nearest ancestor that contains `.dont/config.toml`

#### Scenario: no project directory produces config-missing error

- **WHEN** `DONT_DIR` is unset and the filesystem walk reaches the root without finding `.dont/config.toml`
- **THEN** the process exits with code `3` and the envelope has `code: "config-missing"` with a remediation entry suggesting `dont init`

#### Scenario: DONT_DIR path that lacks config.toml is rejected

- **WHEN** `DONT_DIR` points to a directory that exists but does not contain `config.toml`
- **THEN** the process exits with code `3` and the envelope has `code: "config-missing"`

### Requirement: Config validation on every invocation

The system SHALL validate `config.toml` before executing any command payload. Validation
MUST parse the TOML file, check all known fields for type correctness and valid values,
and reject the invocation with a structured error when any field is invalid. The error
envelope MUST carry `code: "config-invalid"` and a `message` identifying the offending
field. Invalid configuration MUST be surfaced before any store mutation is attempted.

#### Scenario: invalid config field aborts invocation

- **WHEN** `config.toml` contains a field with an invalid type or out-of-range value
- **THEN** the process exits with code `3` before any store mutation
- **AND** the envelope has `code: "config-invalid"` and `message` names the offending field

#### Scenario: valid config proceeds to command execution

- **WHEN** `config.toml` parses and validates successfully
- **THEN** the system proceeds to execute the requested command

### Requirement: Author resolution order

The system SHALL resolve the author string for each invocation through a fixed priority
chain: the `--author` / `-a` flag (highest priority), then the `$DONT_AUTHOR` environment
variable, then the `$USER` environment variable. If none of these yields a value the
author is omitted from the event record for that invocation. The resolved author is stored
only on events produced during that invocation and does not persist to `config.toml`.

#### Scenario: explicit flag takes highest priority

- **WHEN** `--author agent:ci-bot` is supplied and `DONT_AUTHOR=human:alice` is set
- **THEN** the author recorded on the resulting event is `agent:ci-bot`

#### Scenario: DONT_AUTHOR overrides USER

- **WHEN** `DONT_AUTHOR=agent:ci-bot` is set and `USER=alice` is also set and no `--author` flag is provided
- **THEN** the author recorded on the resulting event is `agent:ci-bot`

#### Scenario: USER is the last fallback

- **WHEN** neither `--author` nor `DONT_AUTHOR` provides a value and `USER=alice` is set
- **THEN** the author recorded on the resulting event is `alice`

### Requirement: Help surface contract

The system SHALL expose two equivalent surfaces for per-command help: `dont help <cmd>`
and `dont <cmd> --help`. Both MUST produce the same output. The help output for each
command MUST include the command description, all flags with their types and defaults,
and an `after_help` examples block showing the canonical invocation patterns for that
command. The examples block MUST use the exact form `dont <cmd> <typical-args>` and MUST
cover at least the most common successful invocation and one invocation showing a
non-default flag. The global `--help` output (bare `dont --help` or `dont help`) MUST
list every subcommand with a one-line description and conclude with an `after_help`
block listing representative session-start examples.

#### Scenario: subcommand help is reachable two ways

- **WHEN** the caller runs `dont lock --help`
- **THEN** the output is identical to `dont help lock`

#### Scenario: every command has an examples block

- **WHEN** the caller runs `dont <cmd> --help` for any implemented subcommand
- **THEN** the output contains an examples section showing at least one canonical invocation

#### Scenario: global help lists all subcommands

- **WHEN** the caller runs `dont --help` or `dont help`
- **THEN** the output lists every subcommand with a one-line description
- **AND** the output ends with a representative session-start examples block

#### Scenario: after_help examples use canonical flag form

- **WHEN** the examples block of a subcommand shows a flag
- **THEN** it uses the long form (e.g. `--status`, `--evidence`) rather than the short form
- **AND** each example is a complete, runnable `dont` invocation

### Requirement: Subprocess invocation protocol

The system SHALL define a stable contract for external tools (CI pipelines, LLM harnesses,
shell wrappers) that invoke `dont` as a subprocess. A conformant subprocess caller MUST
always pass `--json` to get machine-readable output on stdout, MUST treat the process exit
code as the primary routing signal before parsing the envelope body, and MUST read the
full stdout before inspecting the exit code to avoid SIGPIPE on large outputs. On exit
code `0` the caller SHOULD read `data` for payload and `warnings[]` for non-blocking
notices. On exit code `1` the caller MUST read `data.remediation[0].command` and act on
it rather than retrying the same invocation. On exit codes `3` and `4` the caller MUST
stop LLM-driven retry and escalate to operator intervention. Exit code `2` indicates a
usage error in the caller's own invocation and MUST be treated as a caller bug.

#### Scenario: subprocess caller routes on exit code before parsing body

- **WHEN** a subprocess caller receives a non-zero exit from `dont`
- **THEN** it branches first on the exit code (0/1/2/3/4) before inspecting the JSON body

#### Scenario: exit code 1 directs caller to remediation

- **WHEN** a subprocess caller receives exit code `1`
- **THEN** it reads `data.remediation[0].command` and issues that command as the next action
- **AND** it does not retry the refused invocation unchanged

#### Scenario: exit codes 3 and 4 halt LLM retry

- **WHEN** a subprocess caller receives exit code `3` or `4`
- **THEN** it does not attempt further LLM-driven retries
- **AND** it surfaces the error envelope to the operator for manual intervention

#### Scenario: exit code 2 is treated as caller error

- **WHEN** a subprocess caller receives exit code `2`
- **THEN** it treats this as a bug in its own argument construction
- **AND** reads `data.message` to understand the malformed argument

#### Scenario: subprocess caller reads full stdout before exit code

- **WHEN** a subprocess caller runs `dont` with `--json`
- **THEN** it reads all of stdout before checking the process exit code to prevent SIGPIPE on the dont side

### Requirement: Subprocess output routing

The system SHALL route all JSON envelope output to stdout and all human-readable
diagnostic text (warnings, errors in non-json mode) to stderr. When `--json` is active,
stdout MUST contain only the JSON envelope line and nothing else; log noise or debug text
MUST go to stderr. A subprocess caller that captures only stdout while discarding stderr
MUST receive a clean, parseable JSON line.

#### Scenario: JSON mode writes only the envelope to stdout

- **WHEN** `dont <cmd> --json` is invoked
- **THEN** stdout contains exactly one JSON object per invocation (or one per stdin line in NDJSON mode)
- **AND** no human-readable prose appears on stdout

#### Scenario: stderr carries warnings and diagnostic text

- **WHEN** `dont <cmd>` is invoked in human mode and warnings are produced
- **THEN** the warning text appears on stderr, not stdout

#### Scenario: subprocess can discard stderr safely

- **WHEN** a subprocess caller captures only stdout with `--json`
- **THEN** it receives a complete, parseable JSON envelope regardless of warning or debug output

### Requirement: Environment variable reference

The system SHALL honour the following environment variables. `DONT_DIR` (string): path
to the project directory; overrides the filesystem walk. `DONT_AUTHOR` (string): default
author string used when `--author` is not supplied; may be in any format but `<kind>:<id>`
is recommended. `NO_COLOR` (any non-empty string): disables ANSI colour codes in human
output; see `dont-cli-surface` for full colour-resolution rules. `CLICOLOR_FORCE=1`:
forces ANSI colour even when stdout is redirected; see `dont-cli-surface`. No other
`DONT_` variables are part of the stable contract; callers MUST NOT rely on
undocumented `DONT_` variables between releases.

#### Scenario: DONT_DIR is the documented project-root override

- **WHEN** a caller sets `DONT_DIR=/tmp/myproject/.dont`
- **THEN** `dont` uses that directory as the project root without walking the filesystem
- **AND** this behaviour is stable across releases

#### Scenario: DONT_AUTHOR sets the default event author

- **WHEN** a CI pipeline sets `DONT_AUTHOR=agent:ci-pipeline`
- **THEN** all events recorded during that session carry `agent:ci-pipeline` as the author unless `--author` overrides it

#### Scenario: undocumented DONT_ variables are not relied upon

- **WHEN** a caller script sets an undocumented variable such as `DONT_VERIFY_EVIDENCE_MOCK`
- **THEN** that variable is not guaranteed to exist or behave consistently in future releases
- **AND** the caller MUST NOT use it in production integrations

### Requirement: DONT_DIR isolation for test environments

The system SHALL support test isolation through `DONT_DIR`. A test harness MUST be able
to point multiple concurrent `dont` invocations at distinct temporary directories using
`DONT_DIR` without those invocations interfering with each other or with the host
project. Each isolated `DONT_DIR` path MUST be independently initialised with `dont init`
before any other command is issued against it.

#### Scenario: two concurrent test invocations with distinct DONT_DIR values

- **WHEN** two `dont` processes run concurrently with `DONT_DIR` pointing at different temporary directories
- **THEN** each process operates on its own isolated project state with no cross-contamination

#### Scenario: test isolation does not require a parent .dont directory

- **WHEN** `DONT_DIR` points at a path that is not named `.dont` (e.g. `/tmp/test-abc/`)
- **THEN** `dont` accepts that path as the project directory without requiring the name to be `.dont`
