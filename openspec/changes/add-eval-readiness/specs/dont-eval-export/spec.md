## ADDED Requirements

### Requirement: dont export --eval command

The system SHALL provide `dont export --eval` as a read-only command that produces a structured JSON
document describing session or time-window activity in a format suitable for eval harnesses. The
command SHALL accept the same `--session <id>`, `--since <timestamp>`, and `--until <timestamp>`
scope flags as `dont stats`. The default scope when no scope flag is given SHALL be the current
calendar day (midnight-to-now UTC). `dont export --eval` MUST NOT open a write transaction or
mutate any stored state.

#### Scenario: eval export returns EvalExport payload

- **WHEN** the caller runs `dont export --eval --json`
- **THEN** the command returns `envelope_kind: "eval_export"`
- **AND** the payload contains a complete `EvalExport` document as defined by this spec
- **AND** the command performs no writes

#### Scenario: eval export scoped to session

- **WHEN** the caller runs `dont export --eval --session <session-id> --json`
- **THEN** the payload's `scope.session_id` reflects the provided session identifier
- **AND** all aggregated data is restricted to that session's events

#### Scenario: unknown session ID returns error

- **WHEN** the caller runs `dont export --eval --session <nonexistent-id> --json`
- **AND** no session matching the provided identifier exists in the store
- **THEN** the command returns an error envelope identifying the unknown session ID
- **AND** `envelope.ok` is `false`
- **AND** the process exits with a non-zero exit code consistent with the `dont-cli-surface` error exit convention

#### Scenario: eval export scoped to time window

- **WHEN** the caller runs `dont export --eval --since <t1> --until <t2> --json`
- **THEN** the payload's `scope.since` and `scope.until` reflect the provided timestamps
- **AND** all aggregated data is restricted to events within `[t1, t2)`

#### Scenario: eval export on empty scope returns zeroed payload

- **WHEN** no events exist in the specified scope
- **THEN** the command returns a valid `EvalExport` with all count maps empty and array fields empty
- **AND** `envelope.ok` is `true`

### Requirement: EvalExport payload structure

The `EvalExport` payload SHALL be a flat JSON document (not NDJSON) containing the following fields:

- `exported_at` — RFC 3339 timestamp of export generation
- `scope` — object with fields `since` (RFC 3339), `until` (RFC 3339), and optionally `session_id`
  (string) when a session scope is used
- `claims_by_status` — map of status string to non-negative integer count of claims at scope end
- `events_by_kind` — map of event-kind string to non-negative integer count of events in scope
- `trust_events` — array of objects, one per trust event in scope, each containing `event_id`,
  `target_claim_id`, `doubt` (boolean), `reason_excerpt` (first 120 Unicode code points of the
  reason string; included because reason text may contain task-specific content — callers sharing
  eval exports externally should be aware that excerpts may leak task context), and `timestamp`
  (RFC 3339)
- `dedup_refusals` — array of objects, one per duplicate-refused error event in scope, each
  containing `attempted_text_hash` (the normalized hash of the rejected text) and `timestamp` (RFC
  3339)
- `wall_clock_duration_seconds` — a non-negative integer, present only when a session scope is used
  and both a session-start and session-end event exist; absent otherwise

All count fields SHALL be non-negative integers. Array fields SHALL be empty arrays when no matching
events exist in scope, not null or absent.

#### Scenario: payload contains all required top-level fields

- **WHEN** the caller runs `dont export --eval --json` over a non-empty scope
- **THEN** the payload contains `exported_at`, `scope`, `claims_by_status`, `events_by_kind`,
  `trust_events`, and `dedup_refusals`

#### Scenario: trust_events array includes one entry per trust event

- **WHEN** three trust events exist in the scope
- **THEN** `trust_events` contains exactly three objects
- **AND** each object includes `event_id`, `target_claim_id`, `doubt`, `reason_excerpt`, and `timestamp`

#### Scenario: reason_excerpt is truncated at 120 characters

- **WHEN** a trust event's reason string is longer than 120 characters
- **THEN** `reason_excerpt` contains exactly the first 120 Unicode code points of the reason string

#### Scenario: dedup_refusals array is empty when no duplicates were attempted

- **WHEN** no duplicate-refused error events exist in the scope
- **THEN** `dedup_refusals` is an empty array, not null or absent

#### Scenario: wall_clock_duration_seconds is absent for time-window scope

- **WHEN** the caller uses `--since` / `--until` rather than `--session`
- **THEN** `wall_clock_duration_seconds` is absent from the payload

#### Scenario: wall_clock_duration_seconds is absent when session lacks end event

- **WHEN** a session scope is used but no session-end event has been recorded
- **THEN** `wall_clock_duration_seconds` is absent from the payload

### Requirement: Eval export is the only --eval subformat for now

The `dont export` command family SHALL accept `--eval` as the only defined format flag in this
change. Bare `dont export` without a format flag SHALL return a usage error listing the available
format flags. The design deliberately leaves room for future format flags (e.g., `--graphml`,
`--csv`) without defining them in this change.

#### Scenario: bare export without format flag returns usage error

- **WHEN** the caller runs `dont export --json` without `--eval` or any other format flag
- **THEN** the command returns an error envelope listing the available format flags
- **AND** `envelope.ok` is `false`

#### Scenario: unknown format flag returns usage error

- **WHEN** the caller runs `dont export --unknown-format --json`
- **THEN** the command returns an error envelope identifying the unknown flag
