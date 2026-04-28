# Change: add core dont specs

## Why

The repository's main draft, `dont-spec-v0_3_2.md`, is comprehensive but too large to use as an operational specification. We need smaller OpenSpec capabilities so future work can reason about `dont` incrementally, validate changes precisely, and evolve the design without editing one monolithic document.

## What Changes
- Add initial OpenSpec capabilities for the core `dont` model.
- Capture the core purpose and invariants around standalone operation, append-only history, explicit epistemic status, and claims/terms as the only initial first-class entities.
- Capture a shared persisted lifecycle model for claims and terms, while treating dependency fallout as computed trace analysis rather than a stored `stale` status.
- Capture the CLI core verbs as phrase-interpreted commands (`dont <verb>`), including `dont trust` as explicit doubt registration.
- Capture strict deduplication for claim and term creation while preserving append-only enrichment such as redefinition and additional evidence.
- Establish the first decomposition boundary so later changes can continue splitting the remaining sections.
- Make the capability boundary explicit: core purpose/invariants, lifecycle semantics, and CLI command contracts are split into separate specs.

## Deferred
- Lifecycle-adjacent verbs such as `lock`, `reopen`, `ignore`, and `verify-evidence`
- Modes, initialization, and seed vocabulary
- Data model, evidence/import details, and advanced envelope behavior
- Integration and operational sections outside the first core split

## Traceability
- `dont-core` is sourced mainly from sections 1-3 of `dont-spec-v0_3_2.md`
- `dont-status-lifecycle` is sourced mainly from section 5.1 plus related lifecycle material, but narrows the persisted lattice to explicit statuses and moves dependency fallout into computed trace assessment
- `dont-cli-core` is sourced mainly from section 9 and references lifecycle behavior instead of duplicating it, while clarifying phrase-form command semantics and strict duplicate refusal

## Impact
- Affected specs: `dont-core`, `dont-status-lifecycle`, `dont-cli-core`
- Affected docs: `dont-spec-v0_3_2.md`, `openspec/project.md`
- Affected workflow: future `dont` design work can target individual capabilities instead of the monolith
