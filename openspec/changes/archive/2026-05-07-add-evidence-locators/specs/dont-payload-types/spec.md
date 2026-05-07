## ADDED Requirements
### Requirement: Evidence entries may carry structured repository locator details
JSON payloads that expose evidence entries SHALL allow optional structured repository locator fields in addition to the existing URI summary form. When repository-grounded evidence is used, each evidence entry MAY be an object containing `kind: "repo-file"`, repository-relative `path`, any available `line_start`, `line_end`, `anchor`, or `note`, and drift-audit fields such as `excerpt`, `fingerprint`, and `audit.status`. Parsers MUST ignore absent optional fields and MUST preserve present structured fields without reparsing display text.

#### Scenario: claim view exposes repository locator fields
- **WHEN** `dont show <entity-id> --json` returns a claim grounded in repository evidence
- **THEN** the relevant evidence entry can include structured top-level repository locator fields with path and line-span details

#### Scenario: why view preserves drift-audit fields
- **WHEN** `dont why <entity-id> --json` returns evidence whose stored fingerprint no longer matches the current source slice
- **THEN** the evidence entry can include structured drift-audit fields such as `audit.status` without changing the entity's persisted status
