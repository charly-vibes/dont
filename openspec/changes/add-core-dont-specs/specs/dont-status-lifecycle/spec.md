## ADDED Requirements

### Requirement: Shared status lattice
The system SHALL define a status lattice for claims and terms containing `unverified`, `verified`, `doubted`, `stale`, `locked`, and `ignored`.

#### Scenario: entity statuses are drawn from the shared lattice
- **WHEN** the specification describes claim or term state
- **THEN** that state is expressed using the shared status lattice

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

### Requirement: Doubt stales dependent non-terminal entities
The system SHALL mark dependent non-terminal entities as `stale` when one of their dependencies becomes `doubted`, and SHALL apply this rule across claim-to-claim, claim-to-term, and term-to-term dependency edges.

#### Scenario: claim becomes stale after claim dependency is doubted
- **WHEN** a claim depends on another claim
- **AND** the dependency transitions to `doubted`
- **THEN** the dependent claim transitions to `stale` unless it is already terminal

#### Scenario: claim becomes stale after term dependency is doubted
- **WHEN** a claim depends on a term
- **AND** the term transitions to `doubted`
- **THEN** the dependent claim transitions to `stale` unless it is already terminal

#### Scenario: term becomes stale after term dependency is doubted
- **WHEN** a term depends on another term
- **AND** the dependency transitions to `doubted`
- **THEN** the dependent term transitions to `stale` unless it is already terminal

### Requirement: Stale entities may recover or be manually reopened
The system SHALL allow a stale entity to return to its prior non-stale status automatically when the dependencies that caused the stale transition are no longer in the `doubted` state, and SHALL also allow manual reopening to `unverified`.

#### Scenario: stale entity auto-recovers after dependencies stop being doubted
- **WHEN** all dependencies responsible for a stale state are no longer `doubted`
- **THEN** the stale entity returns to its prior non-stale status automatically

#### Scenario: stale entity can be reopened for independent reconsideration
- **WHEN** an actor explicitly reopens a stale entity
- **THEN** the entity transitions to `unverified`

### Requirement: Status transitions record audit context
The system SHALL record author identity and timestamp for each status transition, and SHALL allow transition-specific context such as reasons or evidence references to be attached when applicable.

#### Scenario: status transition carries audit context
- **WHEN** any status transition occurs
- **THEN** the transition history includes who performed it and when it occurred

#### Scenario: trust transition records its reason
- **WHEN** an actor transitions an entity to `doubted`
- **THEN** the transition record can include the stated reason for doubt

#### Scenario: dismiss transition records evidence references
- **WHEN** an actor transitions an entity toward `verified` through dismissal
- **THEN** the transition record can include the evidence references used for that dismissal
