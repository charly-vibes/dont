# Change: Replace `dont sync-docs` with `dont doctor --fix` and tighten managed-block contract

## Why

`dont sync-docs` is a single-purpose verb that overlaps with `dont doctor`'s existing role as the project health command. Today `doctor` reports problems but cannot resolve them, so agents and operators must remember a separate verb to refresh the managed documentation block. Folding the managed-doc refresh into `doctor --fix` gives operators one diagnostic command with two modes (report, repair) and removes a verb that does not pull its weight.

This change also tightens the contract for what goes inside the root managed block. The block must be **minimal** (a pointer to `.dont/AGENTS.md`, not a copy of it) but **noisy enough that an LLM agent reading `CLAUDE.md` or `AGENTS.md` cannot miss the orientation step**. Today the spec only says "shorter pointer"; in practice that has produced quiet pointers that agents skim past.

`wai` (see `../wai/src/managed_block.rs` and `../wai/src/commands/doctor.rs`) already implements this pattern: sentinel-bounded block + content-equality staleness check + fix via the doctor command. We are codifying that pattern for `dont`.

## What Changes

- **BREAKING**: Remove the `dont sync-docs` verb. The managed-block refresh becomes a check inside `dont doctor` with a repair path via `dont doctor --fix`.
- Strengthen the managed-block content contract in `dont-agent-help`: the root block MUST contain prominent "do not edit" markers, a session-start command (`dont prime --json`), and an explicit pointer to `.dont/AGENTS.md` as the canonical document.
- Add a managed-block staleness check to `dont doctor`. The check MUST use content-equality (regenerated expected content vs. the bytes between markers on disk), not version strings or hashes.
- Add `--fix` to `dont doctor`. When passed, doctor MUST rewrite any stale managed blocks and overwrite `.dont/AGENTS.md`. Without `--fix`, doctor is read-only.
- Make `dont init` responsible for the first injection of the root managed block and for writing `.dont/AGENTS.md`. `init` and `doctor --fix` MUST produce byte-identical results for the same project state.
- Update `dont-project-config` so the `[harness].managed_docs` list documents itself as the input to `doctor --fix` rather than `sync-docs`.

## Impact

- Affected specs:
  - `dont-agent-help` — REMOVE the `dont sync-docs` requirement; MODIFY the managed-block requirement to specify minimal-but-noisy content; ADD a staleness-check requirement and a `doctor --fix` repair requirement.
  - `dont-derived-queries` — MODIFY the diagnostic-queries requirement so `doctor` includes the managed-docs staleness check and exposes `--fix`.
  - `dont-project-config` — MODIFY the harness-config requirement so managed-doc targets reference `doctor --fix`.
  - `dont-init-modes` — MODIFY the init requirement so it injects the root managed block and writes `.dont/AGENTS.md`.
- Affected code (planned, not part of this proposal):
  - New `src/managed_block.rs` mirroring `../wai/src/managed_block.rs`.
  - `dont init` wires in block injection + `.dont/AGENTS.md` write.
  - `dont doctor` gains a managed-docs check and a `--fix` flag.
- Affected docs:
  - `.dont/AGENTS.md` becomes a fully-managed file with a "managed by `dont init` / `dont doctor --fix`" header.
  - Root `CLAUDE.md` / `AGENTS.md` host only the pointer block between `<!-- DONT:START -->` / `<!-- DONT:END -->`.
