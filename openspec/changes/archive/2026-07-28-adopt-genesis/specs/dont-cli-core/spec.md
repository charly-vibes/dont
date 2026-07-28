# dont-cli-core spec delta: feedback verb

## ADDED Requirements

### Requirement: feedback subcommand

dont SHALL provide a `feedback` subcommand that files a structured issue against dont's upstream repo via `gh`, wrapping `genesis::feedback` for the redactor, context-bundle, error-scratch, and `gh`-invocation machinery.

#### Scenario: agent files a bug with last error

- **WHEN** `dont feedback bug --from-last-error --yes` is run after a non-zero exit
- **THEN** dont SHALL read its own error scratch (`$XDG_CACHE_HOME/dont/errors.jsonl`)
- **AND** SHALL assemble and redact the body via `genesis::feedback`
- **AND** SHALL invoke `gh issue create` against dont's `Cargo.toml` `repository` with labels `agent-reported`, `bug`, `has-repro`.

#### Scenario: error with no self-healing fix

- **WHEN** dont exits non-zero and no `genesis::suggestions::Fix` is available
- **THEN** the error footer SHALL print `Feedback: dont feedback bug --from-last-error`.
