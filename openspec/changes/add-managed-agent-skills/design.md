## Context

The downloaded `dont-grill` draft is a good behavioral match for `dont`'s goal: it turns claim discipline into an agent-facing interview protocol instead of leaving the burden entirely in prose instructions. Its strongest patterns already align with the current project:

- inspect before asking
- one question per turn
- recommended answer always supplied
- explicit routing between claim/term/evidence/lifecycle branches
- glossary-aware interruption when wording drifts
- inline command crystallisation rather than vague closeout summaries

The draft does **not** fit the current implementation unchanged. Today `dont` manages only `.dont/AGENTS.md` plus shorter root managed-doc blocks. There is no managed skill install surface, no staleness check for `.agents/skills/`, and no config knob for selecting first-party skill packs. The project layout spec also currently treats writes outside `.dont/` as exceptional, with only managed docs called out.

## Goals / Non-Goals

### Goals
- Let `dont` install a managed set of first-party agent skills into `.agents/skills/`.
- Make skill installation auditable, deterministic, and repairable via the same init/doctor flow used for managed docs.
- Integrate the `dont-grill` family as the first managed pack.
- Preserve autonomy guardrails by making only the router auto-loadable and keeping sub-skills name-invoked.
- Keep user-authored skills outside the managed pack untouched.

### Non-Goals
- Do not implement a general third-party package ecosystem.
- Do not make `dont` execute skills itself; it only installs and refreshes artifacts for external agent harnesses.
- Do not specify every line of the `dont-grill` prose as immutable forever; the capability should specify required behavior and ownership, not freeze incidental wording.

## Key adjustments from the downloaded draft

1. **Canonical verb alignment**
   The installed skills must use the current canonical CLI terms from the specs: `flag` rather than `dismiss`, `lock` rather than `forget`, while still tolerating deprecated aliases in examples if needed.

2. **Managed-artifact ownership**
   The skill family must be generated and rewritten as a tool-owned pack, analogous to `.dont/AGENTS.md`, rather than copied once and left to drift.

3. **Project-layout exception**
   `.agents/skills/` must become an explicit allowed write target even though the main persistent state remains under `.dont/`.

4. **Selective overwrite**
   `dont` must only rewrite the packs it owns. User-created sibling skills in `.agents/skills/` must remain untouched.

5. **Autoload boundary**
   The router skill (`dont-grill`) is the only skill that should auto-load. Sub-skills should carry `disable-model-invocation: true`, matching the draft's intended composition model.

## Decisions

### Decision: install managed packs under `.agents/skills/`
- **Why**: This matches the harness convention the user requested and makes the skills discoverable by agent tooling without requiring the harness to read `.dont/` internals.
- **Alternative considered**: storing skills only inside `.dont/skills/`. Rejected because it would require an extra sync step or harness-specific discovery logic and would not satisfy the requested installation location.

### Decision: selection is config-driven by pack id
- **Why**: `[harness].managed_skill_packs = ["dont-grill"]` is simple, auditable, and extensible.
- **Alternative considered**: a single boolean `install_managed_skills`. Rejected because it does not scale to multiple first-party packs.

### Decision: init and doctor --fix are the writers
- **Why**: This matches the existing managed-doc model and keeps installation/repair in two already-familiar commands.
- **Alternative considered**: a separate `dont sync-skills` verb. Rejected because it repeats the same mistake already removed from doc syncing.

### Decision: preserve unmanaged siblings
- **Why**: users may already have local skills under `.agents/skills/`; `dont` should not become a destructive package manager.
- **Alternative considered**: own the entire `.agents/skills/` tree. Rejected as too invasive.

## Risks / Trade-offs

- **Risk**: writing outside `.dont/` weakens the current self-contained-state story.
  **Mitigation**: scope the exception narrowly to generated skill packs under `.agents/skills/` and document it alongside the existing managed-doc exception.

- **Risk**: managed skills may drift from the CLI/spec vocabulary.
  **Mitigation**: make them generated artifacts covered by doctor checks and refresh them from the current version's templates.

- **Risk**: users may edit managed skills directly and lose changes on repair.
  **Mitigation**: each generated skill file should declare that it is managed by `dont`; doctor should report staleness before `--fix` overwrites it.

## Open Questions

- Should `dont` write a pack-level manifest (for example under `.agents/skills/dont-grill/`) or rely purely on path ownership plus content equality? Recommendation: allow implementation to choose the lightest mechanism that supports deterministic staleness checks.
- Should the canonical `.dont/AGENTS.md` inline a short section advertising installed managed packs, or should it only point to `.agents/skills/`? Recommendation: advertise the router pack by name so agents know it exists.
