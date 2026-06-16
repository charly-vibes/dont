## MODIFIED Requirements
### Requirement: Per-project init establishes local state and mode
The system SHALL provide a per-project `init` operation that creates persistent project-local `dont` state, installs the project's seed vocabulary snapshot, records the project's initial operating mode as an auditable project event, writes the canonical `.dont/AGENTS.md` file in whole from the current generator output, injects the managed `dont` block (delimited by `<!-- DONT:START -->` and `<!-- DONT:END -->`) into each configured root document listed by `[harness].managed_docs` in project configuration, and installs each configured first-party managed skill pack under `.agents/skills/`. `init` MUST produce semantically equivalent managed files to those that `dont doctor --fix` would produce for the same *detected project state* — the inputs the generator reads (configured `managed_docs` targets, configured `managed_skill_packs`, `dont` version, installed rules, project mode) — so that running either command after the other with no state change is a no-op. Equivalence means byte-identical content; implementers MUST NOT include timestamps or other non-deterministic values in generated managed content, ensuring that repeated generation from the same project state always produces byte-identical output.

#### Scenario: init creates canonical docs and skills
- **WHEN** the caller runs `dont init` in a new project with default managed docs and configured managed skill packs
- **THEN** the command writes `.dont/AGENTS.md`
- **AND** injects the managed `dont` block into each configured root document in `[harness].managed_docs`
- **AND** installs each configured first-party managed skill pack under `.agents/skills/`

#### Scenario: init preserves preexisting root content outside managed block
- **WHEN** a configured root document already exists with user-authored content and does not yet contain the managed `dont` block sentinels
- **AND** the caller runs `dont init`
- **THEN** the tool inserts the managed block between the sentinels at a deterministic position
- **AND** preserves the preexisting content outside the inserted managed block exactly
