## ADDED Requirements

### Requirement: Atom
The glossary SHALL define an **Atom** as an independently checkable sub-statement of a claim. Atom-level behavior and transitions are owned by the data-model and CLI capabilities rather than by the glossary.

#### Scenario: Reader looks up atom
- **WHEN** a reader encounters `atom` in a claim-verification flow
- **THEN** the glossary explains that it names a sub-statement rather than a full claim

### Requirement: Atom-completion gate
The glossary SHALL define the **Atom-completion gate** as the rule that a claim with declared atoms becomes `verified` only when every declared atom becomes `verified`. Detailed refusal and transition semantics are owned by the data-model and CLI capabilities.

#### Scenario: Reader looks up atom-completion gate
- **WHEN** a reader encounters `atoms-incomplete` or atom-specific dismissal behavior
- **THEN** the glossary explains that whole-claim verification is gated by full atom verification

### Requirement: Claim
The glossary SHALL define a **Claim** as a declarative assertion tracked by `dont` within the epistemic workflow. Claim lifecycle behavior is owned by the status-lifecycle and CLI capabilities.

#### Scenario: Reader looks up claim
- **WHEN** a reader sees `claim` in a command contract
- **THEN** the glossary explains that the term refers to an assertion managed by the store and lattice

### Requirement: Term
The glossary SHALL define a **Term** as an LLM-coined vocabulary entry used within a project and tracked as a first-class entity in the epistemic workflow. Definition, redefinition, and validation behavior are owned by adjacent capabilities.

#### Scenario: Reader looks up term
- **WHEN** a reader encounters `term` in `define` or stale-cascade behavior
- **THEN** the glossary explains that the word names project vocabulary rather than an ordinary claim

### Requirement: Epistemic lattice
The glossary SHALL define the **Epistemic lattice** as the canonical name for the status system governing claims and terms in `dont`.

#### Scenario: Reader looks up epistemic lattice
- **WHEN** a reader encounters a discussion of allowed status transitions
- **THEN** the glossary explains that the phrase refers to the shared status model for claims and terms

### Requirement: Status lattice alias
The glossary SHALL record **Status lattice** as an alias of **Epistemic lattice**.

#### Scenario: Reader encounters alternate naming
- **WHEN** one artifact says `status lattice` and another says `epistemic lattice`
- **THEN** the glossary makes clear that both refer to the same canonical concept

### Requirement: Core four verbs
The glossary SHALL define the **Core four verbs** as `conclude`, `define`, `trust`, and `dismiss`, which are the primary verbs that move entities through the epistemic workflow.

#### Scenario: Reader looks up core four verbs
- **WHEN** a reader encounters the phrase `core four verbs`
- **THEN** the glossary enumerates the four primary verbs and distinguishes them from lifecycle verbs

### Requirement: Lifecycle verb
The glossary SHALL define a **Lifecycle verb** as a verb that operates around the core lattice-managed workflow rather than serving as one of the core four verbs. The canonical lifecycle verbs in the v0.3 line are `lock`, `reopen`, `ignore`, and `verify-evidence`.

#### Scenario: Reader distinguishes verb categories
- **WHEN** a reader compares `dismiss` with `lock`
- **THEN** the glossary explains that one is a core verb and the other is a lifecycle verb

### Requirement: Forcing function
The glossary SHALL define **Forcing function** as the project’s design stance of blocking ungrounded assertions until the actor takes the next required grounding step.

#### Scenario: Reader looks up forcing function
- **WHEN** a reader encounters the phrase `forcing function`
- **THEN** the glossary explains that it names the design intent behind `dont`, not a separate command

### Requirement: CURIE
The glossary SHALL define a **CURIE** as a compact URI in `prefix:local` form used to identify terms and external references concisely.

#### Scenario: Reader looks up CURIE
- **WHEN** a reader encounters `gr:RicciTensor`
- **THEN** the glossary explains that the identifier is a CURIE rather than a free-form string

### Requirement: Evidence
The glossary SHALL define **Evidence** as the URI, CURIE, or comparable reference material cited to ground a claim or term during verification-oriented workflows. Liveness checking behavior is owned by adjacent capabilities.

#### Scenario: Reader looks up evidence
- **WHEN** a reader encounters `--evidence` in a command contract
- **THEN** the glossary explains that the argument names grounding material rather than a status change by itself

