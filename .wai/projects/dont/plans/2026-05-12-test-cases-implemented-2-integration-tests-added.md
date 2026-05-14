---
tags: [pipeline-run:tdd-ro5-2026-05-12-dont-4k5-12-lock-unlock-pre-conditions, pipeline-step:red]
---

test cases implemented: 2 integration tests added
- lock_nonexistent_claim_returns_not_found in tests/lock.rs — verifies claim-not-found error code and message content
- reopen_locked_claim_is_rejected in tests/reopen.rs — verifies invalid-transition error for locked entities (permanently locked)
Both pass immediately — pre-condition ordering is already correct, confirming audit verdict: SOUND
