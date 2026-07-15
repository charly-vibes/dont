---
tags: [pipeline-run:tdd-ro5-2026-07-09-dont-yqse-exclude-ignored-claims-from-dont-status-and-dont-export-counts, pipeline-step:plan]
---

F22: Exclude ignored claims from dont status and dont export counts

## Problem
Ignored claims remain visible in dont status and dont export counts.
Per path (b) from the issue: exclude ignored claims from counting so
the ledger stays clean without adding a delete verb.

## Root cause
claim_counts_by_status() in src/store.rs returns counts for ALL statuses
including ignored. The call sites (Stats, Export, Check) use these counts
unfiltered.

## Fix
Filter ignored claims out of claim_counts_by_status() so downstream
consumers automatically exclude them. For Prime command, stop displaying
the ignored count.

## Test strategy
- Test that claming an ignored claim does not affect total claim counts
  in dont export --eval --json and dont stats --json output
- Test that dont list still shows ignored claims (list should not change)

## Existing tests affected
- None explicitly assert on ignored counts in status/export
