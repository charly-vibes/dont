---
tags: [pipeline-run:tdd-ro5-2026-05-13-dont-4k5-8-project-initialization-error-paths, pipeline-step:fix]
---

Rule of 5 review for dont-4k5.8 (commit 0779ace)

Verified scope: src/project.rs, src/main.rs, tests/init.rs. Confirmed `cargo test --test init` passes (18/18).

Findings:
1. HIGH — init still has generic I/O reporting for managed-doc write failures. `src/project.rs:306-312` calls `write_canonical()` / `replace_or_prepend_root_block()` with `?`, which falls through `From<std::io::Error>` at `src/project.rs:63-70` and produces `ProjectError::Io` with no op/path context. Some init failures therefore still emit plain `I/O error: ...` instead of naming the failed path and operation. Recommendation: wrap managed-doc I/O with contextual `map_err`, as done in `init()` at `src/project.rs:415-425`.

2. MEDIUM — failure-mode coverage is incomplete versus the issue pass criteria. `tests/init.rs:221-345` adds re-init and one create-failure case, but there is still no integration test for malformed pre-existing config and no test for a later init write failure such as managed-doc or `.gitignore` permission denial. Recommendation: add targeted integration tests for malformed existing `config.toml` and for a post-store-open write failure path.

3. LOW — the new guard test is broader than its stated intent. `tests/init.rs:321-326` asserts `src/project.rs` contains no `.expect(` anywhere, even though the requirement is specifically init-related code paths. Useful as a coarse regression net, but brittle and not init-specific. Recommendation: narrow the assertion or rename the test to match file-wide scope.

Convergence: review converged. No critical correctness bug found in the changed logic; main remaining risk is incomplete error-context propagation plus missing failure-mode coverage.
