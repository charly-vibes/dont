---
tags: [pipeline-run:tdd-ro5-2026-07-10-dont-tcy0-1-allow-staged-but-uncommitted-files-as-evidence]
---

Created OpenSpec change add-log-sync for event-log export/import. Three spec deltas: new dont-log-sync capability, modified dont-project-layout (events.jsonl entry), modified dont-init-modes (git scaffolding on init). Design preserves source tx values, uses event ULID for idempotence, recommends merge=union for JSONL. Deferred global tx ordering to follow-up.
