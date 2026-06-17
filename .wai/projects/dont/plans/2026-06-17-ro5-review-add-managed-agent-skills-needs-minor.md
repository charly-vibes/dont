---
tags: [pipeline-run:tdd-ro5-2026-06-17-add-managed-agent-skills, pipeline-step:review]
---

Ro5 Review - add-managed-agent-skills: NEEDS_MINOR_FIXES. HIGH: (1) refresh_managed_skill_packs leaves extra files on disk — disk_content_hash reads all files recursively, doctor never converges after version upgrade removes a sub-skill; fix by removing pack_dir files not in generated set. (2) skill_pack.rs has no unit tests — hash functions and generator untested in isolation; add 3-4 unit tests. MEDIUM: (3) PackState missing Copy derive. LOW: (4) write_canonical uses 0o600 — inconsistent with normal 0o644 skill file convention. (5) pack_content_hash lacks inter-file separator — safe for markdown, undocumented.
