# dont-lifecycle-verbs Deltas

## MODIFIED Requirements

### Requirement: trust registers explicit doubt
The system SHALL provide `trust` to register explicit doubt about a claim or term. Skepticism recorded via `trust` SHALL transition an entity from `unverified` or `verified` to `doubted`.

#### Scenario: trust registers doubt with reason
- **WHEN** `dont trust claim:X --reason "..."` is invoked
- **THEN** the claim transitions to `doubted` and the reason is recorded in the event log

### Requirement: flag (formerly dismiss) adds evidence
The system SHALL provide `flag` (formerly `dismiss`) to add evidence to an entity and transition it to `verified`. The `dismiss` verb remains supported as an alias for `flag` in v0.3 but is deprecated.

#### Scenario: flag transitions to verified with evidence
- **WHEN** `dont flag claim:X --evidence <uri>` is invoked
- **THEN** the claim transitions to `verified` and the evidence is recorded

#### Scenario: flag on doubted entity clears doubt
- **WHEN** `dont flag claim:X --evidence <uri>` is invoked on a doubted claim
- **THEN** the claim transitions to `verified` and doubt is cleared

#### Scenario: deprecated alias emits warning
- **WHEN** `dont dismiss` is invoked
- **THEN** a deprecation warning is emitted to stderr suggesting `flag` instead
- **AND** the command proceeds normally
- **AND** the deprecation warning goes to stderr regardless of whether `--json` is set

## ADDED Requirements

### Requirement: undoubt retracts explicit doubt
The system SHALL provide `undoubt` to retract explicit doubt on an entity, transitioning it from `doubted` back to `unverified`.

#### Scenario: undoubt retracts doubt
- **WHEN** `dont undoubt claim:X` is invoked on a doubted claim
- **THEN** the claim transitions to `unverified` and the retraction is recorded in the event log

#### Scenario: undoubt on non-doubted entity errors
- **WHEN** `dont undoubt claim:X` is invoked on a claim that is not doubted
- **THEN** the command exits non-zero with a `not-doubted` error
