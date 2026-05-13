## MODIFIED Requirements

### Requirement: Init creates persistent per-project state

The system SHALL provide a per-project `init` operation that creates persistent project-local `dont` state, installs the project's seed vocabulary snapshot, records the project's initial operating mode as an auditable project event, writes the canonical `.dont/AGENTS.md` file in whole from the current generator output, and injects the managed `dont` block (delimited by `<!-- DONT:START -->` and `<!-- DONT:END -->`) into each configured root document listed by `[harness].managed_docs` in project configuration. `init` MUST produce byte-identical managed files to those that `dont doctor --fix` would produce for the same *detected project state* — the inputs the generator reads (configured `managed_docs` targets, `dont` version, installed rules, project mode) — so that running either command after the other with no state change is a no-op.

#### Scenario: init creates persistent project-local state

- **WHEN** an actor initializes `dont` in a project directory
- **THEN** the tool creates persistent project-local `dont` state for that directory
- **AND** installs the seed vocabulary snapshot for that project
- **AND** records the project's initial operating mode as an auditable project event
- **AND** writes `.dont/AGENTS.md` in whole from the current generator output
- **AND** injects the managed `dont` block into each configured root document in `[harness].managed_docs`

#### Scenario: init preserves user-authored content in root documents

- **WHEN** a configured root document already exists with user-authored content and does not yet contain the managed `dont` block sentinels
- **AND** an actor runs `dont init`
- **THEN** the tool inserts the managed block between the sentinels at a deterministic position
- **AND** all preexisting bytes outside the inserted block are preserved exactly

#### Scenario: init is consistent with doctor --fix

- **WHEN** a project has just been initialized by `dont init`
- **AND** the actor immediately runs `dont doctor --fix` with no other state changes
- **THEN** the command makes no byte-level changes to `.dont/AGENTS.md` or to any configured root document

#### Scenario: repeated init refuses re-initialization

- **WHEN** an actor invokes `init` for a directory that is already initialized
- **THEN** the command is refused as an already-initialized project rather than silently overwriting the existing project state

#### Scenario: init defaults to permissive mode unless strict is explicitly requested

- **WHEN** an actor initializes a new project without explicitly requesting strict mode
- **THEN** the project starts in permissive mode
