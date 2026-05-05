# Test Strategy: Tracer Bullet

## Purpose

This document defines the acceptance criteria and coverage requirements for the
Phase 10 integration tests. Unit tests in phases 2–9 validate individual
modules; integration tests validate the assembled binary against the spec
contract end-to-end.

## Test Harness Approach

Each integration test:

1. Runs `dont` as a subprocess (not linked directly).
2. Captures stdout (JSON envelope), stderr (ignored), and exit code.
3. Parses stdout as `Envelope<T>` and asserts schema conformance before any
   field-level assertions.
4. Uses `DONT_DIR` to isolate each test to a temporary directory; tests MUST
   NOT share a `.dont/` directory.

Schema conformance means:
- Top-level fields `ok`, `envelope_version`, `cli_version`, `envelope_kind`,
  `data`, `warnings`, and `meta` are all present.
- `envelope_version` is exactly `"0.2"`.
- `warnings` is always an array.
- `meta.duration_ms` is a non-negative integer, `meta.tx` is an integer for
  mutating commands and `null` for read-only commands, and `meta.request_id` is
  `null` for the tracer.
- When `ok: true`, `hints` is present as an array and the `data` shape matches
  the declared `envelope_kind`.
- When `ok: false`, `hints` is absent and `data.remediation` is a non-empty
  array where every entry has `command` and `description` strings.

## Required Coverage

### Envelope contract (every command)

| Scenario | Expected |
|---|---|
| Every successful command | `ok: true`, envelope_version `"0.2"`, non-null `envelope_kind` |
| Every refusal | `ok: false`, exit 1, non-empty `remediation[]`, `rule_name: null` for verb-level refusals |
| Every substrate error | `ok: false`, exit 3 |

### Exit code conformance

Every scenario in tasks 5.2, 6.2, 7.2, 7.4, 8.2, 9.3 must assert the exit
code explicitly (not just parse the envelope), to catch mismatches between
envelope `ok` and actual exit behaviour.

### Persistence across invocations

| Scenario | Expected |
|---|---|
| `conclude` then separate `show` invocation | `show` returns the claim created by the prior `conclude` invocation |
| `conclude` → `trust` → separate `show` | `show` reflects `doubted` status from the prior `trust` invocation |

These confirm CozoDB persistence survives process exit, not just in-process
state.

### Project discovery

| Scenario | Expected |
|---|---|
| Run command from subdirectory of the project root | `.dont/` found via parent walk; command succeeds |
| Run command outside any project (no `.dont/` ancestor) | Refusal with `config-missing`, exit 3, remediation suggests `dont init` |
| `DONT_DIR` set to a valid project root | Command uses that directory regardless of CWD |
| `DONT_DIR` set to a non-existent path | Refusal with `config-missing`, exit 3 |

### Hedge MVP

| Scenario | Expected |
|---|---|
| `trust <id> --reason "maybe"` | Refusal `reason-not-hedge`, exit 1, non-empty `remediation[]` |
| `trust <id> --reason "I think this is wrong"` | Refusal `reason-not-hedge`, exit 1 |
| `trust <id> --reason "Evidence contradicts §3.2"` | Succeeds, exit 0 |

The hedge check MUST use configured case-insensitive substring matching, not
regular expressions, to match `dont-project-config`.

### Status lattice

All four valid transitions must be covered in integration tests (not just unit
tests), proving they work through the full binary including storage round-trip:

| Transition | Verb | Exit |
|---|---|---|
| `unverified → doubted` | `trust` | 0 |
| `unverified → verified` | `dismiss` | 0 |
| `verified → doubted` | `trust` | 0 |
| `doubted → verified` | `dismiss` | 0 |

Invalid transitions must also be confirmed at the integration level:

| Scenario | Expected |
|---|---|
| `trust` on already-doubted claim | `invalid-transition`, exit 1 |
| `dismiss` on already-verified claim with additional evidence | Appends evidence/history without changing identity or status; exits 0 |

### Performance

`dont list` must complete in under 50ms on a project seeded with 100 claims.
Measured as wall-clock time of the subprocess call, not just CozoDB query time.
The 100-claim seed MUST be created via the actual `conclude` command (not
direct DB writes) to include realistic event log overhead.

## What Integration Tests Do Not Cover

- Rule engine behaviour (no rules in tracer)
- Spawn protocol and harness detection
- Colour rendering and shell completions
- Import adapters
- `--json` flag parsing beyond the always-on JSON output

These are explicitly deferred per `proposal.md`.
