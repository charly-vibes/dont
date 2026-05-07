## ADDED Requirements
### Requirement: Help teaches structured repository evidence as the preferred grounding mode
The system SHALL teach structured repository evidence as the preferred mode for grounding repository facts. Operator-facing help and tutorials MUST recommend repository-relative file locators over opaque absolute `file://` paths when the evidence source is inside the current project, while still noting that URI-only evidence remains supported for compatibility.

#### Scenario: tutorial recommends repository-relative evidence
- **WHEN** the caller reads the grounding-oriented tutorial or how-to material
- **THEN** the examples prefer repository-relative evidence locators over absolute `file://` paths for project files

#### Scenario: compatibility path remains documented
- **WHEN** the help text describes repository evidence locators
- **THEN** it still notes that plain URI-only evidence remains supported as a compatibility path
