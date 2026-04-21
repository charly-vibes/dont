## ADDED Requirements

### Requirement: Single rule source format with sibling translation
The system SHALL represent project rules in a single executable rule format rather than multiple interchangeable rule syntaxes. In v0.3 this executable format MUST be Cozo Datalog stored under the project rule surface, and every executable rule file MUST have a sibling human-readable translation document explaining what the rule does and how to satisfy it.

#### Scenario: shipped rule has translation sibling
- **WHEN** the system ships a rule such as `ungrounded`
- **THEN** the executable rule file is paired with a sibling English translation document for explanation and operator review

#### Scenario: dual-format hedge is not allowed
- **WHEN** a project defines rules in v0.3
- **THEN** the rule surface uses one executable format rather than maintaining multiple normative rule syntaxes in parallel

### Requirement: Shipped rule catalogue
The system SHALL ship the following default rules with the documented semantics: `ungrounded`, `unresolved-terms`, `stale-cascade`, `lockable`, `correlated-error`, and `dangling-definition`.

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

### Requirement: Severity defaults and override boundaries
The system SHALL assign default severities by rule and mode. `unresolved-terms`, `dangling-definition`, and `stale-cascade` MUST remain strict and non-overridable. `lockable` MUST remain a manual gate evaluated only on `dont lock`. `ungrounded` MUST default to warn in permissive mode and strict in strict mode, and remain overridable. `correlated-error` MUST default to warn in both modes and remain overridable.

#### Scenario: permissive mode keeps ungrounded as warning
- **WHEN** the project runs in permissive mode without overrides
- **THEN** `ungrounded` produces warnings rather than refusing `dont conclude`

#### Scenario: strict mode escalates ungrounded
- **WHEN** the project runs in strict mode without overrides
- **THEN** `ungrounded` refuses unresolved CURIE references

#### Scenario: non-overridable rules stay strict
- **WHEN** a project attempts to soften `unresolved-terms`, `dangling-definition`, or `stale-cascade`
- **THEN** those rules remain strict because their severity is not project-overridable

#### Scenario: lockable is manual gate only
- **WHEN** a verified claim exists but the caller does not invoke `dont lock`
- **THEN** `lockable` does not autonomously refuse unrelated commands

### Requirement: Rule outcomes and error taxonomy boundary
The system SHALL distinguish rule-layer outcomes from verb-level validators. Rule-layer strict failures MUST use error code `rule-not-met` with `rule_name` naming the specific rule. Rule-layer warn outcomes MUST surface in `warnings[]`. Verb-level validators such as `reason-required`, `reason-not-hedge`, `no-evidence`, `atoms-incomplete`, and `wrong-entity-kind` MUST use their own dedicated error codes and MUST set `rule_name` to `null`.

#### Scenario: strict rule failure uses rule-not-met
- **WHEN** `lockable` refuses a `dont lock` operation
- **THEN** the error envelope uses code `rule-not-met`
- **AND** `rule_name` is `lockable`

#### Scenario: warning rule emits warning payload
- **WHEN** `correlated-error` is configured as warn and triggers during an operation
- **THEN** the operation may still succeed
- **AND** the envelope carries a warning entry naming `correlated-error`

#### Scenario: verb-level validator is not presented as rule failure
- **WHEN** `dont trust` rejects a hedge-only reason
- **THEN** the error code is `reason-not-hedge`
- **AND** `rule_name` is `null` rather than `vague-reason`

### Requirement: Vague-reason migration boundary
The system SHALL treat the v0.2 `vague-reason` rule as removed from the shipped rule set. In v0.3 the presence and anti-hedge checks on `trust --reason` MUST be implemented as verb-level validators, while projects remain free to add softer custom rule-layer checks on related reasoning quality concerns.

#### Scenario: shipped rule list excludes vague-reason
- **WHEN** the operator inspects the default rule catalogue
- **THEN** `vague-reason` is absent from the shipped rules

#### Scenario: custom softer rule remains possible
- **WHEN** a project wants a warn-level check adjacent to reason quality
- **THEN** it may add a project-specific rule without replacing the unconditional verb-level refusal semantics of `trust`
