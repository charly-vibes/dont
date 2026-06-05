---
reviews: 2026-06-05-ro5-review-of-exit-codes-rs-json-flag-coverage-r.md
verdict: pass-with-findings
tags: [pipeline-run:tdd-ro5-2026-06-05-cli-consistency-exit-codes-json-flag, pipeline-step:review]
---

Ro5 review of exit_codes.rs / json_flag_coverage.rs / main.rs. 5-wave pass, converged.

HIGH (1): main.rs:5452-5455,5462-5464 — dont help error paths bypass emit_error_and_exit; --json silently ignored for unknown-howto and unknown-cmd errors. Machine consumers receive plain stderr instead of an error envelope.

MEDIUM (2): json_flag_coverage.rs:91 — prime_json_emits_prime_envelope omits .success() assertion; test passes even when prime exits 1. exit_codes.rs — no test for prime exits 1 when blockers present (specified pipeline-signalling behavior).

LOW (3): assert_valid_envelope does not verify error is null on success; missing exit-code tests for why/trace/atom; missing --json tests for atom/ground.

Action required: fix HIGH before ship. MEDIUM items should be addressed in fix step.
