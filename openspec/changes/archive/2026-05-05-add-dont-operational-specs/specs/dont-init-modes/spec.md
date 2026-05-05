## ADDED Requirements

### Requirement: Init creates persistent per-project state
The system SHALL provide a per-project `init` operation that creates persistent project-local `dont` state, installs the project's seed vocabulary snapshot, and records the project's initial operating mode as an auditable project event.

#### Scenario: init creates persistent project-local state
- **WHEN** an actor initializes `dont` in a project directory
- **THEN** the tool creates persistent project-local `dont` state for that directory
- **AND** installs the seed vocabulary snapshot for that project
- **AND** records the project's initial operating mode as an auditable project event

#### Scenario: repeated init refuses re-initialization
- **WHEN** an actor invokes `init` for a directory that is already initialized
- **THEN** the command is refused as an already-initialized project rather than silently overwriting the existing project state

#### Scenario: init defaults to permissive mode unless strict is explicitly requested
- **WHEN** an actor initializes a new project without explicitly requesting strict mode
- **THEN** the project starts in permissive mode

### Requirement: Init installs a locked seed vocabulary snapshot
The system SHALL install a seed vocabulary with the `dont:` prefix during initialization, SHALL snapshot that seed into project-local state, and SHALL make the snapshotted seed authoritative for that project until an explicit migration mechanism changes it.

#### Scenario: seed vocabulary is installed during init
- **WHEN** `init` succeeds for a new project
- **THEN** the seed vocabulary is snapshotted into the project
- **AND** seed terms are created from that snapshot

#### Scenario: seed terms start locked
- **WHEN** the seed vocabulary is installed
- **THEN** each seed term enters the project in the `locked` state

#### Scenario: tool upgrades do not silently rewrite the project seed snapshot
- **WHEN** the `dont` tool is upgraded after project initialization
- **THEN** the existing project's seed snapshot remains authoritative until an explicit seed-migration action occurs

### Requirement: Canonical seed vocabulary content is constrained
The system SHALL define a canonical ten-term `dont:` seed vocabulary containing `dont:Entity`, `dont:Claim`, `dont:Term`, `dont:Evidence`, `dont:kind_of`, `dont:related_to`, `dont:defined_as`, `dont:Hypothesis`, `dont:Retraction`, and `dont:external_ref`, and SHALL exclude external ontology terms that would import unwanted semantics into the seed.

#### Scenario: seed vocabulary includes the canonical dont bootstrap terms
- **WHEN** the project inspects the installed seed vocabulary
- **THEN** it includes `dont:Entity`, `dont:Claim`, `dont:Term`, `dont:Evidence`, `dont:kind_of`, `dont:related_to`, `dont:defined_as`, `dont:Hypothesis`, `dont:Retraction`, and `dont:external_ref`

#### Scenario: seed vocabulary excludes external ontology defaults
- **WHEN** the project inspects the installed seed vocabulary
- **THEN** it does not treat imported ontology defaults such as `owl:Thing` or `rdfs:subClassOf` as part of the seed vocabulary

### Requirement: Projects operate in permissive or strict mode
The system SHALL support permissive and strict operating modes with the invariant that no verified entity may depend on unresolved term references.

#### Scenario: permissive mode allows unresolved references at conclude time
- **WHEN** the project is in permissive mode
- **AND** an actor concludes a claim with unresolved term references
- **THEN** the claim may enter the project as `unverified`
- **AND** later verification remains blocked until the unresolved references are resolved

#### Scenario: strict mode refuses unresolved references at conclude time
- **WHEN** the project is in strict mode
- **AND** an actor concludes a claim with unresolved term references
- **THEN** the command is refused until those references are resolved

#### Scenario: verified entities never retain unresolved references
- **WHEN** an entity reaches `verified`
- **THEN** the project does not permit that entity to depend on unresolved term references

### Requirement: Define always refuses dangling references
The system SHALL refuse `define` operations that reference undefined parent, related, or attribute terms regardless of project mode.

#### Scenario: permissive mode does not relax define references
- **WHEN** the project is in permissive mode
- **AND** an actor invokes `define` with unresolved referenced terms
- **THEN** the command is refused

#### Scenario: strict mode also refuses unresolved define references
- **WHEN** the project is in strict mode
- **AND** an actor invokes `define` with unresolved referenced terms
- **THEN** the command is refused

### Requirement: Mode changes are recorded as project events
The system SHALL record the project's initial mode and subsequent mode changes as auditable project events attached to project state.

#### Scenario: init records the initial mode
- **WHEN** a project is initialized
- **THEN** the chosen initial mode is recorded as an auditable project event

#### Scenario: later mode changes are recorded
- **WHEN** the project's operating mode changes after initialization
- **THEN** the mode transition is recorded with the previous mode, new mode, author context, and timestamp
