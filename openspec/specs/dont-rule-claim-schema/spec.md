# dont-rule-claim-schema Specification

## Purpose
TBD - created by archiving change add-rule-claim-schema. Update Purpose after archive.
## Requirements
### Requirement: Six semantic slots for rule-describing claims
The system SHALL define a six-slot semantic schema for claims that describe the behavior of a `dont` rule. The six slots are: INVOCATION MODEL, TRIGGER CONDITION, PRECONDITION GUARD, EVALUATION MODEL, MODE/CONFIG, and BOUNDARY. TRIGGER CONDITION and MODE/CONFIG SHALL be mandatory — a rule claim that omits both is incomplete by definition. MODE/CONFIG is satisfied by either a `[CONFIG]` marker (covers enablement: on/off by default), a `[MODE]` marker (covers severity behavior across project modes), or both — either sub-marker alone meets the requirement. INVOCATION MODEL, PRECONDITION GUARD, EVALUATION MODEL, and BOUNDARY SHALL be optional, each carrying an explicit default that the `rule-claim-structure` rule applies when the slot is absent.

The `rule-claim-structure` rule SHALL validate mandatory slot marker presence only. It SHALL NOT evaluate the accuracy of slot content — a structurally complete claim with incorrect content passes the rule. Content accuracy is a human responsibility enforced through the claim evidence and doubt mechanisms.

Slot defaults when omitted:
- INVOCATION MODEL: the rule runs as a background lint evaluated by `dont prime`
- PRECONDITION GUARD: the rule evaluates all inputs without a silent-skip threshold
- EVALUATION MODEL: the rule is stateless and demand-evaluated
- BOUNDARY: no explicit sibling-rule boundary exists for this rule

#### Scenario: trigger slot absence is a schema violation
- **WHEN** a claim is tagged as a rule claim and does not contain a TRIGGER CONDITION slot marker
- **THEN** the `rule-claim-structure` rule flags it as a schema violation

#### Scenario: mode/config slot absence is a schema violation
- **WHEN** a claim is tagged as a rule claim and contains neither a MODE slot marker nor a CONFIG slot marker
- **THEN** the `rule-claim-structure` rule flags it as a schema violation

#### Scenario: optional slot omission applies known default
- **WHEN** a rule claim omits the EVALUATION MODEL slot
- **THEN** the claim is treated as asserting that the rule is stateless and demand-evaluated
- **AND** the `rule-claim-structure` rule does not flag the omission as a violation

#### Scenario: optional slot omission is not a violation when default is correct
- **WHEN** a rule claim for `ungrounded` omits the INVOCATION MODEL slot
- **THEN** the claim inherits the background-lint default
- **AND** no warning is emitted because the default is correct for `ungrounded`

### Requirement: Rule claim tagging via term dependency
The system SHALL support tagging a claim as a rule claim by including the `term:uuid` of the `rule-claim-type` term in its `depends_on` entries. The `rule-claim-type` term SHOULD be defined with `dont define` before creating rule claims; the resulting `term:uuid` is the correct depends_on value. Using the bare string `rule-claim-type` or an unregistered CURIE as a depends_on entry will trigger the `unresolved-terms` rule and is not a valid tag. The `rule-claim-structure` rule SHALL only evaluate claims that carry this tag; untagged claims SHALL be ignored by the rule regardless of their content.

#### Scenario: tagged claim is evaluated by rule-claim-structure
- **WHEN** a claim lists `rule-claim-type` in its `depends_on`
- **AND** the `rule-claim-structure` rule is enabled
- **THEN** the rule checks the claim's text for mandatory slot markers

#### Scenario: untagged claim is not evaluated
- **WHEN** a claim does not include `rule-claim-type` in its `depends_on`
- **THEN** `rule-claim-structure` ignores the claim even if its text resembles a rule description

### Requirement: Slot marker format and template
The system SHALL publish a canonical slot marker format for rule claim text. The format SHALL use bracketed uppercase slot symbols (`[TRIGGER]`, `[MODE]`, `[CONFIG]`, `[GUARD]`, `[EVAL]`, `[INVOCATION]`, `[BOUNDARY]`) as lexical markers that both human authors and the `rule-claim-structure` rule use to identify slot coverage. The canonical template SHALL be published in `.dont/AGENTS.md` (in the rule claim authoring section) and accessible via `dont help --howto rule-claims`.

Canonical template:

```
[INVOCATION] <rule-name> runs as: background lint | opt-in via `dont check --<flag>`
[CONFIG]     Enabled by default: yes | no
[MODE]       In permissive mode: warn | strict | same as strict | n/a
[TRIGGER]    Fires when: <condition>
[GUARD]      Silently skips: <inputs>   (omit line if no guard)
[EVAL]       Evaluation model: stateless demand | event-driven on <event>   (omit if stateless demand)
[BOUNDARY]   Does not handle: <edge cases>; defers to <other-rule>   (omit if no boundary)
```

Optional slots SHALL be documented as omit-when-default to reduce claim verbosity.

#### Scenario: template is discoverable via help
- **WHEN** the caller runs `dont help --howto rule-claims`
- **THEN** the output includes the canonical template and per-slot guidance on when optional slots become load-bearing

#### Scenario: template is in agent-facing docs
- **WHEN** an agent reads `.dont/AGENTS.md`
- **THEN** the rule claim authoring section includes the canonical template

#### Scenario: claim authored from template passes structural validation
- **WHEN** an author creates a rule claim using the canonical template with both mandatory slots filled
- **AND** the claim includes the `rule-claim-type` term:uuid in its `depends_on`
- **THEN** `rule-claim-structure` does not flag the claim
