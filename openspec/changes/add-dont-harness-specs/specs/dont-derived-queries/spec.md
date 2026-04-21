## ADDED Requirements

### Requirement: List and vocabulary query scopes
The system SHALL provide read-only collection queries for claims and terms. `dont list` MUST return claims by default, filtered by optional `--status` and `--as-of` arguments. `dont vocab` MUST return terms by default with the same filter semantics. `dont list --all` MUST include terms, and `dont vocab` MUST remain equivalent to `list --kind term`.

#### Scenario: list defaults to claims
- **WHEN** the caller runs `dont list --json`
- **THEN** the command returns `envelope_kind: "claims"`
- **AND** the payload contains claims rather than terms by default

#### Scenario: vocab narrows to terms
- **WHEN** the caller runs `dont vocab --status unverified --json`
- **THEN** the command returns only term entities matching that status filter

#### Scenario: as-of produces historical slice
- **WHEN** the caller supplies `--as-of <timestamp>` to `dont list` or `dont vocab`
- **THEN** the query evaluates entity state at that historical point rather than at the latest transaction

### Requirement: Entity inspection queries
The system SHALL provide `dont show <entity-id>` for the current view of one entity and `dont why <entity-id>` for the current view plus explanatory context. `dont show --history` MUST include the entity's event timeline. `dont why` MUST include the entity, its events, and all currently applicable rules with remediation for unmet conditions.

#### Scenario: show returns current entity view
- **WHEN** the caller runs `dont show claim:01HX05A9K8VP --json`
- **THEN** the command returns the current `ClaimView` or `TermView` for that entity
- **AND** the default response omits the full event timeline unless `--history` is set

#### Scenario: show history includes timeline
- **WHEN** the caller runs `dont show claim:01HX05A9K8VP --history --json`
- **THEN** the response includes the entity's event timeline in addition to the current view

#### Scenario: why explains current blockers
- **WHEN** the caller runs `dont why claim:01HX05A9K8VP --json`
- **THEN** the response includes the entity, its events, and the rules currently applicable to it
- **AND** unmet rules include remediation context the caller can act on

### Requirement: Session-orientation query
The system SHALL provide `dont prime` as the session-start orientation query. `dont prime --json` MUST return the `PrimeView` payload describing the current project mode, rule activation, status counts, harness mode, and blocking work relevant to the next session step.

#### Scenario: prime on fresh project
- **WHEN** the caller runs `dont prime --json` in a freshly initialised project
- **THEN** the response returns `envelope_kind: "prime"`
- **AND** the payload reports zero-state counts and an empty blocking set

#### Scenario: prime reports harness mode
- **WHEN** harness-mode detection selects harness or direct mode
- **THEN** `PrimeView.harness_mode` reflects that resolved mode so the caller can verify it at session start

### Requirement: Diagnostic and schema queries
The system SHALL provide read-only diagnostic and schema queries. `dont doctor` MUST report substrate reachability, rule compilation health, and freshness/availability checks for auxiliary surfaces. `dont schema <name>` MUST print the JSON Schema for the named envelope or payload type, and bare `dont schema` MUST list available schema names.

#### Scenario: doctor reports health checks
- **WHEN** the caller runs `dont doctor --json`
- **THEN** the command returns `envelope_kind: "doctor"`
- **AND** the payload contains the diagnostic checks relevant to store health, rules, and auxiliary tooling

#### Scenario: doctor strict mode escalates warnings
- **WHEN** the caller runs `dont doctor --strict --json`
- **THEN** warning or failing checks are treated according to the strict-exit semantics defined by `dont-cli-surface`

#### Scenario: schema without argument lists names
- **WHEN** the caller runs `dont schema --json`
- **THEN** the response lists schema names available for inspection rather than one schema document

#### Scenario: schema with name prints one schema
- **WHEN** the caller runs `dont schema claim --json`
- **THEN** the response returns the `SchemaDoc` for the named schema target

### Requirement: Canonical examples query
The system SHALL provide `dont examples` as a read-only query returning canonical worked examples that teach the intended workflow.

#### Scenario: examples returns worked-example set
- **WHEN** the caller runs `dont examples --json`
- **THEN** the command returns `envelope_kind: "examples"`
- **AND** the payload contains the canonical examples list described by `ExamplesList`
