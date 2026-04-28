## ADDED Requirements

### Requirement: Shared persisted status lattice
The system SHALL define a single persisted status lattice for claims and terms containing `unverified`, `verified`, `doubted`, `locked`, and `ignored`.

#### Scenario: entity statuses are drawn from the shared lattice
- **WHEN** the specification describes the persisted state of a claim or term
- **THEN** that state is expressed using the shared persisted status lattice

### Requirement: Persisted status and derived assessment are distinct
The system SHALL distinguish persisted status from derived assessment, and SHALL treat dependency-trace findings such as compromised verification support as computed analysis rather than stored lifecycle state.

#### Scenario: derived trace finding does not replace persisted status
- **WHEN** dependency analysis finds that a claim or term has compromised support because an upstream dependency is doubted or unresolved
- **THEN** the entity keeps its persisted lifecycle status
- **AND** the compromised support finding is presented as separate computed analysis

### Requirement: Entry state is unverified
The system SHALL require both `conclude` and `define` to introduce new claims and terms in the `unverified` state.

#### Scenario: new claim enters as unverified
- **WHEN** an actor invokes `conclude` successfully
- **THEN** the new claim is introduced as `unverified`

#### Scenario: new term enters as unverified
- **WHEN** an actor invokes `define` successfully
- **THEN** the new term is introduced as `unverified`

### Requirement: Locked and ignored states are terminal
The system SHALL treat `locked` and `ignored` as terminal states that reject further normal state transitions.

#### Scenario: locked entity refuses later transitions
- **WHEN** an actor attempts a normal state-changing operation on a locked entity
- **THEN** the operation is refused

#### Scenario: ignored entity refuses later transitions
- **WHEN** an actor attempts a normal state-changing operation on an ignored entity
- **THEN** the operation is refused

### Requirement: Dependency fallout is computed on demand
The system SHALL compute dependency fallout on demand across claim-to-claim, claim-to-term, and term-to-term edges, and SHALL NOT persist automatic downstream status changes solely because a dependency became `doubted`.

#### Scenario: direct doubt does not cascade into persisted dependent status changes
- **WHEN** a dependency transitions to `doubted`
- **THEN** dependent entities do not receive automatic persisted lifecycle transitions solely from that dependency change

#### Scenario: verification support may be reported as compromised in trace output
- **WHEN** a verification or audit trace traverses a dependency that is `doubted`, unresolved, `ignored`, or `locked`
- **THEN** the trace may report compromised or constrained support
- **AND** it does so without rewriting the dependent entity's persisted status

### Requirement: Reopen applies to persisted lifecycle closure
The system SHALL treat `reopen` as operating on explicit persisted lifecycle states rather than as recovery from derived dependency fallout.

#### Scenario: reopened entity leaves a persisted terminal state
- **WHEN** an actor explicitly reopens an entity that is in a persisted terminal state governed by lifecycle rules
- **THEN** the entity transitions according to those lifecycle rules
- **AND** the reopen operation does not target a derived trace assessment

### Requirement: Status transitions record audit context
The system SHALL make status transitions auditable and SHALL allow transition-specific context such as reasons or evidence references to be attached when applicable.

#### Scenario: status transition carries audit context
- **WHEN** any persisted status transition occurs
- **THEN** the transition history records enough context for later audit according to the data-model specification

#### Scenario: trust transition records its reason
- **WHEN** an actor transitions an entity to `doubted`
- **THEN** the transition record can include the stated reason for doubt

#### Scenario: dismiss transition records evidence references
- **WHEN** an actor transitions an entity toward `verified` through dismissal
- **THEN** the transition record can include the evidence references used for that dismissal

### Requirement: Derived assessments may inform later command policy
The system SHALL allow derived dependency-trace assessments to inform later command warnings or refusals without treating those assessments as persisted status changes.

#### Scenario: command policy can consult computed trace results
- **WHEN** a later command evaluates whether to proceed with an operation on a claim or term
- **THEN** it may consult computed dependency-trace results
- **AND** any resulting warning or refusal does not itself rewrite persisted lifecycle state
