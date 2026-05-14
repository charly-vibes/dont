---
tags: [pipeline-run:tdd-ro5-2026-05-12-dont-4k5-12-lock-unlock-pre-conditions, pipeline-step:plan]
---

dont-4k5.12 lock/unlock pre-conditions audit plan

Audit scope: verify that all pre-condition checks happen before store mutation in the lock/forget command.

Finding: Implementation is SOUND — all 5 pre-condition gates (entity type, existence, current status, lockable rule, dependency status) are checked before append_status_change() is called at line 2733 main.rs. Error codes are distinct and descriptive.

Test gaps found (3):
1. Lock non-existent claim — no test for claim-not-found code path (line 2663 main.rs)
2. Reopen locked claim — no test that model_reopen(Locked) returns invalid-transition (model.rs:36,80)
3. Lock with unresolved dependency — lockable gate references this but no explicit test

Test strategy: integration tests in tests/lock.rs (matching existing style). Each test: create minimal store state, run dont forget/dont reopen, assert exit code + JSON error code.

Tests to write:
- lock_nonexistent_claim_returns_not_found
- reopen_locked_claim_is_rejected
- lock_with_unresolved_dependency_blocked_by_lockable_gate

All are integration tests; no production code changes expected.
