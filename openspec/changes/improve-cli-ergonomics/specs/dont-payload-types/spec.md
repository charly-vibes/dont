# dont-payload-types Deltas

## MODIFIED Requirements

### Requirement: Human-mode output structure
The system SHALL provide a structured human-readable output format that presents entity details in a "card" or "table" view by default. This view SHALL include the entity's ID, status, content (statement or definition), and any attached evidence, hypotheses, or atoms.

#### Scenario: ClaimView in human mode
- **WHEN** `dont show claim:X` is invoked without the `--json` flag
- **THEN** the output displays a human-readable card including the claim's statement and current status

## ADDED Requirements

### Requirement: Hypothesis and Atom payloads
The system SHALL support structured payloads for hypotheses and atoms, including their assessment status and supporting/refuting evidence references.

#### Scenario: viewing assessed hypotheses
- **WHEN** a claim's hypotheses are displayed in human or JSON mode
- **THEN** each entry includes its index, text, and the lists of supporting/refuting evidence identifiers
