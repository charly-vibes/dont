## ADDED Requirements
### Requirement: GroundInput schema
`GroundInput` SHALL accept: `statement` (required string), `evidence` (required non-empty EvidenceSpec array), and `author?` (AuthorString). In the initial version it SHALL NOT add ground-specific convenience fields for atoms, dependencies, or references; callers needing those richer shapes MUST use the underlying core verbs directly.

#### Scenario: ground with statement and evidence validates
- **WHEN** `dont ground "..." --evidence uri1 --json` is run
- **THEN** the input is validated as `{statement, evidence: [{uri: "uri1"}]}`

#### Scenario: ground rejects empty evidence list
- **WHEN** `GroundInput.evidence` is provided as an empty array
- **THEN** the command is refused rather than creating an ungrounded claim
