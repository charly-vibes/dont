## ADDED Requirements
### Requirement: Trace query explains dependency and support blockers
The system SHALL provide `dont trace <entity-id>` as a read-only diagnostic query that explains the currently relevant dependency or support paths for a claim or term. `trace` SHALL traverse the same dependency/support relationships consulted by current derived-assessment and blocker analysis, including dependency edges such as claim-to-claim, claim-to-term, and term-to-term relations where applicable, rather than inventing a separate graph model. The query SHALL focus on paths that explain derived assessments or gated verification decisions rather than dumping the entire graph indiscriminately.

#### Scenario: stale claim reveals blocking path
- **WHEN** a claim is currently blocked because one upstream dependency makes its support trace stale
- **THEN** `dont trace <claim-id> --json` returns the path from the claim to the blocking dependency and identifies the blocker condition

#### Scenario: unresolved term reveals missing resolution point
- **WHEN** a claim or term carries an unresolved-term condition
- **THEN** the trace output identifies the unresolved CURIE or missing target that caused the blockage

### Requirement: Trace output is structured and actionable
The trace result SHALL expose enough structured information for a harness or human to act on it. At minimum, each reported blocker path MUST identify the starting entity, the traversed path, the blocking node or unresolved reference, the blocker kind, and one or more suggested next actions. Suggested next actions SHALL be valid commands for the blocker kind being reported and SHALL follow the normal remediation style used elsewhere in `dont` rather than inventing free-form pseudo-commands.

#### Scenario: trace includes remediation-rich blocker entry
- **WHEN** a claim is blocked by an upstream doubted or unresolved dependency
- **THEN** the trace output includes a blocker entry with a suggested next command such as inspecting, defining, or revisiting the blocking node

#### Scenario: multiple independent blockers are listed separately
- **WHEN** two unrelated dependency paths independently block one entity
- **THEN** the trace output reports them as separate blocker entries rather than collapsing them into one opaque summary

#### Scenario: cyclic dependency trace remains bounded
- **WHEN** the traversed dependency/support graph contains a cycle
- **THEN** `dont trace <entity-id> --json` terminates without infinite expansion and reports the bounded path context needed for diagnosis

### Requirement: Healthy entities yield empty blocker traces
The tracing query SHALL succeed on healthy entities and return an empty blocker set when no dependency or support fallout currently needs explanation.

#### Scenario: verified standalone claim has empty trace blockers
- **WHEN** a verified claim has no unresolved, dangling, stale, or otherwise blocking dependency conditions
- **THEN** `dont trace <claim-id> --json` succeeds and returns no blocker paths
