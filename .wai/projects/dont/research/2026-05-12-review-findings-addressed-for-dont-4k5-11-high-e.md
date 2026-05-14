---
tags: [pipeline-run:tdd-ro5-2026-05-12-dont-4k5-11-mode-enforcement-boundary-audit, pipeline-step:fix]
---

Review findings addressed for dont-4k5.11:
HIGH: extracted switch_mode(&TempDir, from, to) helper — eliminates 3 identical config mutation blocks.
MEDIUM: .expect() with context added to all fs reads/writes inside switch_mode.
MEDIUM: claim survival assertion added to test 4 (ungrounded_rule_severity_reads_persisted_mode_not_default).
LOW: deferred — test 3 rename is cosmetically minor and the current name is unambiguous enough.
All 11 tests pass.
