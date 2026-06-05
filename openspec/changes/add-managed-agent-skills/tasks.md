## 1. Spec and design
- [ ] 1.1 Add `dont-managed-skills` capability deltas covering managed pack installation, pack ownership, and the `dont-grill` router/sub-skill family.
- [ ] 1.2 Modify `dont-project-layout`, `dont-project-config`, `dont-init-modes`, `dont-derived-queries`, and `dont-agent-help` to reference managed skill packs.
- [ ] 1.3 Validate the proposal with `openspec validate add-managed-agent-skills --strict`.

## 2. TDD plan
- [ ] 2.1 Red: add failing tests for `dont init` creating configured managed skill packs under `.agents/skills/` without disturbing unrelated sibling skills.
- [ ] 2.2 Red: add failing tests for `dont doctor --json` reporting stale or missing managed skill packs.
- [ ] 2.3 Red: add failing tests for `dont doctor --fix` repairing managed skill packs and remaining idempotent.
- [ ] 2.4 Red: add failing tests for config parsing/defaults around `[harness].managed_skill_packs`.

## 3. Implementation
- [ ] 3.1 Add config support for managed skill pack selection.
- [ ] 3.2 Add generator/rendering helpers for first-party managed skill packs, starting with `dont-grill`.
- [ ] 3.3 Extend project init/repair paths to install or refresh managed skill packs.
- [ ] 3.4 Extend doctor checks to detect stale, missing, or partially edited managed skill packs.
- [ ] 3.5 Ensure only tool-owned managed packs are rewritten; preserve unmanaged sibling skills.

## 4. Tidy and docs
- [ ] 4.1 Refactor any shared managed-artifact code so docs and skills use the same comparison/repair patterns where sensible.
- [ ] 4.2 Update canonical agent-help output to mention the installed managed router skill.
- [ ] 4.3 Run repo checks and strict OpenSpec validation.
