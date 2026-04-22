## MODIFIED Requirements

### Requirement: Shipped rule catalogue
The system SHALL ship the following default rules with the documented semantics: `ungrounded`, `unresolved-terms`, `stale-cascade`, `lockable`, `correlated-error`, `dangling-definition`, and `term-nonfunctional-label`.

`term-nonfunctional-label` is an off-by-default warn-severity rule that flags terms whose label text suggests a non-functional relationship has been folded into the type. Matching is heuristic (configurable token patterns; see `dont-project-config`); the rule is disabled by default because false-positives are expected on valid noun phrases. Its purpose is to surface candidates for aspect-shaped redesign once aspects land as a primitive (currently deferred to §17). Every shipped rule, including `term-nonfunctional-label`, MUST have a sibling human-readable translation document.

#### Scenario: ungrounded flags or refuses unresolved CURIEs
- **WHEN** a claim references CURIEs that do not resolve
- **THEN** `ungrounded` either emits a warning or refuses the transition according to the project's mode and severity configuration

#### Scenario: unresolved-terms blocks dismiss
- **WHEN** the caller attempts `dont dismiss` on a claim whose CURIEs remain unresolved in coined or imported vocabulary
- **THEN** `unresolved-terms` refuses the dismissal

#### Scenario: stale-cascade propagates doubt
- **WHEN** a trust transition occurs on an entity
- **THEN** `stale-cascade` propagates `stale` across supported dependency edges
- **AND** locked and ignored entities are exempt from that cascade

#### Scenario: lockable gates lock
- **WHEN** the caller attempts `dont lock`
- **THEN** `lockable` checks for verified status, at least three assessed hypotheses, and at least two independent supporting evidence items before allowing the transition

#### Scenario: correlated-error flags shared-source evidence
- **WHEN** a claim's only evidence shares a source with its author
- **THEN** `correlated-error` emits a warning or strict refusal according to severity configuration

#### Scenario: dangling-definition blocks unresolved term relations
- **WHEN** `dont define` names `--kind-of` or `--related-to` references that do not resolve
- **THEN** `dangling-definition` refuses the definition in both permissive and strict modes

#### Scenario: term-nonfunctional-label emits warning when enabled
- **WHEN** `term-nonfunctional-label` is enabled and an actor defines a term whose label matches a configured non-functional-relationship pattern (e.g. `"a node that has a child"`)
- **THEN** the `define` command succeeds and the envelope carries a `term-nonfunctional-label` warning

#### Scenario: term-nonfunctional-label is disabled by default
- **WHEN** a project does not explicitly enable `term-nonfunctional-label` in its configuration
- **THEN** no `term-nonfunctional-label` warnings are emitted, regardless of label content

#### Scenario: term-nonfunctional-label has a translation sibling
- **WHEN** the system ships `term-nonfunctional-label`
- **THEN** the executable rule file is paired with a sibling English translation document for explanation and operator review
