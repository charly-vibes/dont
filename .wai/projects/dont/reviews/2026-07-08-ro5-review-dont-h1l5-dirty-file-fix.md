# Rule of 5 Review — F21: dirty-file evidence acceptance

**Reviewed:** `dont flag --file` with dirty tracked files
**Verdict:** READY
**Convergence:** Stage 5

## Summary

- **CRITICAL:** 0
- **HIGH:** 0
- **MEDIUM:** 0
- **LOW:** 2

## Stage 1: DRAFT — GOOD

The change is well-scoped: a single branch replacement in `check_git_provenance()`.
Remove the error exit for dirty files, instead call `git hash-object` to compute
a content-based provenance ref. Two tests cover the happy path. No architectural concerns.

## Stage 2: CORRECTNESS — EXCELLENT

- Git command follows the same `env_remove` pattern as surrounding calls
- `git:content:` prefix cleanly distinguishes content-based refs from commit-based (`git:`)
- Fallback to `None` on hash-object failure matches existing non-git-repo pattern
- Tests verify exact SHA matching via `git hash-object` from test itself

## Stage 3: CLARITY — GOOD

- Code is straightforward: branch replaces `emit_error_and_exit` with hash-object call
- Brief comment explains intent
- Test names clearly communicate what they test
- `_cmd_prefix` suppression is a minor cosmetic side-effect

## Stage 4: EDGE CASES — GOOD

- Non-git repo or git unavailable: returns `None` (existing pattern)
- hash-object fails (file deleted between status check and hash): returns `None`, caller falls through to `unreadable-evidence`
- Binary files: hash-object handles fine (content-agnostic)
- Special characters in path: `rel` variable passed directly to git
- Very large files: hash-object streams; may be slow but won't OOM

## Stage 5: EXCELLENCE — GOOD

Minimal and surgical change. Both tests are deterministic.

## Findings

1. [LOW] `_cmd_prefix` unused except as parameter — harmless, internal function
2. [LOW] No explicit negative assertion that clean files never get `git:content:` — implicitly covered by existing `flag_file_locator_includes_commit_ref` test