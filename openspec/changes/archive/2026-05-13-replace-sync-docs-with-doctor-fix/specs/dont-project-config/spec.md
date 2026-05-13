## MODIFIED Requirements

### Requirement: Harness configuration surface

The system SHALL expose a `[harness]` block including the managed-doc targets and the spawn-timeout window. Managed-doc targets MUST determine which root files may receive the shorter `dont` block (those files become inputs to `dont doctor`'s managed-docs check and to `dont doctor --fix`), and the timeout window MUST inform `spawn_request.expires_at` computation.

#### Scenario: managed docs list targets root files

- **WHEN** `[harness].managed_docs` lists `AGENTS.md` and `CLAUDE.md`
- **THEN** those files are eligible targets for `dont doctor --fix` and for the managed-docs staleness check run by `dont doctor`

#### Scenario: spawn timeout config drives expiry

- **WHEN** `[harness].spawn_timeout_hours = 24`
- **THEN** newly issued spawn requests expire twenty-four hours after issuance unless resolved earlier
