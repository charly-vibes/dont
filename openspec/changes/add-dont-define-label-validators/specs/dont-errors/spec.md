## MODIFIED Requirements

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

### Requirement: Known error codes for envelope version 0.2
The system SHALL define the following error codes for envelope version 0.2. Refusal codes (exit 1, `ok: false`): `no-evidence`, `reason-required`, `reason-not-hedge`, `atoms-incomplete`, `claim-not-found`, `term-not-found`, `claim-locked`, `term-locked`, `claim-ignored`, `term-ignored`, `claim-not-verified`, `term-not-verified`, `claim-not-stale`, `term-not-stale`, `rule-not-met`, `wrong-entity-kind`, `already-initialised`, `unresolvable-uri`, `schema-mismatch`, `db-locked`, `config-missing`, `spawn-not-found`, `spawn-expired`, `linkml-unsupported-feature`, `usage`, `internal`, `term-label-empty`, `term-shape-indefinite`, `term-shape-punctuated`, `term-compound-undeclared`, `term-label-sentence`. Warning codes (attached to `warnings[]` on `ok: true` envelopes), grouped by origin — verb-level: `evidence-malformed`, `evidence-stale`, `term-doc-shape-indefinite`, `term-doc-shape-punctuated`, `term-doc-shape-sentence`; rule-layer: `term-nonfunctional-label`.

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
