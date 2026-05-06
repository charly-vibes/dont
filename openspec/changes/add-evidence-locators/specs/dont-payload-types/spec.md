## ADDED Requirements
### Requirement: Evidence entries may carry structured repository locator details
JSON payloads that expose evidence entries SHALL allow optional structured locator fields in addition to the existing summary fields. When repository-grounded evidence is used, each evidence entry MAY include a `locator` object naming the repository-relative path and any available line span, anchor, or note, and MAY include drift-audit fields such as `excerpt`, `fingerprint`, and `fingerprint_status`. Parsers MUST ignore absent optional fields and MUST preserve present structured fields without reparsing display text.

#### Scenario: claim view exposes repository locator fields
- **WHEN** `dont show <entity-id> --json` returns a claim grounded in repository evidence
- **THEN** the relevant evidence entry can include a structured `locator` object with repository-relative path and line-span details

#### Scenario: why view preserves drift-audit fields
- **WHEN** `dont why <entity-id> --json` returns evidence whose stored fingerprint no longer matches the current source slice
- **THEN** the evidence entry can include structured drift-audit fields such as `fingerprint_status` without changing the entity's persisted status
