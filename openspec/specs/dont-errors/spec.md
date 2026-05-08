# dont-errors Specification

## Purpose
TBD - created by archiving change add-dont-envelope-specs. Update Purpose after archive.
## Requirements
### Requirement: Structured error envelope
The system SHALL represent errors using an `ErrorResult` payload with fields `code` (stable lowercase-kebab identifier), `message` (human-readable one-liner), `rule_name` (the specific §13 rule that refused, or `null` for verb-level validators), `spec_ref` (advisory pointer to the spec section — not stable across spec versions, MUST NOT be used for programmatic branching), `entity_id` (target of the refused operation, when applicable), `unmet_clauses` (structured list of failing conditions), and `remediation` (non-empty array of next-action pairs).

#### Scenario: error envelope contains structured fields
- **WHEN** a command is refused
- **THEN** the envelope has `ok: false`, `envelope_kind: "error"`, and `data` contains an `ErrorResult` with at minimum `code`, `message`, and `remediation`

#### Scenario: rule_name distinguishes rule refusals from verb-level validators
- **WHEN** an error is caused by a rule-system refusal
- **THEN** `rule_name` identifies the specific rule (e.g. `lockable`, `unresolved-terms`)
- **AND** `code` is `rule-not-met`

#### Scenario: verb-level validators have dedicated codes and null rule_name
- **WHEN** an error is caused by a verb-level validator (not a rule)
- **THEN** `code` is the validator's dedicated identifier (e.g. `no-evidence`, `reason-required`) and `rule_name` is `null`

### Requirement: Scope boundary for rule-not-met
The system SHALL use `rule-not-met` exclusively for refusals originating from the §13 rule system and SHALL NOT use it for verb-level validators. Verb-level validators (`no-evidence`, `reason-required`, `reason-not-hedge`, `atoms-incomplete`, `wrong-entity-kind`, `term-label-empty`, `term-shape-indefinite`, `term-shape-punctuated`, `term-compound-undeclared`, `term-label-sentence`) SHALL have dedicated error codes and SHALL set `rule_name` to `null`.

#### Scenario: rule-not-met is not used for verb-level validators
- **WHEN** a verb-level validator refuses an operation
- **THEN** the error code is the validator's own dedicated code, not `rule-not-met`

#### Scenario: rule-not-met always carries a rule_name
- **WHEN** an error has `code: "rule-not-met"`
- **THEN** `rule_name` is non-null and identifies the specific rule that refused

#### Scenario: define label validator error has null rule_name
- **WHEN** `dont define` is refused by a term-shape validator
- **THEN** `rule_name` is `null` and `code` is the validator's dedicated code (e.g. `term-shape-indefinite`)

### Requirement: Remediation invariant
Every error envelope SHALL contain a non-empty `remediation[]` array of `{command, description}` pairs. An error envelope with empty or absent `remediation[]` is non-conformant with this specification. This is Invariant 3.2.5 — it applies to every code path that constructs an `ErrorResult`, including `usage` and `internal` errors.

#### Scenario: every error carries at least one remediation
- **WHEN** any error envelope is produced
- **THEN** the `remediation` array contains at least one `{command, description}` entry

#### Scenario: schema enforces remediation invariant
- **WHEN** the `ErrorResult` JSON Schema is validated
- **THEN** it requires `remediation` with `minItems: 1`

#### Scenario: usage errors point to help
- **WHEN** an error has `code: "usage"`
- **THEN** `remediation` includes an entry whose `command` points to `dont help <cmd>` for the relevant command

#### Scenario: internal errors point to doctor and issue reporting
- **WHEN** an error has `code: "internal"`
- **THEN** `remediation` includes an entry pointing to `dont doctor` and an entry pointing to issue reporting

### Requirement: Remediation ordering
The system SHALL order `remediation[]` entries from most-specific to least-specific. The first entry (`remediation[0]`) MUST be the most directly actionable remediation for the specific refusal.

#### Scenario: first remediation is most actionable
- **WHEN** a harness reads `remediation[0]`
- **THEN** the entry is the most specific and directly actionable remediation for the refusal

### Requirement: Unmet clauses with fix snippets
The system SHALL include an `unmet_clauses[]` array on error envelopes listing each failing condition with a `clause` description and a `fix` snippet showing how to satisfy it.

#### Scenario: error with multiple failing conditions lists each clause
- **WHEN** a command is refused due to multiple failing conditions
- **THEN** `unmet_clauses` contains one entry per condition, each with `clause` and `fix` fields

### Requirement: Open error-code set
The system SHALL treat the error-code set as open, allowing new codes in minor envelope versions, and SHALL require that parsers have a default branch for unknown codes rather than failing closed.

#### Scenario: parser encounters unknown error code
- **WHEN** a parser receives an error with a code not in its known set
- **THEN** the parser handles it through a default branch rather than failing

#### Scenario: new error codes do not require a major version bump
- **WHEN** a new error code is added
- **THEN** it is introduced in a minor envelope version without breaking existing parsers

