---
date: 2026-04-20T19:10:00-03:00
git_commit: 03aefea
branch: main
directory: /var/home/sasha/para/areas/dev/gh/charly/dont
issue: dont-71y / dont-edn
status: handoff
---

# Handoff: operational specs split complete, continue decomposing dont spec

## Context

This repo is being converted from the monolithic draft `dont-spec-v0_3_2.md` into smaller OpenSpec capabilities. The setup, research import, first core capability split, and second operational capability split are already committed.

The latest completed work is the operational split for initialization, seed vocabulary, modes, and lifecycle-adjacent verbs. The current OpenSpec decomposition now covers: core purpose/invariants, status lifecycle, four primary CLI verbs, init/modes/seed behavior, and lifecycle verbs. The next major decomposition targets are envelopes/error contracts and then data model/import behavior.

## Current Status

### Completed
- [x] Imported root design docs into `wai` research artifacts under `.wai/projects/dont/research/`
- [x] Populated OpenSpec project context in `openspec/project.md`
- [x] Added core OpenSpec change in `openspec/changes/add-core-dont-specs/`
- [x] Added operational OpenSpec change in `openspec/changes/add-dont-operational-specs/`
- [x] Tightened the operational specs after Rule-of-5 review; current HEAD already contains those fixes
- [x] Validated OpenSpec changes with `openspec validate ... --strict`

### In Progress
- [ ] Continue splitting `dont-spec-v0_3_2.md` into the next capability batch

### Planned
- [ ] Add capability specs for envelopes, JSON contracts, and error taxonomy
- [ ] Add capability specs for data model, evidence, and import behavior
- [ ] Later split derived commands, spawn/orchestration, and diagnostics

## Critical Files

1. `openspec/changes/add-core-dont-specs/specs/dont-cli-core/spec.md:1` - Current baseline for `conclude`, `define`, `trust`, and `dismiss`
2. `openspec/changes/add-core-dont-specs/specs/dont-status-lifecycle/spec.md:1` - Status lattice, stale cascade, terminal states, and audit context
3. `openspec/changes/add-dont-operational-specs/specs/dont-init-modes/spec.md:3` - Init semantics, permissive default, seed snapshot authority, canonical seed terms, and mode events
4. `openspec/changes/add-dont-operational-specs/specs/dont-lifecycle-verbs/spec.md:3` - `lock`, `reopen`, `ignore`, and `verify-evidence` command contracts
5. `dont-spec-v0_3_2.md:265` - Source sections for seed vocabulary, modes, lifecycle verbs, and next unsplit material starting at derived commands/envelope
6. `openspec/project.md:1` - Current project-level OpenSpec context and decomposition strategy

## Recent Changes

- `openspec/changes/add-dont-operational-specs/specs/dont-init-modes/spec.md:3` - Tightened init to persistent project-local state, permissive default, authoritative seed snapshot, and explicit ten-term seed vocabulary
- `openspec/changes/add-dont-operational-specs/specs/dont-lifecycle-verbs/spec.md:3` - Added explicit lock gate, already-locked refusal, ignore hedge-only refusal, structural failures for `verify-evidence`, and bounded network politeness requirement
- `openspec/changes/add-dont-operational-specs/proposal.md:7` - Added sequencing rationale for why this batch comes before envelopes/imports
- `openspec/changes/add-dont-operational-specs/design.md:24` - Added explicit next split order

## Key Learnings

1. Capability boundaries work best when `dont-core` stays conceptual, lifecycle stays stateful, and CLI/lifecycle-adjacent commands stay operational.
   - Evidence: `openspec/changes/add-core-dont-specs/design.md`

2. Rule-of-5 review is catching real spec weaknesses, especially vague operational terms and missing refusal paths.
   - Evidence: both operational specs were tightened around init defaults, seed authority, lock gating, and `verify-evidence` structural failures.

3. The most natural next split is not imports first, but envelopes/error contracts.
   - Evidence: `dont-spec-v0_3_2.md:556` begins the next large coherent operational surface after the completed sections.

## Open Questions

- [ ] Should envelopes and error taxonomy be one capability or split into separate `dont-envelope-contract` and `dont-errors` capabilities?
- [ ] How much of the Cozo/storage detail belongs in OpenSpec versus later design notes?
- [ ] Should import behavior and evidence/data model live together or be separated into `dont-imports` and `dont-entities-evidence`?

## Next Steps

1. Read `dont-spec-v0_3_2.md` from section 10 onward and scaffold the next OpenSpec change for envelope/error behavior.
2. Create or update a `bd` task under epic `dont-71y` for the next split batch if you want issue-level traceability.
3. Validate every new change with `openspec validate <change-id> --strict` and optionally run Rule-of-5 again before committing.

## Artifacts

New files:
- `openspec/changes/add-dont-operational-specs/proposal.md`
- `openspec/changes/add-dont-operational-specs/tasks.md`
- `openspec/changes/add-dont-operational-specs/design.md`
- `openspec/changes/add-dont-operational-specs/specs/dont-init-modes/spec.md`
- `openspec/changes/add-dont-operational-specs/specs/dont-lifecycle-verbs/spec.md`
- `handoffs/2026-04-20_19-10-00_operational-specs-next-splits.md`

Modified files:
- `openspec/project.md`
- `.wai/projects/dont/.state`
- `.wai/projects/dont/research/*`

## Related Links

- `openspec/changes/add-core-dont-specs/`
- `openspec/changes/add-dont-operational-specs/`
- `dont-spec-v0_3_2.md`
- `bd show dont-71y`
- `bd show dont-edn`

## Additional Context

Current git HEAD is `03aefea` (`docs(spec): add operational dont capability specs`). At handoff time, there are no tracked uncommitted changes. Remaining untracked root docs are the original research/spec source files and `.agents/`; they were intentionally left out of the recent commits.
