# Change: add core dont specs

## Why

The repository's main draft, `dont-spec-v0_3_2.md`, is comprehensive but too large to use as an operational specification. We need smaller OpenSpec capabilities so future work can reason about `dont` incrementally, validate changes precisely, and evolve the design without editing one monolithic document.

## What Changes
- Add initial OpenSpec capabilities for the core `dont` model.
- Capture the status lattice and primary entity semantics as capability specs.
- Capture the CLI core verbs as capability specs.
- Establish the first decomposition boundary so later changes can continue splitting the remaining sections.
- Make the capability boundary explicit: core purpose/invariants, lifecycle semantics, and CLI command contracts are split into separate specs.

## Deferred
- Lifecycle-adjacent verbs such as `lock`, `reopen`, `ignore`, and `verify-evidence`
- Modes, initialization, and seed vocabulary
- Data model, evidence/import details, and advanced envelope behavior
- Integration and operational sections outside the first core split

## Traceability
- `dont-core` is sourced mainly from sections 1-3 of `dont-spec-v0_3_2.md`
- `dont-status-lifecycle` is sourced mainly from section 5.1 plus related lifecycle material
- `dont-cli-core` is sourced mainly from section 9 and references lifecycle behavior instead of duplicating it

## Impact
- Affected specs: `dont-core`, `dont-status-lifecycle`, `dont-cli-core`
- Affected docs: `dont-spec-v0_3_2.md`, `openspec/project.md`
- Affected workflow: future `dont` design work can target individual capabilities instead of the monolith
