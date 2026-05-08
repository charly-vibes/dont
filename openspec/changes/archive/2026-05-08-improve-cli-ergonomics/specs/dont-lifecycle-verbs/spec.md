# dont-lifecycle-verbs Deltas

## ADDED Requirements

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

## MODIFIED Requirements

### Requirement: Undoubt retracts explicit doubt
The system SHALL provide an `undoubt` operation that moves a `doubted` claim or term back to `unverified`, allowing the doubt to be retracted when it was registered in error or superseded. `undoubt` SHALL only target entities in the `doubted` state and SHALL refuse all other persisted statuses. Use `reopen` for `ignored` entities — `undoubt` and `reopen` are distinct recovery operations for distinct closure states.

#### Scenario: undoubt moves doubted entity to unverified
- **WHEN** an actor invokes `undoubt` on an entity whose persisted status is `doubted`
- **THEN** the entity transitions to `unverified`

#### Scenario: undoubt refuses non-doubted entities
- **WHEN** an actor invokes `undoubt` on an entity whose persisted status is `unverified`, `verified`, `ignored`, or `locked`
- **THEN** the command is refused
