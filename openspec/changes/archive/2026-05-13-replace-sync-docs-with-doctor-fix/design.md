## Context

`dont` is designed to force LLM agents into a discipline of grounding claims before asserting them. Two surfaces enforce that discipline:

1. The CLI refuses or warns when claims, terms, or evidence are missing.
2. The agent-facing documentation (`AGENTS.md`, `CLAUDE.md`, `.dont/AGENTS.md`) teaches the agent what `dont` expects.

Today (2026-05-13), the doc surface is governed by a single verb, `dont sync-docs`, which rewrites a sentinel-bounded managed block in configured root files. The block content is described only as a "shorter pointer to `.dont/AGENTS.md`" — there is no contract on prominence, and there is no machine check for whether the on-disk block matches what the current `dont` version would emit.

`wai` solved the same problem (see `../wai/src/managed_block.rs` and `../wai/src/commands/doctor.rs`):
- Sentinel pair `<!-- WAI:START -->` / `<!-- WAI:END -->`.
- Procedural content generator (no `include_str!` template) keyed off detected project state.
- Staleness detection by **content equality**: regenerate expected content, compare against bytes between markers, warn if different.
- Repair routed through the same doctor command (`wai doctor` reports; `wai init` rewrites).

This change codifies the same pattern for `dont`, with three changes from the `wai` baseline:
- Repair lives behind an explicit `--fix` flag on `doctor` rather than a separate `init` re-run. This keeps `init` as a one-shot project bootstrap and makes `doctor` the single command an agent reaches for to both diagnose and remediate.
- The managed-block content contract is tightened: the spec mandates prominent warnings and an executable session-start command, not just a "pointer".
- The staleness check normalizes line endings to `\n` and trims trailing whitespace on both sides before comparing the normalized content. `wai` performs a raw byte comparison and therefore false-positives on benign editor reformatting; `dont` opts to suppress that noise at the cost of a slightly stricter generator contract (the generator MUST emit pre-normalized output so that `init` and `doctor --fix` produce no diff against a freshly-normalized comparison).

## Goals / Non-Goals

### Goals

- One verb for project health: `dont doctor` reports, `dont doctor --fix` repairs.
- Content-based staleness detection so block drift is caught even when no version number changed.
- Minimal but noisy root block: short enough that operators tolerate it in their `CLAUDE.md`, prominent enough that an LLM agent reading the file performs `dont prime --json` before proceeding.
- `.dont/AGENTS.md` is canonical, fully-managed, and overwritten wholesale on `init` and `doctor --fix`.
- `dont init` and `dont doctor --fix` produce byte-identical output for the same *detected project state* — meaning the inputs the generator reads (configured `managed_docs` targets, current `dont` version, installed rules, project mode). Mutations to detected state between calls will produce different output; that is expected and desired.

### Non-Goals

- A backwards-compatible `dont sync-docs` alias. There is no implementation yet (`dont` is spec-stage); we are not maintaining compatibility with a non-existent installed base.
- A general-purpose doc-templating system. The block content is procedural Rust string assembly keyed off detected state, not a user-editable template.
- Embedding the full canonical instructions in the root block. Operators MAY remove the block and rely only on `.dont/AGENTS.md`; `doctor` will not refuse the project, only warn.
- Auto-running `--fix` from a hook or pre-commit. `--fix` is operator-invoked.

## Decisions

### Decision 1: Repair flag lives on `doctor`, not as a separate `sync` verb

- **What**: Add `--fix` to `dont doctor`. Remove `dont sync-docs`.
- **Why**: A separate verb for "rewrite the block" is a verb the operator must remember. `doctor` already owns the diagnostic surface; pairing it with a repair flag matches the mental model "show me problems / now fix them".
- **Alternatives considered**:
  - Keep `sync-docs` as the imperative refresh verb. Rejected: redundant with `doctor` once the staleness check exists.
  - Put repair on `init --refresh`. Rejected: muddles `init`'s "first-run" semantics and forces operators to know which command to use for which scenario.
- **Trade-off**: `doctor --fix` performs a side effect, which is a small departure from the principle that diagnostic commands are read-only. We accept this because the alternative is two commands for one user goal. Default `doctor` remains read-only.

### Decision 2: Content-equality, not hashes or version pins

- **What**: Staleness check regenerates expected content from current project state and compares against the bytes between markers on disk.
- **Why**: Matches `wai`'s approach (`../wai/src/commands/doctor.rs:1790-1864`, `check_managed_block_staleness()`) and avoids the bookkeeping of versioned hashes. The generator code is the source of truth; if it changes, every project's block becomes stale, which is what we want.
- **Alternatives considered**:
  - Embed a version stamp in the block (e.g. `<!-- DONT:v0.3.2 -->`). Rejected: forces a version bump on every doc change and produces false-clean signals when the version is unchanged but the content drifted.
  - Hash-based check stored in `config.toml`. Rejected: extra state to maintain, redundant with the regen-and-compare approach.
