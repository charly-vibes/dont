# dont-core Specification

## Purpose
TBD - created by archiving change add-core-dont-specs. Update Purpose after archive.
## Requirements
### Requirement: Epistemic forcing-function purpose
The system SHALL define `dont` as a project-local command-line tool for managing epistemic entities and their statuses, whose purpose is to interrupt unsupported assertions and require grounding before claims or terms may become `verified`.

#### Scenario: tool purpose is framed around grounding before verification
- **WHEN** a project adopts `dont`
- **THEN** the tool is specified as enforcing doubt and grounding rather than task tracking, workflow orchestration, generic knowledge-base authoring, or acting as a truth oracle

### Requirement: Core entity scope
The system SHALL treat claims and coined terms as the only first-class core epistemic entities in this initial capability split.

#### Scenario: core entity set is limited to claims and terms
- **WHEN** the core specification defines the primary objects managed by `dont`
- **THEN** it names claims and terms as the first-class entities
- **AND** any additional entity kind requires a future spec change instead of being implicitly introduced

### Requirement: Tool independence
The system SHALL specify `dont`, `wai`, and `beads`/`bd` as independent tools that may share conventions but do not require shared code, shared configuration, or a shared runtime.

#### Scenario: companion tool roles remain distinct
- **WHEN** the project describes how `dont` interacts with companion tools
- **THEN** `wai` is treated as workflow context, `beads`/`bd` as memory or issue tracking, and `dont` as epistemic discipline

#### Scenario: dont can operate without companion-tool runtime coupling
- **WHEN** `dont` is executed in a project that follows the shared conventions
- **THEN** its specification does not require `wai` or `beads`/`bd` to be linked into the same runtime or configuration surface

#### Scenario: integrations cannot redefine core semantics
- **WHEN** a companion tool or harness integrates with `dont`
- **THEN** the integration may enrich workflow behavior
- **AND** it does not redefine the meaning of core entities, persisted statuses, or history semantics

### Requirement: Append-only event history
The system SHALL represent state changes as append-only events and SHALL forbid destructive deletion or silent overwrite as normal state-management mechanisms.

#### Scenario: retraction is recorded instead of deleting history
- **WHEN** a claim or term is challenged, superseded, or otherwise reconsidered
- **THEN** the specification records that as a new event in history instead of deleting the earlier assertion

### Requirement: Verification is justified status, not absolute truth
The system SHALL treat `verified` as a recorded status justified under the system's evidence and rule model rather than as a claim of metaphysical or objective truth.

#### Scenario: verified status remains epistemically modest
- **WHEN** the specification describes what it means for a claim or term to be `verified`
- **THEN** it defines verification in terms of recorded justification and evidence
- **AND** it does not equate `verified` with unquestionable truth

### Requirement: Core invariants constrain future capabilities
The system SHALL require later capabilities to compose with core invariants including standalone operation, append-only history, and explicit epistemic status rather than overriding them.

#### Scenario: later capability extends without breaking core guarantees
- **WHEN** a later capability adds commands, storage rules, or integration behavior
- **THEN** the added behavior preserves standalone operation, append-only history, and explicit epistemic status semantics
