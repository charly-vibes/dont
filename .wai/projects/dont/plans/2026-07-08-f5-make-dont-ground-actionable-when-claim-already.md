---
tags: [pipeline-run:tdd-ro5-2026-07-08-dont-cfki-dont-ground-should-upsert-or-suggest-dont-flag-when-claim-already-exists, pipeline-step:plan]
---

F5: Make dont ground actionable when claim already exists

## Problem
Running dont ground a second time with the same statement errors with:
  'claim with equivalent text already exists as {existing_id}'
The only remediation suggested is dont show {existing_id} — but the user's
intent is to add evidence, not inspect. This forces a context switch:
figure out that flag exists, look up the claim ID, re-run.

## Root cause
handle_store_error_code (src/main.rs:1262-1280) handles DuplicateClaim by
emitting a refusal with a single remediation pointing to dont show.

## Fix
Add a second remediation entry to the DuplicateClaim refusal:
1. 'dont show {existing_id}' — keep existing (inspect)
2. 'dont flag {existing_id} --evidence <locator>' — NEW (actionable next step)

This is Option B (lower risk) from the issue: actionable error with
the correct dont flag invocation, rather than silent upsert.

## Test strategy
- New test file or add to existing: test that dont ground on a duplicate
  statement emits a structured error with:
  - ok: false
  - data.code: duplicate-refused
  - data.remediation contains an entry with dont flag and the claim ID
- Key: use dont ground with an http:// evidence URI (not --file) to keep
  the test simple (no git repo needed)

## Existing tests affected
- No existing tests test the duplicate ground path
- All existing ground tests should be unaffected

## Interface change
- No CLI flag changes, no new commands
- Only the error message and remediation suggestions change
