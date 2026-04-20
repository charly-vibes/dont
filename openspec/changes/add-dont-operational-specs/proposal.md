# Change: add dont operational specs

## Why

The initial OpenSpec split covers the core purpose, lifecycle model, and four primary verbs, but important operational behaviour still lives only in `dont-spec-v0_3_2.md`. To continue decomposing the monolith, the next change should capture project initialization, seed vocabulary, mode handling, and lifecycle-adjacent verbs as focused capabilities.

## What Changes
- Add a capability for project initialization, seed vocabulary, and mode handling.
- Add a capability for lifecycle-adjacent verbs: `lock`, `reopen`, `ignore`, and `verify-evidence`.
- Preserve the separation between core CLI verbs and operational/lifecycle commands.
- Sequence this batch before envelopes, imports, and lower-level data-model work because these operational behaviors sit closest to the already-split core interaction model.

## Deferred
- Derived commands and envelope contracts
- Data model and import-specific capabilities
- Rule-engine, spawn, and operational diagnostics sections

## Traceability
- `dont-init-modes` is sourced mainly from sections 4.4, 7, and 8 of `dont-spec-v0_3_2.md`
- `dont-lifecycle-verbs` is sourced mainly from section 9A of `dont-spec-v0_3_2.md`

## Impact
- Affected specs: `dont-init-modes`, `dont-lifecycle-verbs`
- Affected docs: `dont-spec-v0_3_2.md`, `openspec/project.md`
- Affected workflow: future decomposition can build on explicit operational specs instead of the monolith