### Requirement: Hedge pattern
The glossary SHALL define a **Hedge pattern** as a configured phrase pattern that is too vague to satisfy the non-hedged reason requirement for `trust`.

#### Scenario: Reader looks up hedge pattern
- **WHEN** a reader sees a refusal related to a weak `trust --reason`
- **THEN** the glossary explains that hedge-pattern matching rejects generic uncertainty without a concrete defect

### Requirement: Stale
The glossary SHALL define **Stale** as the transient status used when an entity’s dependency has been doubted and the entity’s grounding is therefore no longer current.

#### Scenario: Reader looks up stale
- **WHEN** a reader encounters stale-cascade behavior
- **THEN** the glossary explains that `stale` reflects dependency drift rather than direct refutation

### Requirement: Doubted
The glossary SHALL define **Doubted** as the status applied when an entity is actively challenged through the doubt workflow.

#### Scenario: Reader looks up doubted
- **WHEN** a reader encounters `trust` semantics
- **THEN** the glossary explains that the resulting status is `doubted`

### Requirement: Verified
The glossary SHALL define **Verified** as the status used for an entity whose required grounding conditions have been satisfied.

#### Scenario: Reader looks up verified
- **WHEN** a reader encounters dismissal or grounding success
- **THEN** the glossary explains that `verified` names the earned grounded state

### Requirement: Locked
The glossary SHALL define **Locked** as a terminal status for a verified claim that has passed the additional locking gate. The detailed locking rule is owned by the lifecycle and rules capabilities.

#### Scenario: Reader looks up locked
- **WHEN** a reader encounters `lock`
- **THEN** the glossary explains that `locked` is terminal and stricter than merely `verified`

### Requirement: Ignored
The glossary SHALL define **Ignored** as a terminal status used for entities intentionally removed from the active verification workflow without deleting their history.

#### Scenario: Reader looks up ignored
- **WHEN** a reader encounters `ignore`
- **THEN** the glossary explains that the entity leaves active epistemic play but remains in the audit trail

### Requirement: Rule
The glossary SHALL define a **Rule** as a named predicate or policy used to evaluate conditions, emit warnings, or refuse operations in `dont`. Specific rule logic is owned by the rules capability.

#### Scenario: Reader looks up rule
- **WHEN** a reader encounters a rule-related refusal
- **THEN** the glossary explains that a rule is the named policy being enforced

### Requirement: Lockable rule
The glossary SHALL define the **Lockable rule** as the named gate that determines whether a verified claim is eligible to transition to `locked`.

#### Scenario: Reader looks up lockable rule
- **WHEN** a reader sees `lockable` in locking guidance
- **THEN** the glossary explains that it names the eligibility rule for entering `locked`

### Requirement: Author string
The glossary SHALL define an **Author string** as the structured identity string recorded on events in `<actor-kind>:<id>` form.

#### Scenario: Reader looks up author string
- **WHEN** a reader encounters `llm:claude-opus-4.7`
- **THEN** the glossary explains that the value is an event author identifier with a stable shape

### Requirement: Seed vocabulary
The glossary SHALL define **Seed vocabulary** as the initial locked `dont:`-prefixed vocabulary installed when a project is initialized.

#### Scenario: Reader looks up seed vocabulary
- **WHEN** a reader encounters initialization behavior
- **THEN** the glossary explains that the seed vocabulary is the bootstrap ontology shipped with the tool

### Requirement: Event
The glossary SHALL define an **Event** as an immutable history record from which current views are derived.

#### Scenario: Reader looks up event
- **WHEN** a reader encounters append-only history language
- **THEN** the glossary explains that state is understood through recorded events rather than destructive updates

### Requirement: Event kind
The glossary SHALL define **Event kind** as the classifier that names what an event records. The canonical event-kind enumeration is owned by the data-model capability.

#### Scenario: Reader looks up event kind
- **WHEN** a reader sees `dismissed` or `mode_changed`
- **THEN** the glossary explains that these are event-kind values

### Requirement: Remediation
The glossary SHALL define **Remediation** as the actionable command-and-description guidance returned in an error response to help the actor recover.

#### Scenario: Reader looks up remediation
- **WHEN** a reader encounters an error envelope
- **THEN** the glossary explains that remediation entries are the suggested next steps
