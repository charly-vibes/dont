## ADDED Requirements
### Requirement: Trace is part of the read-only diagnostic query surface
The system SHALL provide `dont trace <entity-id>` as a read-only diagnostic query for dependency/support blocker inspection. `trace` complements `show` and `why`: `show` returns the current entity view, `why` explains current rules and history, and `trace` explains the blocker paths currently relevant to derived assessments or verification gating.

#### Scenario: trace returns blocker-oriented diagnostics
- **WHEN** the caller runs `dont trace claim:01HX05A9K8VP --json`
- **THEN** the command returns a trace-oriented payload rather than a ClaimView or WhyView

#### Scenario: healthy entity trace succeeds with empty blockers
- **WHEN** the caller runs `dont trace <entity-id> --json` on a healthy entity with no current dependency/support fallout
- **THEN** the command succeeds and returns an empty blocker set
