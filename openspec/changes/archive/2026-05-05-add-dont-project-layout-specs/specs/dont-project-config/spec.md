## ADDED Requirements

### Requirement: Project identity and mode configuration
The system SHALL expose a `[project]` configuration block containing at least the project name and the current operating mode. The mode MUST support `permissive` and `strict`, and changing the configured mode MUST affect the runtime gating behaviour defined by the mode-dependent command specs without changing the stored data shape.

#### Scenario: permissive mode configured in project block
- **WHEN** `config.toml` sets `[project].mode = "permissive"`
- **THEN** runtime behaviour follows the permissive-mode gating rules referenced by the command and rule specs

#### Scenario: strict mode does not change data shape
- **WHEN** a project switches from permissive to strict in configuration
- **THEN** existing stored entities remain valid as data
- **AND** only subsequent gating behaviour changes according to strict-mode rules

### Requirement: Mode changes are auditable runtime events
The system SHALL treat initial mode establishment and subsequent mode changes as auditable runtime events. When a command start detects that the configured mode differs from the previously recorded mode, the tool MUST record a `mode_changed` event against the synthetic project entity with old mode, new mode, author, and timestamp. The synthetic project entity MUST be a reserved internal entity representing the project itself rather than a claim or term, and it is the canonical target for project-wide audit events.

#### Scenario: init records initial mode
- **WHEN** `dont init` establishes the project's first mode
- **THEN** the tool records a `mode_changed` event for that initial mode establishment

#### Scenario: config mode edit is recorded on next invocation
- **WHEN** an operator edits `config.toml` to change mode and then runs the next `dont` command
- **THEN** the tool records a `mode_changed` event reflecting the old and new modes

### Requirement: Output default configuration
The system SHALL expose an `[output]` block for default output formatting. The default format MUST be able to select JSON as the project's preferred output form.

#### Scenario: json default format configured
- **WHEN** `[output].default_format = "json"`
- **THEN** the project defaults to machine-readable output unless an invocation overrides it

### Requirement: Direct-mode LLM configuration
The system SHALL expose an `[llm]` block for provider and model selection used only in direct mode. Harness mode MUST ignore this block for spawn-producing reasoning commands.

#### Scenario: llm config ignored in harness mode
- **WHEN** the project runs in harness mode
- **THEN** the configured `[llm]` provider and model do not override the harness-managed spawn flow

#### Scenario: llm config used in direct mode
- **WHEN** direct mode is active for a spawn-producing command
- **THEN** the tool consults `[llm]` to decide which provider/model to call

### Requirement: Harness configuration surface
The system SHALL expose a `[harness]` block including the managed-doc targets and the spawn-timeout window. Managed-doc targets MUST determine which root files may receive the shorter `dont` block, and the timeout window MUST inform `spawn_request.expires_at` computation.

#### Scenario: managed docs list targets root files
- **WHEN** `[harness].managed_docs` lists `AGENTS.md` and `CLAUDE.md`
- **THEN** those files are eligible targets for `dont sync-docs`

#### Scenario: spawn timeout config drives expiry
- **WHEN** `[harness].spawn_timeout_hours = 24`
- **THEN** newly issued spawn requests expire twenty-four hours after issuance unless resolved earlier

### Requirement: Rule severity configuration
The system SHALL expose a `[rules]` block for project-level severity assignment. This block MUST be able to declare `strict` and `warn` rule lists, while respecting the non-overridable boundaries defined by the rule-engine capability and the mode-driven default for `ungrounded`.

#### Scenario: correlated-error configured as warning
- **WHEN** `[rules].warn` includes `correlated-error`
- **THEN** that rule contributes warning entries rather than strict refusals unless another rule-engine constraint forbids override

#### Scenario: ungrounded mode default still applies
- **WHEN** the project omits an explicit override for `ungrounded`
- **THEN** its effective severity is still derived from the project mode

### Requirement: Trust hedge configuration
The system SHALL expose `[trust.hedges]` patterns for the verb-level anti-hedge validator on `trust --reason`. Projects MAY extend these patterns for domain-specific epistemic mush, but this configuration MUST remain a verb-level validation aid rather than a rule-engine rule list.

#### Scenario: trust hedge pattern list extended
- **WHEN** a project adds a new hedge phrase under `[trust.hedges].patterns`
- **THEN** `dont trust` treats that phrase as part of the anti-hedge validation surface

### Requirement: Storage tuning configuration
The system SHALL expose a `[storage]` block for store busy-retry tuning, including retry attempts and retry backoff base duration.

#### Scenario: storage retry tuned
- **WHEN** `[storage]` sets busy retry attempts and base delay
- **THEN** store-conflict recovery uses those configured values when applying its retry policy

### Requirement: Evidence verification tuning
The system SHALL expose a `[verify_evidence]` block for network politeness and timeout tuning, including concurrency, per-host request rate, per-host burst, retry limit for `429`/`503`, and default timeout.

#### Scenario: verify-evidence timeout configured
- **WHEN** `[verify_evidence].default_timeout_s` is set
- **THEN** `dont verify-evidence` uses that configured default when the caller does not supply `--timeout`

#### Scenario: verify-evidence politeness configured
- **WHEN** the project sets per-host rate and concurrency values
- **THEN** evidence verification uses those configured network politeness values

### Requirement: Import adapter configuration
The system SHALL expose an `[import]` configuration block for adapter enablement and endpoint/base settings. The configuration surface MUST be able to represent importer-specific settings such as OLS base URL, Wikidata endpoint, OpenAlex base URL, and LinkML enablement. It MUST also support importer-specific sub-blocks for tightening shared defaults such as rate limits or adapter-local settings.

#### Scenario: importer endpoint configured
- **WHEN** `[import].wikidata.endpoint` is set
- **THEN** the Wikidata importer uses that configured endpoint

#### Scenario: linkml adapter can be disabled or enabled
- **WHEN** `[import].linkml.enabled` is configured
- **THEN** the project can explicitly control availability of the LinkML adapter surface

#### Scenario: importer-specific sub-block tightens defaults
- **WHEN** the project config defines a per-importer sub-block such as `[import.wikidata]` with stricter network settings
- **THEN** that importer uses the stricter values for its own operations without changing the global defaults for other importers
