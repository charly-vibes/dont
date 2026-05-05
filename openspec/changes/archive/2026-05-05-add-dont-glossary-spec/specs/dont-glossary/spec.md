## ADDED Requirements

### Requirement: Epistemic entity
The glossary SHALL define "epistemic entity" as the umbrella concept representing any distinct assertion or definition whose status is managed by the system's shared epistemic lattice. The glossary MUST state that claims and terms are the concrete instantiations of epistemic entities, and it MUST NOT specify their structural or behavioral rules, instead pointing to `dont-data-model` and `dont-status-lifecycle`.

#### Scenario: resolving epistemic entity meaning
- **WHEN** a reader looks up "epistemic entity"
- **THEN** they learn it is an umbrella concept covering both claims and terms

### Requirement: Claim
The glossary SHALL define "claim" as a declarative assertion managed by `dont`. It MUST point to `dont-data-model` for its structure and `dont-cli-core` for its creation via `conclude`.

#### Scenario: resolving claim meaning
- **WHEN** a reader looks up "claim"
- **THEN** they learn it is a declarative assertion that instantiates an epistemic entity

### Requirement: Term
The glossary SHALL define "term" as an LLM-coined or imported project vocabulary entry managed by `dont`. It MUST point to `dont-data-model` for its structure and `dont-cli-core` for its creation via `define`.

#### Scenario: resolving term meaning
- **WHEN** a reader looks up "term"
- **THEN** they learn it is a vocabulary entry that instantiates an epistemic entity

### Requirement: Epistemic lattice
The glossary SHALL define "epistemic lattice" as the shared status model governing claims and terms. It SHALL explicitly list "status lattice" as an alias that maps identically to "epistemic lattice". It MUST point to `dont-status-lifecycle` for the actual transition semantics.

#### Scenario: looking up status lattice alias
- **WHEN** a reader looks up "status lattice"
- **THEN** they are directed to the canonical concept "epistemic lattice"

### Requirement: Atom
The glossary SHALL define "atom" as an independently checkable sub-statement of a claim. It MUST point to `dont-data-model` for its structural representation and scope.

#### Scenario: resolving atom meaning
- **WHEN** a reader looks up "atom"
- **THEN** they learn it is a sub-statement of a claim

### Requirement: Atom-completion gate
The glossary SHALL define "atom-completion gate" as the rule that all atoms must verify before their parent claim becomes verified. It MUST point to `dont-data-model` for the enforcement of this invariant.

#### Scenario: resolving atom-completion gate meaning
- **WHEN** a reader looks up "atom-completion gate"
- **THEN** they learn it is the rule tying atom verification to parent claim verification

### Requirement: Core four verbs
The glossary SHALL define "core four verbs" as the collective grouping of the four primary CLI verbs: `conclude`, `define`, `trust`, and `dismiss`. It MUST point to `dont-cli-core` for their contracts.

#### Scenario: resolving core four verbs meaning
- **WHEN** a reader looks up "core four verbs"
- **THEN** they learn it refers specifically to conclude, define, trust, and dismiss

### Requirement: Lifecycle verb
The glossary SHALL define "lifecycle verb" as the collective grouping of secondary CLI verbs that manage entity states outside the core four: `lock`, `reopen`, `ignore`, and `verify-evidence`.

#### Scenario: resolving lifecycle verb meaning
- **WHEN** a reader looks up "lifecycle verb"
- **THEN** they learn it refers to lock, reopen, ignore, and verify-evidence

### Requirement: Evidence
The glossary SHALL define "evidence" as the grounding material cited during verification workflows. It MUST point to `dont-data-model` for its structural representation as a core relation.

#### Scenario: resolving evidence meaning
- **WHEN** a reader looks up "evidence"
- **THEN** they learn it is the grounding material used in verification

### Requirement: Hedge pattern
The glossary SHALL define "hedge pattern" as a configured vague-reason pattern that fails reason quality checks during explicit doubt. It MUST point to future configuration and CLI specs for enforcement rules.

#### Scenario: resolving hedge pattern meaning
- **WHEN** a reader looks up "hedge pattern"
- **THEN** they learn it is a vague reason that is actively rejected by the system

### Requirement: Rule
The glossary SHALL define "rule" broadly as a named policy or predicate used to warn or refuse commands. It MUST point to the future `dont-rules` capability for the engine implementation.

#### Scenario: resolving rule meaning
- **WHEN** a reader looks up "rule"
- **THEN** they learn it is a named policy or predicate

### Requirement: Author string
The glossary SHALL define "author string" as the identity format used for events, represented as `<actor-kind>:<id>`. It MUST point to `dont-data-model` for its structural validation and constraints.

#### Scenario: resolving author string meaning
- **WHEN** a reader looks up "author string"
- **THEN** they learn it represents the `<actor-kind>:<id>` format

### Requirement: Seed vocabulary
The glossary SHALL define "seed vocabulary" as the bootstrap `dont:` vocabulary installed at project initialization time. It MUST point to the future `dont-init-modes` capability for bootstrap logic.

#### Scenario: resolving seed vocabulary meaning
- **WHEN** a reader looks up "seed vocabulary"
- **THEN** they learn it is the initial vocabulary installed when a project begins

### Requirement: Event
The glossary SHALL define "event" as an immutable history record representing an action or state change in the system. It MUST clarify that events are distinct from event kinds.

#### Scenario: resolving event meaning
- **WHEN** a reader looks up "event"
- **THEN** they learn it is an immutable history record

### Requirement: Event kind
The glossary SHALL define "event kind" as the classifier identifying the type of an event. It MUST point to `dont-data-model` for the canonical list of closed event kinds.

#### Scenario: resolving event kind meaning
- **WHEN** a reader looks up "event kind"
- **THEN** they learn it classifies an event, distinct from the event itself

### Requirement: Remediation
The glossary SHALL define "remediation" as actionable recovery guidance returned in error payloads. It MUST point to the future `dont-envelope` capability for the payload schema.

#### Scenario: resolving remediation meaning
- **WHEN** a reader looks up "remediation"
- **THEN** they learn it provides actionable recovery guidance
