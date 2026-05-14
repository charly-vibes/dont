---
tags: [pipeline-run:tdd-ro5-2026-05-13-implement-managed-docs-refresh-via-doctor-fix-dont-ymdq-replace-sync-docs-with-doctor-fix, pipeline-step:red]
---

Red phase complete for managed docs refresh: added tests/managed_docs.rs with 5 failing integration cases covering (1) init creates canonical .dont/AGENTS.md plus root AGENTS.md/CLAUDE.md managed blocks, (2) init preserves existing root content outside injected managed block, (3) doctor reports pass on fresh init, (4) doctor ignores whitespace-only drift but warns on real root-block edits with actionable detail to run dont doctor --fix, and (5) doctor --fix repairs stale/missing root and canonical docs and is idempotent. Verified new tests fail and existing init suite still passes unchanged.
