## MODIFIED Requirements
### Requirement: Diagnostic and schema queries
The system SHALL provide diagnostic and schema queries. By default, these queries are read-only. `dont doctor` MUST report substrate reachability, rule compilation health, freshness/availability checks for auxiliary surfaces, a managed-docs staleness check covering the root managed block(s) and `.dont/AGENTS.md` as defined in `dont-agent-help`, and a managed-skills staleness check covering configured first-party managed skill packs under `.agents/skills/` as defined in `dont-managed-skills`. `dont doctor` MUST accept a `--fix` flag whose behaviour is defined by `dont-agent-help` and `dont-managed-skills` (rewrite managed regions, overwrite `.dont/AGENTS.md`, and repair configured managed skill packs). Without `--fix`, `dont doctor` MUST remain read-only. `dont schema <name>` MUST print the JSON Schema for the named envelope or payload type, and bare `dont schema` MUST list available schema names.

#### Scenario: doctor reports checks in json envelope
- **WHEN** the caller runs `dont doctor --json`
- **THEN** the command succeeds with a doctor envelope
- **AND** the payload contains the diagnostic checks relevant to store health, rules, auxiliary tooling, managed-docs staleness, and managed-skills staleness

#### Scenario: doctor with fix performs managed artifact repair
- **WHEN** the caller runs `dont doctor --fix --json`
- **AND** one or more configured managed docs or managed skill packs are stale
- **THEN** the command applies the repair behaviour defined by `dont-agent-help` and `dont-managed-skills`
- **AND** the doctor envelope subsequently reports the managed-docs and managed-skills checks as `status: "pass"` when re-run on the same project state
