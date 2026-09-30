# dont-rule-engine Specification

## Purpose
TBD - created by archiving change add-dont-rules-specs. Update Purpose after archive.
## Requirements
### Requirement: Single rule source format with sibling translation
The system SHALL represent project rules in a single executable rule format rather than multiple interchangeable rule syntaxes. In v0.3 this executable format MUST be Cozo Datalog stored under the project rule surface, and every executable rule file MUST have a sibling human-readable translation document explaining what the rule does and how to satisfy it.

#### Scenario: shipped rule has translation sibling
- **WHEN** the system ships a rule such as `ungrounded`
- **THEN** the executable rule file is paired with a sibling English translation document for explanation and operator review

#### Scenario: dual-format hedge is not allowed
- **WHEN** a project defines rules in v0.3
- **THEN** the rule surface uses one executable format rather than maintaining multiple normative rule syntaxes in parallel

### Requirement: Shipped rule catalogue
The system SHALL ship the following default rules with the documented semantics: `ungrounded`, `unresolved-terms`, `stale-cascade`, `lockable`, `correlated-error`, `dangling-definition`, `term-nonfunctional-label`, and `rule-claim-structure`.

`term-nonfunctional-label` is an off-by-default warn-severity rule that flags terms whose label text suggests a non-functional relationship has been folded into the type. Matching is heuristic (configurable token patterns; see `dont-project-config`); the rule is disabled by default because false-positives are expected on valid noun phrases. Its purpose is to surface candidates for aspect-shaped redesign once aspects land as a primitive (currently deferred to §17). Every shipped rule, including `term-nonfunctional-label`, MUST have a sibling human-readable translation document.

`rule-claim-structure` is an off-by-default warn-severity rule that validates claims tagged as rule claims (via `rule-claim-type` term:uuid in `depends_on`) against the six-slot semantic schema defined in `dont-rule-claim-schema`. The rule SHALL check for the presence of mandatory slot markers (`[TRIGGER]` and one of `[MODE]` or `[CONFIG]`) in the claim text. Missing mandatory slots SHALL produce a warning identifying the absent slot. Warn-severity violations SHALL NOT change the claim's stored status — a verified claim that triggers a `rule-claim-structure` warning remains verified in the database. The rule MUST have a sibling human-readable translation document. The rule SHALL be disabled by default because it is only useful to projects that have adopted the rule claim convention.

#### Scenario: ungrounded flags or refuses unresolved CURIEs
- **WHEN** a claim references CURIEs that do not resolve
- **THEN** `ungrounded` either emits a warning or refuses the transition according to the project's mode and severity configuration

#### Scenario: unresolved-terms blocks dismiss
- **WHEN** the caller attempts `dont dismiss` on a claim whose CURIEs remain unresolved in coined or imported vocabulary
- **THEN** `unresolved-terms` refuses the dismissal

#### Scenario: stale-cascade computes derived stale assessment
- **WHEN** a trust transition moves an entity to persisted status `doubted`
- **THEN** `stale-cascade` computes `stale` as a derived assessment for verified dependents reached through supported dependency edges
- **AND** it does not emit a status-changing event or mutate persisted dependent statuses
- **AND** locked and ignored entities are exempt from derived stale output

#### Scenario: stale-cascade traversal is cycle-safe
- **WHEN** dependency traversal revisits an entity already seen in the current trace
- **THEN** traversal stops for that branch
- **AND** the cycle itself does not trigger `stale`

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

#### Scenario: rule-claim-structure warns on missing mandatory slot
- **WHEN** `rule-claim-structure` is enabled and evaluates a tagged rule claim missing the `[TRIGGER]` marker
- **THEN** the envelope carries a warning identifying the absent slot by name
- **AND** the triggering operation is not refused (warn severity)

#### Scenario: rule-claim-structure ignores untagged claims
- **WHEN** `rule-claim-structure` is enabled and evaluates a claim without `rule-claim-type` in its `depends_on`
- **THEN** no warning is emitted for that claim regardless of its text content

#### Scenario: rule-claim-structure is disabled by default
- **WHEN** a project does not explicitly enable `rule-claim-structure` in its configuration
- **THEN** no `rule-claim-structure` warnings are emitted

#### Scenario: rule-claim-structure has a translation sibling
- **WHEN** the system ships `rule-claim-structure`
- **THEN** the executable rule file is paired with a sibling English translation document for explanation and operator review

### Requirement: Severity defaults and override boundaries
The system SHALL assign default severities by rule and mode. `unresolved-terms`, `dangling-definition`, and `stale-cascade` MUST remain strict and non-overridable. Strict `stale-cascade` means commands that gate on dependency integrity (including `dont lock`) MUST refuse when the computed trace contains `stale`; it does not mean the rule persists a `stale` status. `lockable` MUST remain a manual gate evaluated only on `dont lock`. `ungrounded` MUST default to warn in permissive mode and strict in strict mode, and remain overridable. `correlated-error` MUST default to warn in both modes and remain overridable.

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
