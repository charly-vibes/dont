## MODIFIED Requirements
### Requirement: Core layout entries and roles
The system SHALL reserve the following entries within `.dont/`: the primary store path, `config.toml`, `AGENTS.md`, `seed/`, `vocab/`, `rules/`, `imports/`, `sessions/`, and `schemas/`. Each entry MUST exist to serve its documented role: persistent store, project configuration, canonical LLM-facing docs, seed vocabulary snapshot, coined vocabulary files, rule files, import manifests, spawn/session scratch, and JSON Schema documents respectively. In addition to the `.dont/` tree, the system MAY manage generated agent-facing compatibility artifacts outside `.dont/` only where another capability explicitly authorizes them; as of this change, the authorized exceptions are the root managed-doc blocks and configured first-party managed skill packs under `.agents/skills/`; this list of authorized exceptions grows only when a subsequent capability change explicitly authorises an additional outside-`.dont/` location.

#### Scenario: canonical agent docs live under .dont
- **WHEN** a harness or operator looks for the primary LLM-facing documentation
- **THEN** `.dont/AGENTS.md` is the canonical source

#### Scenario: rule files live under rules directory
- **WHEN** a project ships default or custom rules
- **THEN** those rule artifacts live under `.dont/rules/`

#### Scenario: schema docs live under schemas directory
- **WHEN** the project exposes JSON Schemas for envelopes or payloads
- **THEN** those schema documents live under `.dont/schemas/`

#### Scenario: managed skill packs are allowed outside .dont
- **WHEN** a configured first-party managed skill pack is installed
- **THEN** the generated pack may live under the project root `.agents/skills/` directory without violating the canonical `.dont/` ownership model
