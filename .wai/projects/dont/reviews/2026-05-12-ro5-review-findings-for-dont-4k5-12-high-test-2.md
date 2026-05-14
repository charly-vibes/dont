---
reviews: 2026-05-12-test-cases-implemented-2-integration-tests-added.md
tags: [pipeline-run:tdd-ro5-2026-05-12-dont-4k5-12-lock-unlock-pre-conditions, pipeline-step:review]
---

ro5 review findings for dont-4k5.12

HIGH: Test 2 (reopen_locked_claim_is_rejected) does not verify that forget actually produced a locked status before testing reopen. If forget succeeds with unexpected state, the reopen rejection could be for the wrong reason. Fix: capture forget output and assert status == locked.

MEDIUM: Test 2 missing assert_eq!(v[ok], false) for parity with test 1 and all other reopen.rs tests. Fix: add the assertion.

MEDIUM: seed_verified_claim_with_evidence and seed_assessed_hypotheses are now duplicated in lock.rs and reopen.rs. Fix: extract to tests/common.rs module. Deferred as separate refactor ticket.

LOW: Neither test captures JSON in eprintln on failure. Minor debuggability concern, omitted for consistency with codebase style.
