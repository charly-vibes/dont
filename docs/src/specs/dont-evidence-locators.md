# dont-evidence-locators Specification

## Purpose
Defines structured evidence locators for repository-grounded verification. Repository-relative file locators, optional line spans and anchors, captured excerpts, and stability fingerprints enable operators to point to a specific source location when grounding a claim and allow later readers to audit what text was relied on and detect drift.

## Requirements
### Requirement: Repository-scoped evidence locators
The system SHALL support structured evidence locators for repository-grounded verification. At minimum, a locator MUST be able to name a repository-relative file path and MAY additionally include a line span, anchor, and free-form note. Repository-relative locators SHALL resolve against the project root rather than the caller's transient working directory. Plain URI-only evidence SHALL remain supported as a compatibility path, but repository-relative locators are the recommended form for project-grounded claims.

#### Scenario: repository file locator points to a line span
- **WHEN** an operator grounds a claim from `README.md` lines 10 through 18
- **THEN** the evidence locator can represent the repository-relative path and that line span without requiring an opaque absolute `file://` URI

#### Scenario: repository file locator remains stable across subdirectories
- **WHEN** the caller invokes `dont` from a nested working directory inside the same project
- **THEN** the same repository-relative evidence locator still resolves to the same target file

#### Scenario: locator escaping the project root is refused
- **WHEN** an operator supplies a repository-relative locator that normalizes outside the project root, including via `..` traversal or a symlinked escape
- **THEN** the evidence locator is refused rather than resolving to content outside the project

### Requirement: Captured excerpts and fingerprints remain auditable
The system SHALL allow evidence records to carry an optional captured excerpt and an optional stability fingerprint derived from the referenced source material. The excerpt is for later human audit of what text was relied on; the fingerprint is for later drift detection and SHALL NOT be treated as a proof of semantic correctness by itself.

#### Scenario: doc claim stores excerpt for later audit
- **WHEN** a claim is grounded in one sentence from a README section
- **THEN** the evidence record can carry that captured excerpt so a later reader can see what text supported the claim

#### Scenario: fingerprint detects later source drift
- **WHEN** the source file changes after the claim was verified
- **THEN** later inspection surfaces such as `dont show`, `dont why`, or `dont verify-evidence` can report that the stored fingerprint no longer matches the current source slice without changing the claim's persisted status automatically

#### Scenario: line span no longer exists after edits
- **WHEN** the stored locator names a line span that no longer exists in the current file revision
- **THEN** later inspection reports the locator as unresolved or drifted for audit purposes without rewriting the entity's persisted status automatically

### Requirement: Structured locators are visible in inspection views
The system SHALL surface structured evidence locators in claim and term inspection outputs in addition to the existing human-readable evidence summary. JSON inspection payloads MUST preserve the locator structure so external harnesses can render or audit it without reparsing display text.

#### Scenario: show returns structured evidence details
- **WHEN** a caller runs `dont show <entity-id> --json` on an entity grounded in repository evidence
- **THEN** the returned evidence entries include the structured locator data needed to identify the file and line span

#### Scenario: why can explain evidence provenance without guessing from display text
- **WHEN** a harness runs `dont why <entity-id> --json`
- **THEN** the evidence details are available as structured fields rather than only as a flattened prose string
