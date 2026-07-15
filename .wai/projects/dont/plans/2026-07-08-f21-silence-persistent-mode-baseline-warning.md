---
tags: [pipeline-run:tdd-ro5-2026-07-08-dont-qau6-allow-prose-punctuation-in-dont-conclude-statement-text]
---

F21: Silence persistent mode baseline warning

## Problem
Every command emits: 'dont: warning: could not write mode baseline event: ...'
This is noisy and erodes trust — users assume something is broken.

## Root cause
check_and_record_mode_change() in src/project.rs prints an eprintln! warning
when the events file write fails. Mode tracking is best-effort infrastructure,
not user-facing diagnostics.

## Fix
Replace eprintln! with a no-op (suppress the warning). The warning is about
internal bookkeeping that has no user-facing consequence.

## Test strategy
- The warning goes to stderr. Test that stderr does not contain the warning
  string after running a dont command on a cleanly-initialized project.
- Use a TempDir with DONT_DIR pointing to a location without events.jsonl.

## Existing tests affected
- None. No existing tests assert on this warning's presence or absence.
