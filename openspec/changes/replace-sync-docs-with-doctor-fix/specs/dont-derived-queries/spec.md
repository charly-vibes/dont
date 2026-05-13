## MODIFIED Requirements

### Requirement: Diagnostic and schema queries

The system SHALL provide diagnostic and schema queries. By default, these queries are read-only. `dont doctor` MUST report substrate reachability, rule compilation health, freshness/availability checks for auxiliary surfaces, and a managed-docs staleness check covering the root managed block(s) and `.dont/AGENTS.md` as defined in `dont-agent-help`. `dont doctor` MUST accept a `--fix` flag whose behaviour is defined by `dont-agent-help` (rewrite managed regions and the canonical file). Without `--fix`, `dont doctor` MUST remain read-only. `dont schema <name>` MUST print the JSON Schema for the named envelope or payload type, and bare `dont schema` MUST list available schema names.

#### Scenario: doctor reports health checks

- **WHEN** the caller runs `dont doctor --json`
- **THEN** the command returns `envelope_kind: "doctor"`
- **AND** the payload contains the diagnostic checks relevant to store health, rules, auxiliary tooling, and managed-docs staleness

#### Scenario: doctor strict mode escalates warnings

- **WHEN** the caller runs `dont doctor --strict --json`
- **THEN** warning or failing checks are treated according to the strict-exit semantics defined by `dont-cli-surface`

#### Scenario: doctor without fix is read-only

- **WHEN** the caller runs `dont doctor` or `dont doctor --json` without `--fix`
- **THEN** the command performs no writes to any file on disk

#### Scenario: doctor with fix performs managed-doc repair

- **WHEN** the caller runs `dont doctor --fix`
- **THEN** the command applies the repair behaviour defined by `dont-agent-help` for managed regions and `.dont/AGENTS.md`
- **AND** the doctor envelope subsequently reports the managed-docs check as `status: "pass"` when re-run on the same project state

#### Scenario: schema without argument lists names

- **WHEN** the caller runs `dont schema --json`
- **THEN** the response lists schema names available for inspection rather than one schema document

#### Scenario: schema with name prints one schema

- **WHEN** the caller runs `dont schema claim --json`
- **THEN** the response returns the `SchemaDoc` for the named schema target
