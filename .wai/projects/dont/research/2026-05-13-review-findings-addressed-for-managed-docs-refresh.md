---
tags: [pipeline-run:tdd-ro5-2026-05-13-implement-managed-docs-refresh-via-doctor-fix-dont-ymdq-replace-sync-docs-with-doctor-fix, pipeline-step:fix]
---

Review findings addressed for managed docs refresh: doctor now verifies seed snapshot presence instead of hard-coding pass and emits a warn with dont init remediation when .dont/seed/dont-seed.yaml is missing; added managed_docs test coverage for that warning path; clarified the direct DONT_DIR compatibility path in Project::uses_separate_root_docs and surfaced it in doctor pending_spawns detail for standalone state-directory mode. Full cargo test remains green.
