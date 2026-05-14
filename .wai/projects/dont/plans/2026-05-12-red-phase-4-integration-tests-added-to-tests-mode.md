---
tags: [pipeline-run:tdd-ro5-2026-05-12-dont-4k5-11-mode-enforcement-boundary-audit, pipeline-step:red]
---

Red phase: 4 integration tests added to tests/mode_enforcement.rs

1. ungrounded_rule_reports_warn_severity_in_permissive_mode — verifies rules test ungrounded returns severity:warn in permissive mode
2. ungrounded_rule_reports_strict_severity_in_strict_mode — verifies mode switch from permissive to strict reflected in rules engine output
3. mode_switch_from_strict_to_permissive_changes_ungrounded_severity — verifies severity change is immediate after config.toml edit
4. ungrounded_rule_severity_reads_persisted_mode_not_default — verifies engine reads config.toml not a hardcoded default

All 4 tests pass immediately: code is already correct. Audit outcome: mode enforcement via rules engine is sound.
