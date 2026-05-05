## ADDED Requirements

### Requirement: Canonical project directory
The system SHALL use a `.dont/` directory at the project root as the canonical per-project home for persistent `dont` state and documentation.

#### Scenario: project initialisation creates canonical root
- **WHEN** a project is initialised for `dont`
- **THEN** the project contains a `.dont/` directory as the tool-owned root for per-project state

### Requirement: Core layout entries and roles
The system SHALL reserve the following entries within `.dont/`: the primary store path, `config.toml`, `AGENTS.md`, `seed/`, `vocab/`, `rules/`, `imports/`, `sessions/`, and `schemas/`. Each entry MUST exist to serve its documented role: persistent store, project configuration, canonical LLM-facing docs, seed vocabulary snapshot, coined vocabulary files, rule files, import manifests, spawn/session scratch, and JSON Schema documents respectively.

#### Scenario: canonical agent docs live under .dont
- **WHEN** a harness or operator looks for the primary LLM-facing documentation
- **THEN** `.dont/AGENTS.md` is the canonical source

#### Scenario: rule files live under rules directory
- **WHEN** a project ships default or custom rules
- **THEN** those rule artifacts live under `.dont/rules/`

#### Scenario: schema docs live under schemas directory
- **WHEN** the project exposes JSON Schemas for envelopes or payloads
- **THEN** those schema documents live under `.dont/schemas/`

### Requirement: Seed vocabulary snapshot
The system SHALL keep the installed seed vocabulary under `.dont/seed/` as a project-local snapshot rather than relying solely on a global installation copy.

#### Scenario: seed snapshot exists in project
- **WHEN** a project has been initialised
- **THEN** `.dont/seed/dont-seed.yaml` exists as the local seed vocabulary snapshot for that project

### Requirement: Root-document managed block relationship
The system SHALL treat root-level project documents such as `AGENTS.md` and `CLAUDE.md` as optional hosts for a shorter managed `dont` block rather than as the canonical home of the full docs. Those root docs MUST point readers at `.dont/AGENTS.md` and be eligible targets for managed-block rewriting.

#### Scenario: root agents doc contains managed pointer
- **WHEN** the project includes a root `AGENTS.md`
- **THEN** it contains the shorter `dont` managed block rather than replacing `.dont/AGENTS.md` as the canonical document

#### Scenario: claude doc can also host managed block
- **WHEN** the project includes `CLAUDE.md`
- **THEN** it may also receive the managed `dont` block pointing back to `.dont/AGENTS.md`

### Requirement: Rules, imports, and sessions are project-local state
The system SHALL keep rule artifacts, import manifests/provenance, and spawn/session scratch within `.dont/` so that a project's `dont` state remains self-contained and portable with the repository.

#### Scenario: import cursors remain in project
- **WHEN** import adapters track manifests or cursors
- **THEN** those files live under `.dont/imports/` rather than in a global cache

#### Scenario: spawn scratch remains in project
- **WHEN** spawn requests or clean-context session artifacts are persisted
- **THEN** those artifacts live under `.dont/sessions/`
