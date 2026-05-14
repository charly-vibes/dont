---
tags: [pipeline-run:tdd-ro5-2026-05-12-dont-4k5-11-mode-enforcement-boundary-audit, pipeline-step:plan]
---

dont-4k5.11 mode enforcement boundary audit

## Audit Findings

Mode is represented as ProjectMode enum, persisted in config.toml, read via project.mode(). One operation is currently mode-gated: conclude --depends-on with unresolved term references.

The gate is correctly placed: pre-validation before claim creation, reads persisted state (not a default), and emits a clear error naming the mode and unresolved terms.

5 existing tests in tests/mode_enforcement.rs cover the conclude gate adequately.

## Gap Found

The rules engine has a second mode gate at rules/mod.rs line ~111: ungrounded rule severity is promoted to strict in strict mode. This behavior has NO dedicated tests.

## Test Plan

Add tests to tests/mode_enforcement.rs verifying:
1. ungrounded rule in permissive mode → warning severity (exit 0, non-empty stderr)
2. ungrounded rule in strict mode → error/failure (exit 1)
3. switching mode mid-session is reflected immediately in subsequent rule evaluation
4. mode check reads persisted state (not in-memory default) — verify by writing mode directly to config.toml then running command without re-initializing

## Pass Criteria (from ticket)

- Every mode-gated op checks current project mode before executing ✓ (verified)
- Strict mode rejects restricted ops with message naming the mode ✓ (verified)
- Switching mode reflected immediately ✓ (need test for rules engine path)
