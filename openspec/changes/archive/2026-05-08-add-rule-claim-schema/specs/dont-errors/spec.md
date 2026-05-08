## Dependencies

This spec delta requires `dont-rule-claim-schema` to be active. The `rule-claim-structure` rule referenced below is defined in `dont-rule-engine` (this change).

## MODIFIED Requirements

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
