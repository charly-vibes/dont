## Dependencies

This spec delta requires `dont-rule-claim-schema` to be active. The `rule-claim-structure` rule referenced below is defined in `dont-rule-engine` (this change).

## MODIFIED Requirements

### Requirement: Known warning codes for rule-layer rules

The system SHALL extend the known warning codes for envelope version 0.2. The rule-layer warning codes SHALL include `term-nonfunctional-label` and `rule-claim-structure`.

`rule-claim-structure` is a warn-severity code attached to `warnings[]` on `ok: true` envelopes. It identifies tagged rule claims that are missing one or both mandatory slot markers (`[TRIGGER]` and one of `[CONFIG]` or `[MODE]`). The `rule_name` field SHALL be `"rule-claim-structure"` and the `entity_id` SHALL be the `claim:uuid` of the violating claim.

#### Scenario: rule-claim-structure warning appears on success envelope

- **WHEN** `rule-claim-structure` is enabled and `dont prime` evaluates a tagged rule claim missing a mandatory slot marker
- **THEN** the envelope has `ok: true` and `warnings[]` contains an entry with `code: "rule-not-met"`, `rule_name: "rule-claim-structure"`, and `entity_id` identifying the violating claim

#### Scenario: rule-claim-structure warning does not appear when rule is disabled

- **WHEN** `rule-claim-structure` is not enabled in project config
- **THEN** no `rule-claim-structure` warning code appears on any envelope regardless of claim content
