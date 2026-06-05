# Change: Add managed agent skill packs

## Why
`dont` already manages agent-facing documentation, but it does not yet manage executable skill artifacts that help an agent follow the guardrails autonomously. The `dont-grill` draft in `~/Downloads/dont-grill-skills.md` is a strong fit for the current direction, but it needs integration points: a managed install surface, repair/staleness checks, and alignment to the current canonical CLI vocabulary and project layout.

## What Changes
- Add a new `dont-managed-skills` capability for first-party, tool-managed skill packs installed under the project root `.agents/skills/` directory.
- Introduce the first managed pack, `dont-grill`, as a router plus named sub-skills that guide agents through claim, term, evidence, doubt, lock, ignore, trace, scenario, and conclude-worthiness interviews.
- Extend `dont init` and `dont doctor --fix` to install or refresh configured managed skill packs.
- Extend `dont doctor` with managed-skill staleness/repair reporting.
- Extend project configuration so `[harness]` can declare which managed skill packs are installed.
- Update project-layout and agent-help specs so `.agents/skills/` is an explicit managed exception to the otherwise `.dont/`-local state model.

## Impact
- Affected specs:
  - `dont-managed-skills` (new)
  - `dont-project-layout`
  - `dont-project-config`
  - `dont-init-modes`
  - `dont-derived-queries`
  - `dont-agent-help`
- Affected code:
  - `src/project.rs`
  - `src/config.rs`
  - `src/managed_block.rs` or new managed-skill generator helpers
  - `src/main.rs` doctor/init wiring
  - tests covering init/doctor/config/managed artifacts
