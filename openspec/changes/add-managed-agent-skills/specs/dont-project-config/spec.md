## MODIFIED Requirements
### Requirement: Harness configuration surface

The system SHALL expose a `[harness]` block including the managed-doc targets, the managed first-party skill-pack selection, and the spawn-timeout window. Managed-doc targets MUST determine which root files may receive the shorter `dont` block (those files become inputs to `dont doctor`'s managed-docs check and to `dont doctor --fix`), managed skill-pack selection MUST determine which first-party packs are installed or repaired under `.agents/skills/`, and the timeout window MUST inform `spawn_request.expires_at` computation.

#### Scenario: managed docs list targets root files

- **WHEN** `[harness].managed_docs` lists `AGENTS.md` and `CLAUDE.md`
- **THEN** those files are eligible targets for `dont doctor --fix` and for the managed-docs staleness check run by `dont doctor`

#### Scenario: managed skill packs select installed router family
- **WHEN** `[harness].managed_skill_packs = ["dont-grill"]`
- **THEN** `dont init` and `dont doctor --fix` treat `dont-grill` as an installed, repairable first-party managed pack under `.agents/skills/`

#### Scenario: spawn timeout config drives expiry

- **WHEN** `[harness].spawn_timeout_hours = 24`
- **THEN** newly issued spawn requests expire twenty-four hours after issuance unless resolved earlier