### Requirement: Known error codes for envelope version 0.2
The system SHALL define the following error codes for envelope version 0.2. Refusal codes (exit 1, `ok: false`): `no-evidence`, `reason-required`, `reason-not-hedge`, `atoms-incomplete`, `claim-not-found`, `term-not-found`, `claim-locked`, `term-locked`, `claim-ignored`, `term-ignored`, `claim-not-verified`, `term-not-verified`, `rule-not-met`, `wrong-entity-kind`, `already-initialised`, `unresolvable-uri`, `schema-mismatch`, `db-locked`, `config-missing`, `spawn-not-found`, `spawn-expired`, `linkml-unsupported-feature`, `usage`, `internal`, `term-label-empty`, `term-shape-indefinite`, `term-shape-punctuated`, `term-compound-undeclared`, `term-label-sentence`. Warning codes (attached to `warnings[]` on `ok: true` envelopes), grouped by origin — verb-level: `evidence-malformed`, `evidence-stale`, `term-doc-shape-indefinite`, `term-doc-shape-punctuated`, `term-doc-shape-sentence`; rule-layer: `term-nonfunctional-label`, `rule-claim-structure`.

`rule-claim-structure` is a warn-severity rule-layer code attached to `warnings[]` on `ok: true` envelopes. It identifies tagged rule claims that are missing one or both mandatory slot markers (`[TRIGGER]` and one of `[CONFIG]` or `[MODE]`). The `rule_name` field SHALL be `"rule-claim-structure"` and the `entity_id` SHALL be the `claim:uuid` of the violating claim.

#### Scenario: domain refusal uses a known error code
- **WHEN** a domain rule or verb-level validator refuses an operation
- **THEN** the error `code` is one of the known domain error codes

#### Scenario: term-shape refusal exits code 1
- **WHEN** `dont define` is refused by a term-shape validator such as `term-shape-indefinite`
- **THEN** the process exits with code `1` and the envelope has `ok: false`, `envelope_kind: "error"`

#### Scenario: doc-extraction warning appears on success envelope
- **WHEN** an actor invokes `define` without `--label` and the leading phrase of `--doc` fails a shape check
- **THEN** the envelope has `ok: true` and `warnings[]` contains an entry with the appropriate `term-doc-shape-*` code

#### Scenario: usage errors use the usage code with envelope fields
- **WHEN** a command receives malformed arguments or an unknown flag
- **THEN** the error `code` is `usage` and the envelope has `ok: false`, `envelope_kind: "error"`

#### Scenario: internal errors use the internal code
- **WHEN** an unexpected tool failure occurs
- **THEN** the error `code` is `internal` and `remediation` points at `dont doctor` and at issue reporting

#### Scenario: warning codes appear on success envelopes
- **WHEN** an evidence URI is malformed but non-blocking
- **THEN** the envelope has `ok: true` and `warnings[]` contains an entry with the `evidence-malformed` warning code

#### Scenario: rule-claim-structure warning appears on success envelope
- **WHEN** `rule-claim-structure` is enabled and `dont prime` evaluates a tagged rule claim missing a mandatory slot marker
- **THEN** the envelope has `ok: true` and `warnings[]` contains an entry with `code: "rule-not-met"`, `rule_name: "rule-claim-structure"`, and `entity_id` identifying the violating claim

#### Scenario: rule-claim-structure warning does not appear when rule is disabled
- **WHEN** `rule-claim-structure` is not enabled in project config
- **THEN** no `rule-claim-structure` warning code appears on any envelope regardless of claim content

### Requirement: Exit-code contract
The system SHALL define a fixed set of exit codes that harnesses can branch on without parsing the envelope: `0` for success, `1` for refusal (LLM can retry via `remediation[]`), `2` for usage error, `3` for substrate/configuration error (requires operator action), `4` for internal error, `130` for SIGINT, `143` for SIGTERM.

#### Scenario: refusal exits with code 1
- **WHEN** a rule refuses or a verb-level validator trips
- **THEN** the process exits with code `1` and the envelope has `ok: false`, `envelope_kind: "error"`

#### Scenario: usage error exits with code 2
- **WHEN** a command receives malformed arguments
- **THEN** the process exits with code `2` and the envelope has `ok: false`, `envelope_kind: "error"`, `code: "usage"`

#### Scenario: substrate error exits with code 3
- **WHEN** the store is unreachable or configuration is incomplete
- **THEN** the process exits with code `3`

#### Scenario: internal error exits with code 4
- **WHEN** an unexpected bug occurs
- **THEN** the process exits with code `4`

### Requirement: Exit-code boundary semantics
The system SHALL define the boundary between exit `1` (refusal) and exit `3` (substrate) as the harness decision pivot: exit `1` means the LLM can make progress by reading `remediation[]`; exit `3` means the LLM should stop and the operator should check configuration or run `dont doctor`.

#### Scenario: harness distinguishes retryable from non-retryable errors
- **WHEN** a harness receives exit code `1`
- **THEN** it reads `remediation[0].command` to determine the next action

#### Scenario: harness escalates substrate errors to operator
- **WHEN** a harness receives exit code `3`
- **THEN** it stops LLM-driven retry and surfaces the error for operator intervention

### Requirement: Exit code 1 consistency under doctor strict mode
Consistent with the exit-code contract above, the system SHALL exit `1` from `dont doctor --strict` when any check is `warn` or `fail`, preserving the convention that exit `1` means "something the caller should attend to; envelope says what." Without `--strict`, exit `1` occurs only on `fail` checks.

#### Scenario: doctor strict exits 1 on warnings
- **WHEN** `dont doctor --strict` finds a `warn` check
- **THEN** the process exits with code `1`

#### Scenario: doctor non-strict exits 0 on warnings
- **WHEN** `dont doctor` (without `--strict`) finds only `warn` checks and no `fail` checks
- **THEN** the process exits with code `0`
