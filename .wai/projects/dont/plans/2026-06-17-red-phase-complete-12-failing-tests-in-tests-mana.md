---
tags: [pipeline-run:tdd-ro5-2026-06-17-add-managed-agent-skills, pipeline-step:red]
---

Red phase complete: 12 failing tests in tests/managed_skills.rs covering config parsing, init install, doctor stale/missing/pass reporting, doctor --fix repair and idempotency, ownership boundary (unmanaged sibling preservation), pack content (router + 9 sub-skills), canonical verbs, and sub-skill subdirectory placement. 1 test passes (init_skips_skill_packs_when_not_configured). Existing tests unaffected.
