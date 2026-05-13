## 1. Spec deltas

- [ ] 1.1 Draft `specs/dont-agent-help/spec.md` delta: MODIFY managed-block requirement, REMOVE sync-docs requirement, ADD staleness check, ADD `doctor --fix` repair requirement.
- [ ] 1.2 Draft `specs/dont-derived-queries/spec.md` delta: MODIFY diagnostic-queries requirement to include managed-docs check and `--fix` flag.
- [ ] 1.3 Draft `specs/dont-project-config/spec.md` delta: MODIFY harness-config requirement so `[harness].managed_docs` targets reference `doctor --fix`.
- [ ] 1.4 Draft `specs/dont-init-modes/spec.md` delta: MODIFY init requirement so it injects the root managed block and writes `.dont/AGENTS.md`.
- [ ] 1.5 Run `openspec validate replace-sync-docs-with-doctor-fix --strict` and resolve every issue.

## 2. Cross-spec audit

- [ ] 2.1 `rg "sync-docs"` across `openspec/specs/` and confirm no stale references remain after the deltas archive.
- [ ] 2.2 Confirm `dont-errors` and `dont-cli-surface` exit-code semantics still apply unchanged to `doctor --fix` (no new error codes introduced).
- [ ] 2.3 Confirm `dont-payload-types` `DoctorReport` check schema already covers the new `managed_docs` check name and `pass|warn|fail` status values without requiring a schema change.

## 3. Approval gate

- [ ] 3.1 Share proposal for review; do not begin implementation until approved.

## 4. Implementation (after approval — separate beads issues)

- [ ] 4.1 Create beads epic + per-piece issues (managed-block module; init wiring; doctor check; doctor --fix). Each issue maps to a red→green→refactor TDD cycle per `CLAUDE.md`.
- [ ] 4.2 Implement `src/managed_block.rs` mirroring `../wai/src/managed_block.rs` (sentinel pair, read/inject, fully-managed file writer, line-ending normalization).
- [ ] 4.3 Wire `dont init` to inject the root managed block and write `.dont/AGENTS.md`.
- [ ] 4.4 Add managed-docs staleness check to `dont doctor`.
- [ ] 4.5 Implement `dont doctor --fix` repair path.
- [ ] 4.6 Remove any draft `dont sync-docs` plumbing.
- [ ] 4.7 Tests: unit tests for managed-block module; integration tests for `init` → `doctor` (clean) → manual edit → `doctor` (warn) → `doctor --fix` (clean) round trip.
- [ ] 4.8 Update `.dont/AGENTS.md` generator content to match the canonical orientation contract in `dont-agent-help`.

## 5. Archive

- [ ] 5.1 After implementation deploys, run `openspec archive replace-sync-docs-with-doctor-fix --yes`.
- [ ] 5.2 Run `openspec validate --strict` post-archive to confirm specs are consistent.
