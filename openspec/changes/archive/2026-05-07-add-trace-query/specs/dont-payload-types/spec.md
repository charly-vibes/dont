## ADDED Requirements
### Requirement: TraceView payload
The system SHALL return a `TraceView` payload (envelope_kind: `"trace"`) containing: `entity_id`, `blockers[]`, and `as_of`. Each blocker entry MUST identify the blocker kind, the starting entity, the traversed path, the blocking node or unresolved reference, and `remediation[]` entries in the normal `{command, description}` shape. The payload MAY additionally include bounded cycle notes or other explanatory metadata that does not change the core blocker semantics.

#### Scenario: trace payload includes one blocker path
- **WHEN** `dont trace <entity-id> --json` is run on an entity with one currently relevant blocker path
- **THEN** the payload contains one blocker entry with path and remediation details

#### Scenario: healthy entity trace has empty blockers
- **WHEN** `dont trace <entity-id> --json` is run on an entity with no relevant blocker paths
- **THEN** the payload succeeds with `blockers: []`
