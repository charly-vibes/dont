# dont-status-lifecycle Specification

## Purpose
TBD - created by archiving change add-core-dont-specs. Update Purpose after archive.
## Requirements
### Requirement: Shared persisted status lattice
The system SHALL define a single persisted status lattice for claims and terms containing `unverified`, `verified`, `doubted`, `locked`, and `ignored`. In `dont`, `status` is what users explicitly record through commands; `derived_assessments` are what the system automatically infers from dependency graph integrity. The system SHALL NOT mix derived assessments into the persisted lattice.

#### Scenario: entity statuses are drawn from the shared lattice
- **WHEN** the specification describes the persisted state of a claim or term
- **THEN** that state is expressed using the shared persisted status lattice

### Requirement: Persisted status and derived assessment are distinct
The system SHALL distinguish persisted status from derived assessment. A derived assessment is a read-time annotation computed on demand from an entity's persisted status, dependency graph, imported references, and evidence liveness records. Derived assessments SHALL NOT be stored as lifecycle state; they SHALL be recomputed for read/query output and for command policy checks that consult dependency integrity.

The v0.3 derived assessment names are:
- `stale`: at least one transitive support dependency has persisted status `doubted` or itself carries `stale` during the current trace.
- `compromised-support`: at least one transitive support dependency is `ignored` or `locked`, making the dependent entity's verification support constrained without changing its status.
- `dangling-dependency`: a persisted dependency edge points to an entity or imported term that no longer resolves.
- `unresolved-term`: a claim or term references a CURIE that resolves neither to a coined term nor to an imported term.

Each derived assessment SHALL clear automatically from later read/query output when its trigger condition is no longer true. For example, `stale` clears when all transitive support dependencies are no longer `doubted`; `compromised-support` clears when constrained dependencies stop being part of the support trace or leave constraining statuses; `dangling-dependency` clears when the referenced dependency resolves again or the dependency edge is removed; and `unresolved-term` clears when the CURIE resolves or the reference is removed.

#### Scenario: derived trace finding does not replace persisted status
- **WHEN** dependency analysis finds that a verified claim has a transitive dependency in `doubted` status
- **THEN** the claim keeps its persisted `verified` status
- **AND** the output presents `stale` in `derived_assessments[]`

#### Scenario: derived assessments are recomputed after status changes
- **WHEN** any persisted status transition occurs on an entity in the dependency graph
- **THEN** later read/query operations recompute derived assessments for affected entities rather than reading stored assessment state

#### Scenario: ignored entities are excluded from derived assessment output
- **WHEN** an entity has persisted status `ignored`
- **THEN** read/query output for that entity has an empty `derived_assessments[]` unless a future spec explicitly opts into ignored-entity diagnostics

### Requirement: Entry state is unverified
The system SHALL require both `conclude` and `define` to introduce new claims and terms in the `unverified` state.

#### Scenario: new claim enters as unverified
- **WHEN** an actor invokes `conclude` successfully
- **THEN** the new claim is introduced as `unverified`

#### Scenario: new term enters as unverified
- **WHEN** an actor invokes `define` successfully
- **THEN** the new term is introduced as `unverified`

### Requirement: Locked and ignored states reject normal transitions
The system SHALL treat `locked` and `ignored` as closure states that reject further normal state transitions. `locked` is not reopenable in v0.3. `ignored` rejects normal transitions but MAY be restored by the explicit `reopen` lifecycle operation defined in `dont-lifecycle-verbs`.

#### Scenario: locked entity refuses later transitions
- **WHEN** an actor attempts a normal state-changing operation on a locked entity
- **THEN** the operation is refused

#### Scenario: ignored entity refuses later transitions
- **WHEN** an actor attempts a normal state-changing operation on an ignored entity
- **THEN** the operation is refused

### Requirement: Dependency fallout is computed on demand
The system SHALL compute dependency fallout on demand across claim-to-claim, claim-to-term, and term-to-term edges, and SHALL NOT persist automatic downstream status changes solely because a dependency became `doubted`. Dependency traversal SHALL be cycle-safe: a trace SHALL keep a visited set for the current traversal, SHALL terminate when it revisits an entity, and SHALL NOT treat the existence of a cycle by itself as a `stale` assessment.

#### Scenario: direct doubt does not cascade into persisted dependent status changes
- **WHEN** a dependency transitions to `doubted`
- **THEN** dependent entities do not receive automatic persisted lifecycle transitions solely from that dependency change

#### Scenario: verification support may be reported as compromised in trace output
- **WHEN** a verification or audit trace traverses a dependency that is `doubted`, unresolved, `ignored`, or `locked`
- **THEN** the trace may report compromised or constrained support
- **AND** it does so without rewriting the dependent entity's persisted status

### Requirement: Reopen applies to ignored persisted lifecycle closure
The system SHALL treat `reopen` as operating only on the explicit persisted `ignored` lifecycle state in v0.3. `reopen` SHALL NOT target `locked` entities and SHALL NOT target derived assessments such as `stale`.

#### Scenario: reopened ignored entity leaves persisted closure
- **WHEN** an actor explicitly reopens an entity that has persisted status `ignored`
- **THEN** the entity transitions to `unverified`
- **AND** derived assessments for affected dependency traces are recomputed on later reads

#### Scenario: reopen does not target derived stale assessment
- **WHEN** an actor invokes `reopen` on an entity whose persisted status is `verified` and whose `derived_assessments[]` contains `stale`
- **THEN** the command is refused because `stale` is not a persisted lifecycle state

#### Scenario: reopen refuses locked entities
- **WHEN** an actor invokes `reopen` on an entity whose persisted status is `locked`
- **THEN** the command is refused because locked entities are not reopenable in v0.3

### Requirement: Status transitions record audit context
The system SHALL make status transitions auditable and SHALL allow transition-specific context such as reasons or evidence references to be attached when applicable.

#### Scenario: status transition carries audit context
- **WHEN** any persisted status transition occurs
- **THEN** the transition history records enough context for later audit according to the data-model specification

#### Scenario: trust transition records its reason
- **WHEN** an actor transitions an entity to `doubted`
- **THEN** the transition record can include the stated reason for doubt

#### Scenario: flag transition records evidence references
- **WHEN** an actor transitions an entity toward `verified` through flagal
- **THEN** the transition record can include the evidence references used for that flagal

### Requirement: Derived assessments may inform later command policy
The system SHALL allow derived dependency-trace assessments to inform later command warnings or refusals without treating those assessments as persisted status changes.

#### Scenario: command policy can consult computed trace results
- **WHEN** a later command evaluates whether to proceed with an operation on a claim or term
- **THEN** it may consult computed dependency-trace results
- **AND** any resulting warning or refusal does not itself rewrite persisted lifecycle state
