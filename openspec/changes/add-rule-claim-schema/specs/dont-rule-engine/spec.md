## Dependencies

This spec delta requires `dont-rule-claim-schema` to be active. The six-slot schema, mandatory slot definitions, and `rule-claim-type` tagging mechanism referenced below are defined in `dont-rule-claim-schema`.

## MODIFIED Requirements

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
