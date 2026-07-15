---
tags: [pipeline-run:tdd-ro5-2026-07-08-dont-tp8f-silence-persistent-mode-baseline-event-warning, pipeline-step:plan]
---

F21: Silence persistent mode baseline warning

## Problem
Every command emits to stderr: 'dont: warning: could not write mode baseline event: ...'
This is noisy and erodes trust.

## Root cause
check_and_record_mode_change in src/project.rs prints eprintln! warning when
the events file write fails. Mode tracking is best-effort infrastructure.

## Fix
Replace the eprintln! with silence — suppress the warning. Mode tracking
failure has no user-facing consequence and should not pollute stderr.

## Test strategy
- Test that stderr does not contain the warning string after running a dont
  command on a cleanly-initialized project where events.jsonl is missing/empty
  (simulating the broken state).
- Use a TempDir with DONT_DIR.

## Existing tests affected
- None. No existing tests assert on this warning.
