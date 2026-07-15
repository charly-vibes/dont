## MODIFIED Requirements

### Requirement: Core layout entries and roles
The system SHALL reserve the following entries within `.dont/`: the primary store path, `config.toml`, `AGENTS.md`, `seed/`, `vocab/`, `rules/`, `imports/`, `sessions/`, `schemas/`, and `events.jsonl`. Each entry MUST exist to serve its documented role: persistent store, project configuration, canonical LLM-facing docs, seed vocabulary snapshot, coined vocabulary files, rule files, import manifests, spawn/session scratch, JSON Schema documents, and interchange event log respectively. `.dont/db.cozo*` (the primary store file and any Cozo/SQLite journal or WAL) SHALL be treated as a locally-rebuildable cache derived from `events.jsonl` plus in-progress write transactions. Git-tracked artifacts SHALL include `events.jsonl` as the interchange format; gitignored artifacts SHALL include `db.cozo*` as the rebuildable persistent store.

#### Scenario: events.jsonl is a recognised layout entry
- **WHEN** a project has run `dont log export` at least once
- **THEN** `.dont/events.jsonl` exists as a recognised layout entry

#### Scenario: db.cozo and associated files are gitignored
- **WHEN** `dont init` has scaffolded `.gitignore` or a project follows the recommended layout
- **THEN** `.dont/db.cozo`, `.dont/db.cozo-wal`, and any other Cozo/SQLite journal files are excluded from version control