## ADDED Requirements

### Requirement: Four primary epistemic verbs
The system SHALL define `conclude`, `define`, `trust`, and `dismiss` as the primary CLI verbs for driving claims and terms through the epistemic workflow.

#### Scenario: core CLI surface is limited to four primary verbs
- **WHEN** the specification describes the core CLI
- **THEN** it presents `conclude`, `define`, `trust`, and `dismiss` as the primary epistemic verbs

### Requirement: CLI verbs are interpreted in phrase form
The system SHALL define core CLI verb semantics in the phrase context `dont <verb>` rather than by the standalone English verb alone, and SHALL require command help and documentation to explain verbs in that phrase form.

#### Scenario: trust is read as do not trust
- **WHEN** the specification or help text explains `trust`
- **THEN** it explains `dont trust` as an instruction to register doubt rather than to endorse

#### Scenario: phrase form is required in command help
- **WHEN** the core CLI documents a primary verb
- **THEN** the documentation explains the meaning of the full phrase such as `dont dismiss` or `dont conclude`

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

#### Scenario: conclude refuses duplicate claim creation
- **WHEN** an actor invokes `conclude` for a claim that is equivalent to an existing claim under the project's deduplication rules
- **THEN** the command is refused instead of creating a second claim identity

### Requirement: Define introduces coined terms
The system SHALL let `define` introduce a project term with a CURIE and prose definition.

> **Deferred extension**: The `--label` flag and five SK11 shape validators (indefinite noun-phrase enforcement) are specified in the `add-dont-define-label-validators` change and are not part of this implementation slice. Implementations MUST NOT implement `--label` until that change is applied.

#### Scenario: define creates an unverified term
- **WHEN** an actor invokes `define` with a CURIE and definition
- **THEN** the tool creates a term entity
- **AND** the entity enters the `unverified` state as defined by the lifecycle specification

#### Scenario: define refuses unresolved referenced terms
- **WHEN** an actor invokes `define` with parent, related, or attribute references that do not resolve
- **THEN** the command is refused

#### Scenario: define refuses duplicate term creation
- **WHEN** an actor invokes `define` for a CURIE or canonically equivalent term that already exists under the project's deduplication rules
- **THEN** the command is refused as new term creation

#### Scenario: define may append a redefinition for an existing coined CURIE
- **WHEN** an actor invokes `define` for a CURIE that already exists as a coined project term
- **THEN** the command may append a new definition event for that existing term identity instead of creating an unrelated duplicate term

### Requirement: Trust records explicit doubt
The system SHALL let `trust` move a non-terminal claim or term into the `doubted` state and SHALL require a non-empty, substantive reason for doing so.

#### Scenario: trust requires a reason and produces doubt
- **WHEN** an actor invokes `trust` on a non-terminal entity with a substantive reason
- **THEN** the entity transitions to `doubted`
- **AND** the reason is recorded in history

#### Scenario: trust refuses hedge-style reasons
- **WHEN** an actor invokes `trust` with a vague hedge pattern or other low-information reason
- **THEN** the command is refused

#### Scenario: trust refuses terminal entities
- **WHEN** an actor invokes `trust` on a locked or ignored entity
- **THEN** the command is refused

### Requirement: Dismiss verifies through evidence
The system SHALL let `dismiss` serve as the core CLI path into `verified` status for a claim or term using one or more evidence references, subject to deterministic local refusal conditions.

#### Scenario: dismiss verifies an eligible claim or term with evidence
- **WHEN** an actor invokes `dismiss` on an eligible claim or term with one or more evidence references
- **THEN** the command is allowed to transition the target toward `verified` according to the lifecycle rules

#### Scenario: dismiss may append evidence to an already verified entity
- **WHEN** an actor invokes `dismiss` on an already `verified` claim or term with additional evidence references
- **THEN** the command may append evidence to that entity's history without creating a new identity or requiring a status change

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
