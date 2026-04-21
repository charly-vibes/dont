## ADDED Requirements

### Requirement: Rule listing and inspection commands
The system SHALL provide `dont rules list` and `dont rules show <rule-name>` for inspecting the active rule catalogue. The canonical selector for `show`, `test`, and `explain` SHALL be the rule name rather than a numeric index. Listing MUST expose the available rule names and their configured severities. Showing one rule MUST surface the rule's identity and explanation context sufficient for an operator or harness to understand what it checks.

#### Scenario: rules list returns active catalogue
- **WHEN** the caller runs `dont rules list --json`
- **THEN** the response enumerates the active rules available in the project
- **AND** includes their effective severities

#### Scenario: rules show returns one rule
- **WHEN** the caller runs `dont rules show lockable --json`
- **THEN** the response identifies the `lockable` rule and exposes its operator-facing explanation context

#### Scenario: numeric index is not the canonical rule selector
- **WHEN** the operator consults command help or examples for `dont rules show` or `dont rules test`
- **THEN** the documented selector form is the rule name rather than a numeric index

### Requirement: Rule explanation command
The system SHALL provide `dont explain <rule-name>` to return the human-readable explanation for a rule. The explanation source of truth MUST be the rule's sibling translation document rather than reverse-engineering the executable Datalog at call time.

#### Scenario: explain reads sibling translation
- **WHEN** the caller runs `dont explain ungrounded --json`
- **THEN** the response is derived from the human-readable translation paired with the `ungrounded` rule

#### Scenario: explanation tells operator how to satisfy rule
- **WHEN** the caller reads the explanation for a rule
- **THEN** the content states both what the rule checks and how to satisfy or avoid tripping it

### Requirement: Rule addition contract
The system SHALL provide `dont rules add <file.dl>` for installing a project-specific executable rule. A usable project-specific rule addition MUST consist of the executable rule file plus a sibling human-readable translation document, and the rule becomes active on the next `dont` invocation without requiring a process restart.

#### Scenario: added rule becomes active next invocation
- **WHEN** the caller adds a valid project-specific rule and its translation doc
- **THEN** the rule participates in rule evaluation on the next `dont` invocation

#### Scenario: added rule requires translation sibling
- **WHEN** a project-specific rule is installed
- **THEN** the rule surface expects a sibling explanation document so `dont explain <rule>` can describe it

### Requirement: Rule dry-run and compile testing
The system SHALL provide `dont rules test <rule-name>` as a dry-run command that compiles the named rule and evaluates it against the current store without emitting lifecycle events or mutating entity state. If compilation fails, the error surface MUST identify that compilation failure rather than silently ignoring the rule.

#### Scenario: rules test is non-mutating
- **WHEN** the caller runs `dont rules test no-appeal-to-common-knowledge --json`
- **THEN** the command reports the entities currently matched by the rule
- **AND** it does not emit events or change any entity status

#### Scenario: compile failure is surfaced explicitly
- **WHEN** the named rule does not parse or compile
- **THEN** the command returns an error describing the compilation failure so the operator can fix the rule file

### Requirement: Severity guidance for added rules
The system SHALL support assigning project-specific rules as either `warn` or `strict`. Warn severity MUST emit `warnings[]` without refusing the triggering command, while strict severity MUST refuse with `rule-not-met` and the custom `rule_name`. Guidance for newly added rules SHOULD default operators toward `warn` unless the project truly requires an unconditional refusal.

#### Scenario: custom warn rule adds warning only
- **WHEN** a project-specific rule is configured as warn and matches an operation
- **THEN** the operation can still succeed with a warning entry naming that rule

#### Scenario: custom strict rule refuses with its own name
- **WHEN** a project-specific rule is configured as strict and matches an operation
- **THEN** the refusal uses code `rule-not-met`
- **AND** `rule_name` is the custom rule's identifier
