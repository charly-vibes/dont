# dont-derived-queries Specification

## Purpose
TBD - created by archiving change add-dont-harness-specs. Update Purpose after archive.
## Requirements
### Requirement: List and vocabulary query scopes
The system SHALL provide read-only collection queries for claims and terms. `dont list` MUST return claims by default, filtered by optional `--status` and `--kind` arguments. `--status` SHALL match only persisted lifecycle status. `dont list --kind terms` MUST return term entities with the same filter semantics. `dont list --kind claims` is equivalent to bare `dont list`.

> **Not yet implemented**: `--derived-assessment` (filter by computed assessments such as `stale`), `--as-of` (historical slice query), `dont list --all` (include both claims and terms), and the `dont vocab` alias for `dont list --kind terms`. These are planned but not available in the current implementation. When implemented, `--derived-assessment` SHALL match computed assessments without treating them as persisted statuses; `--as-of` SHALL evaluate entity state at a historical transaction point; `dont vocab` SHALL remain equivalent to `dont list --kind terms`.

#### Scenario: list defaults to claims
- **WHEN** the caller runs `dont list --json`
- **THEN** the command returns `envelope_kind: "claims"`
- **AND** the payload contains claims rather than terms by default

#### Scenario: list --kind terms narrows to terms
- **WHEN** the caller runs `dont list --kind terms --status unverified --json`
- **THEN** the command returns `envelope_kind: "terms"` with only term entities matching that status filter

#### Scenario: why explains current blockers
- **WHEN** the caller runs `dont why claim:01HX05A9K8VP --json`
- **THEN** the response includes the entity, its events, and the rules currently applicable to it
- **AND** unmet rules include remediation context the caller can act on

### Requirement: Entity inspection queries
The system SHALL provide `dont show <entity-id>` for the current view of one entity and `dont why <entity-id>` for the current view plus explanatory context. `dont show` MUST include the entity's event timeline in the response. `dont why` MUST include the entity, its events, and all currently applicable rules with remediation for unmet conditions.

> **Note**: The `ClaimView` returned by `dont show` always includes an `events` array in its payload. There is no `--history` flag — the event timeline is unconditionally present. The spec originally described a `--history` opt-in flag; the actual design bundles events into every `ClaimView` response instead.

#### Scenario: show returns current entity view with events
- **WHEN** the caller runs `dont show claim:01HX05A9K8VP --json`
- **THEN** the command returns the current `ClaimView` or `TermView` for that entity
- **AND** the response includes the entity's `events` array unconditionally

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

The system SHALL provide diagnostic and schema queries. By default, these queries are read-only. `dont doctor` MUST report substrate reachability, rule compilation health, freshness/availability checks for auxiliary surfaces, and a managed-docs staleness check covering the root managed block(s) and `.dont/AGENTS.md` as defined in `dont-agent-help`. `dont doctor` MUST accept a `--fix` flag whose behaviour is defined by `dont-agent-help` (rewrite managed regions and the canonical file). Without `--fix`, `dont doctor` MUST remain read-only. `dont schema <name>` MUST print the JSON Schema for the named envelope or payload type, and bare `dont schema` MUST list available schema names.

#### Scenario: doctor reports health checks

- **WHEN** the caller runs `dont doctor --json`
- **THEN** the command returns `envelope_kind: "doctor"`
- **AND** the payload contains the diagnostic checks relevant to store health, rules, auxiliary tooling, and managed-docs staleness

#### Scenario: doctor strict mode escalates warnings

- **WHEN** the caller runs `dont doctor --strict --json`
- **THEN** warning or failing checks are treated according to the strict-exit semantics defined by `dont-cli-surface`

#### Scenario: doctor without fix is read-only

- **WHEN** the caller runs `dont doctor` or `dont doctor --json` without `--fix`
- **THEN** the command performs no writes to any file on disk

#### Scenario: doctor with fix performs managed-doc repair

- **WHEN** the caller runs `dont doctor --fix`
- **THEN** the command applies the repair behaviour defined by `dont-agent-help` for managed regions and `.dont/AGENTS.md`
- **AND** the doctor envelope subsequently reports the managed-docs check as `status: "pass"` when re-run on the same project state

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

### Requirement: Trace is part of the read-only diagnostic query surface
The system SHALL provide `dont trace <entity-id>` as a read-only diagnostic query for dependency/support blocker inspection. `trace` complements `show` and `why`: `show` returns the current entity view, `why` explains current rules and history, and `trace` explains the blocker paths currently relevant to derived assessments or verification gating.

#### Scenario: trace returns blocker-oriented diagnostics
- **WHEN** the caller runs `dont trace claim:01HX05A9K8VP --json`
- **THEN** the command returns a trace-oriented payload rather than a ClaimView or WhyView

#### Scenario: healthy entity trace succeeds with empty blockers
- **WHEN** the caller runs `dont trace <entity-id> --json` on a healthy entity with no current dependency/support fallout
- **THEN** the command succeeds and returns an empty blocker set
