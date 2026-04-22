## ADDED Requirements

### Requirement: Define shape configuration
The system SHALL expose a `[define.shape]` configuration block for controlling the verb-level label-shape validators on `dont define`. Of the five validators, four are individually disableable (`check_indefinite`, `check_punctuated`, `check_compound`, `check_sentence`); `term-label-empty` has no config toggle because disabling a whitespace-only guard provides no value. Each configurable validator SHALL default to `true` (enforcing at refuse-level when `--label` is supplied, warn-level when only `--doc` is available). Setting any to `false` disables that validator entirely. The block SHALL also allow extending the list of compound-structure markers recognized by `term-compound-undeclared` through a `compound_markers` list. Setting `compound_markers` to an empty list disables marker matching entirely for that validator (equivalent to `check_compound = false`); to extend the default markers without replacing them, list the defaults plus any additions.

#### Scenario: validator disabled via config
- **WHEN** `[define.shape].check_sentence = false`
- **THEN** `dont define` does not fire `term-label-sentence` even when a verb token appears in the label

#### Scenario: default config enforces all checks
- **WHEN** the project omits `[define.shape]` from `config.toml`
- **THEN** all four validators are active with their default refuse-on-label / warn-on-doc behaviour

#### Scenario: compound markers list is extensible
- **WHEN** a project adds `"a quintuple"` to `[define.shape].compound_markers`
- **THEN** `term-compound-undeclared` treats `"a quintuple"` as a compound marker requiring a variable list

### Requirement: Nonfunctional label rule configuration
The system SHALL expose a `[rules.term_nonfunctional]` configuration block for the `term-nonfunctional-label` warn rule. The block SHALL default to `enabled = false`. When `enabled = true`, the `patterns` list drives the token-match heuristic. The project MAY extend or replace the default pattern list to capture domain-specific non-functional relationship phrasing.

#### Scenario: rule enabled via config
- **WHEN** `[rules.term_nonfunctional].enabled = true`
- **THEN** `term-nonfunctional-label` emits warnings when a defined term's label matches a configured pattern

#### Scenario: rule disabled by default
- **WHEN** the project omits `[rules.term_nonfunctional]` from `config.toml`
- **THEN** `term-nonfunctional-label` does not fire

#### Scenario: pattern list is extensible
- **WHEN** a project adds `"wraps a"` to `[rules.term_nonfunctional].patterns`
- **THEN** labels containing `wraps a` as a standalone phrase are candidates for the warning