- **Trade-off**: Regenerating expected content on every `doctor` run is a small CPU cost. Negligible for a single string assembly.

### Decision 3: Two-tier surface (block in root, full file in `.dont/`)

- **What**: Keep `dont-project-layout`'s existing rule: `.dont/AGENTS.md` is canonical and fully-managed; root `AGENTS.md` / `CLAUDE.md` host a short managed block that points to it.
- **Why**: Operators host other content in `CLAUDE.md` (project conventions, team notes) and want `dont`'s footprint there to be small. The canonical document needs to be long, but it doesn't need to live in the user-edited file.
- **Alternatives considered**:
  - Inject the full canonical instructions into root files. Rejected: bloats user-edited docs and creates a divergence between two copies of the same content.
  - Skip the root block entirely and rely on `.dont/AGENTS.md`. Rejected: agents that read `CLAUDE.md` first (Claude Code's default) would miss `dont` entirely.

### Decision 4: "Noisy enough" is contractual, not stylistic

- **What**: The spec mandates specific content in the root block: a prominent header that signals overwrite-zone, an actionable session-start command, and the pointer.
- **Why**: The current spec says "shorter pointer", which an implementer can satisfy with a single line that an agent will skim past. The user's stated goal is that agents *follow* the block — that requires prominence the spec enforces, not relies on author taste.
- **Alternatives considered**:
  - Leave content as implementation detail. Rejected: that is the current state and it has not produced agent-followable blocks.
  - Mandate exact wording. Rejected: too rigid; future revisions of the prime command or canonical doc path would force spec changes.
- **Trade-off**: The spec dictates structure (must contain a session-start command, must contain a "do not edit" warning, must point at `.dont/AGENTS.md`) but not exact wording. Implementations have wiggle room; reviewers have something concrete to check.

### Decision 5: Sentinel choice — `<!-- DONT:START -->` / `<!-- DONT:END -->`

- **What**: HTML-comment sentinels with the tool name in uppercase. Identical shape to `wai`'s `<!-- WAI:START -->` / `<!-- WAI:END -->`.
- **Why**: HTML comments are invisible in rendered Markdown, survive most editor reformatting, and parallel the prior-art `wai` pattern an operator may already recognize.
- **Trade-off**: Tools using the same sentinel scheme will not collide because the tool name is in the marker.

## Risks / Trade-offs

- **Risk**: Operators who hand-edit the block lose their changes the next time anyone runs `doctor --fix`. → Mitigation: the block's first line is an explicit overwrite warning; the surrounding `CLAUDE.md` content is preserved verbatim; operators who want custom content put it outside the markers.
- **Risk**: `doctor --fix` running as part of an automation could clobber an unrelated in-flight edit. → Mitigation: `--fix` is operator-invoked, not automatic. No hook or pre-commit triggers it.
- **Risk**: Content-equality is sensitive to trailing-whitespace and line-ending drift. → Mitigation: the generator normalizes line endings to `\n` and trims trailing whitespace on both sides of the comparison before deciding the block is stale. This is a deliberate divergence from `wai`'s implementation, which performs a raw byte comparison (`../wai/src/commands/doctor.rs` `check_managed_block_staleness`) and therefore false-positives on benign editor reformatting. `dont` accepts a slightly stricter generator (must emit normalized output) in exchange for fewer noisy `warn` results.
- **Risk**: `.dont/AGENTS.md` being fully overwritten means user notes inside that file are lost. → Mitigation: the file's first line states it is managed; user notes belong in a sibling file (e.g. `.dont/notes.md`) or in `CLAUDE.md` outside the markers.

## Migration Plan

Because `dont` is spec-stage with no implementation yet, no live migration is required.

- Spec deltas land first (this proposal).
- Implementation work follows in a separate change set, tracked via `bd` issues that map to the tasks list and follow the project's red→green→refactor TDD cadence.
- Existing archived references to `dont sync-docs` in `openspec/changes/archive/` remain unchanged; they are historical.

## Open Questions

- Should `doctor --fix` also rewrite `.dont/AGENTS.md` unconditionally, or only when its content differs from the expected generator output? → Recommendation: only when different (same content-equality rule as the block). Confirm during implementation.
- Should `doctor` non-strict mode emit the managed-docs check as `warn` or `info`? → Recommendation: `warn`, consistent with `wai`'s behaviour and with `DoctorReport`'s existing `pass|warn|fail` status lattice. The check is actionable (`run dont doctor --fix`), and `info` would suppress it from non-strict invocations.
- Should `init` refuse to inject the block if the host file already contains a different tool's managed block at the same position? → Recommendation: no — `init` injects between its own sentinels and is agnostic to other tools' blocks. Confirm during implementation.
