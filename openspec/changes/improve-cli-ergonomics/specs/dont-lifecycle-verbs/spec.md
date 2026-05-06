# dont-lifecycle-verbs Deltas

## MODIFIED Requirements

### Requirement: doubt (formerly trust) registers explicit doubt
The system SHALL provide `doubt` (formerly `trust`) to register explicit doubt about a claim or term. Skepticism recorded via `doubt` SHALL transition an entity from `unverified` or `verified` to `doubted`. The `trust` verb remains supported as an alias for `doubt` in v0.3 but is deprecated.

#### Scenario: doubt registers doubt with reason
- **WHEN** `dont doubt claim:X --reason "..."` is invoked
- **THEN** the claim transitions to `doubted` and the reason is recorded in the event log

### Requirement: verify (formerly dismiss) adds evidence
The system SHALL provide `verify` (formerly `dismiss`) to add evidence to an entity and transition it to `verified`. The `dismiss` verb remains supported as an alias for `verify` in v0.3 but is deprecated.

#### Scenario: verify transitions to verified with evidence
- **WHEN** `dont verify claim:X --evidence <uri>` is invoked
- **THEN** the claim transitions to `verified` and the evidence is recorded

#### Scenario: deprecated aliases emit warning
- **WHEN** `dont trust` or `dont dismiss` is invoked in human mode
- **THEN** a deprecation warning is emitted to stderr suggesting the new verb
- **AND** the command proceeds normally
