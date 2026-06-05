---
tags: [pipeline-run:tdd-ro5-2026-06-05-cli-consistency-exit-codes-json-flag, pipeline-step:plan]
---

cli-consistency-exit-codes-json-flag: Normalize exit codes and --json flag coverage

## What was built
Two consistency fixes across the dont CLI:
1. Exit codes normalized to 0/1/2 per spec (0=success, 1=domain error, 2=usage error)
2. --json flag added to all data-outputting commands that were missing it

## Test strategy
Integration tests at tests/exit_codes.rs and tests/json_flag.rs:
- Each command tested for correct exit code on success, domain error, and usage error
- --json flag tested to produce valid JSON envelope on stdout for all data commands
- Existing tests unchanged; new tests added in parallel

## Test cases written
- exit code 0 on success for: conclude, define, ground, trust, flag, dismiss, undoubt, forget, lock, reopen, ignore, list, show, why, trace, version
- exit code 1 on domain error (not-found, already-flagged, etc.)
- exit code 2 on usage error (missing required arg)
- --json flag produces JSON envelope with ok/data/error fields for: list, show, why, trace, version
- --json output is valid JSON parseable by jq

## Affected commands
conclude, define, ground, trust, flag, dismiss, undoubt, forget, lock, reopen, ignore, list, show, why, trace, version, completions
