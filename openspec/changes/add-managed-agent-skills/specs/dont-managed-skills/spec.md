## ADDED Requirements
### Requirement: Managed first-party skill packs
The system SHALL support first-party, tool-managed agent skill packs installed under the project root `.agents/skills/` directory. A managed skill pack SHALL be selected by configuration, rendered deterministically from the current `dont` version's templates, and treated as a tool-owned artifact comparable to the managed documentation surfaces.

#### Scenario: init installs configured managed skill pack
- **WHEN** `[harness].managed_skill_packs` includes `"dont-grill"` and the caller runs `dont init`
- **THEN** the project root contains the rendered `dont-grill` managed skill pack under `.agents/skills/`
- **AND** the installed contents come from the current tool-managed templates

#### Scenario: doctor fix repairs stale managed skill pack
- **WHEN** a configured managed skill pack under `.agents/skills/` is missing or has drifted from the generator output
- **AND** the caller runs `dont doctor --fix`
- **THEN** the tool rewrites that managed pack to match the current generator output

### Requirement: Managed pack ownership boundary
The system SHALL rewrite only the managed skill packs it owns. Unmanaged files or sibling skill directories under `.agents/skills/` that are not part of a configured first-party managed pack MUST be preserved byte-for-byte by `dont init` and `dont doctor --fix`.

#### Scenario: unmanaged sibling skill is preserved
- **WHEN** `.agents/skills/custom-local-skill/SKILL.md` exists and is not part of any configured managed pack
- **AND** the caller runs `dont doctor --fix`
- **THEN** that unmanaged sibling skill remains unchanged

### Requirement: dont-grill pack structure
The system SHALL provide `dont-grill` as the initial first-party managed skill pack. The pack MUST include one router entry point named `dont-grill` and named sub-skills for conclude, define, flag, trust, lock, ignore, trace, scenarios, and conclude-worthiness. The router MUST be the only auto-loadable entry point in the pack; sub-skills MUST be installed as name-invoked helpers rather than peer auto-load entry points.

#### Scenario: dont-grill installs router and named sub-skills
- **WHEN** the `dont-grill` managed pack is installed
- **THEN** the pack includes a `dont-grill` router skill
- **AND** it includes sub-skills for conclude, define, flag, trust, lock, ignore, trace, scenarios, and conclude-worthiness
- **AND** the sub-skills are marked as non-auto-loadable

### Requirement: Skill content aligns with canonical CLI vocabulary
Managed first-party skill packs SHALL use the current canonical CLI vocabulary from the active `dont` specs and help surfaces. Generated skill content MUST prefer canonical verbs such as `flag` and `lock`, while any deprecated aliases remain optional explanatory compatibility notes rather than the primary wording.

#### Scenario: generated skill text prefers canonical verbs
- **WHEN** the tool renders the `dont-grill` managed pack
- **THEN** the generated content uses `dont flag` rather than `dont dismiss` as the primary verification verb
- **AND** it uses `dont lock` rather than `dont forget` as the primary lifecycle verb
