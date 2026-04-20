## ADDED Requirements

### Requirement: Four primary epistemic verbs
The system SHALL define `conclude`, `define`, `trust`, and `dismiss` as the primary CLI verbs for driving claims and terms through the epistemic workflow.

#### Scenario: core CLI surface is limited to four primary verbs
- **WHEN** the specification describes the core CLI
- **THEN** it presents `conclude`, `define`, `trust`, and `dismiss` as the primary epistemic verbs

### Requirement: Conclude introduces claims
The system SHALL let `conclude` introduce a claim with statement content and may allow structured metadata such as atoms, references, confidence, dependencies, session identity, and author identity.

#### Scenario: conclude creates an unverified claim
- **WHEN** an actor invokes `conclude` with a statement
- **THEN** the tool creates a claim entity
- **AND** the entity enters the `unverified` state as defined by the lifecycle specification

#### Scenario: conclude may accept unresolved claim references in permissive mode
- **WHEN** the project operates in a permissive mode that allows unresolved claim references at creation time
- **THEN** `conclude` may still create the `unverified` claim while recording that verification is blocked until those references are resolved

#### Scenario: conclude may refuse unresolved claim references in strict mode
- **WHEN** the project operates in a strict mode that forbids unresolved claim references at creation time
- **THEN** `conclude` is refused until those references are resolved

### Requirement: Define introduces coined terms
The system SHALL let `define` introduce a project term with a CURIE and prose definition.

#### Scenario: define creates an unverified term
- **WHEN** an actor invokes `define` with a CURIE and definition
- **THEN** the tool creates a term entity
- **AND** the entity enters the `unverified` state as defined by the lifecycle specification

#### Scenario: define refuses unresolved referenced terms
- **WHEN** an actor invokes `define` with parent, related, or attribute references that do not resolve
- **THEN** the command is refused

#### Scenario: define may append a redefinition for an existing coined CURIE
- **WHEN** an actor invokes `define` for a CURIE that already exists as a coined project term
- **THEN** the command may append a new definition event for that CURIE instead of creating an unrelated duplicate term

### Requirement: Trust records explicit doubt
The system SHALL let `trust` move a non-terminal claim or term into the `doubted` state and SHALL require a concrete reason for doing so.

#### Scenario: trust requires a reason and produces doubt
- **WHEN** an actor invokes `trust` on a non-terminal entity with a concrete reason
- **THEN** the entity transitions to `doubted`
- **AND** the reason is recorded in history

#### Scenario: trust refuses terminal entities
- **WHEN** an actor invokes `trust` on a locked or ignored entity
- **THEN** the command is refused

### Requirement: Dismiss verifies through evidence
The system SHALL let `dismiss` clear doubt for a claim or term using one or more evidence references, subject to deterministic local refusal conditions.

#### Scenario: dismiss verifies an eligible claim or term with evidence
- **WHEN** an actor invokes `dismiss` on an eligible claim or term with one or more evidence references
- **THEN** the command is allowed to transition the target toward `verified` according to the lifecycle rules

#### Scenario: dismiss refuses when evidence is absent
- **WHEN** an actor invokes `dismiss` without any evidence references
- **THEN** the command is refused

#### Scenario: dismiss refuses terminal entities
- **WHEN** an actor invokes `dismiss` on a locked or ignored entity
- **THEN** the command is refused

#### Scenario: dismiss refuses malformed evidence references
- **WHEN** an actor invokes `dismiss` with evidence references that are malformed URIs or unresolved CURIE prefixes
- **THEN** the command is refused

#### Scenario: dismiss refuses when referenced terms remain unresolved
- **WHEN** an actor invokes `dismiss` for a claim or term whose required referenced terms do not resolve
- **THEN** the command is refused

#### Scenario: dismiss may require atom completion before claim verification
- **WHEN** a claim has declared atoms and at least one atom remains unverified or doubted
- **THEN** whole-claim dismissal may be refused until the atom-level verification requirements are satisfied
