# Design: eval readiness features

## Scope and motivation

All four features share one motivation: making `dont`'s internal state legible to external evaluation
infrastructure. They are additive read-only commands plus one pass-through flag; none touches the
write path or the claim-graph semantics.

## dont stats — analytics command

**What it queries.** `dont stats` aggregates over the event log for a specified scope (current
session, a date range, or the full project history). It emits a `StatsView` JSON payload.

**Key fields and their definitions:**

| Field | Definition | Eval reference |
|-------|-----------|----------------|
| `verb_counts` | Map of event kind → count | "verb mix" |
| `dedup_refusal_count` | Count of `DUPLICATE_REFUSED` error events | "dedup-hit rate" |
| `idle_skill` | `true` if no `dont` calls were recorded in the scoped period | "idle-skill rate" |
| `claim_verification_rate` | `verified_count / total_claims` at scope end | "claim-verification rate" |
| `caught_contradiction_count` | Count of `trust --doubt` events whose target claim appears as evidence in a later claim | "caught-contradiction rate" |

**Session scoping.** The scope is defined by a `--session <id>` flag referencing a session start
event, or `--since <timestamp>` / `--until <timestamp>` for time-based windows. Bare `dont stats`
with no scope flag returns statistics over the current day (wall-clock midnight-to-now UTC).

**Caught-contradiction computation.** A contradiction is "caught" when:
1. A `trust` event with `doubt: true` targets claim X at time T₁.
2. Before T₁, claim X appears as an `evidence` reference for some other claim Y (i.e., the agent
   had been building on X).

This is computable entirely from the event log using a retrospective join; it requires no new
write-path events.

**Why not a derived query on `dont list`?** Stats requires aggregations (counts, ratios) that are
not naturally expressed as filtered entity lists. A dedicated command avoids overloading `list` with
grouping semantics.

## --no-persist — ephemeral mode flag

**Mechanism.** `--no-persist` is added as a universal flag (alongside `--json`, `--plain`, etc. per
`dont-cli-surface`). When set, the command:
1. Opens the store for reading (to validate entity references, dedup checks, etc.) but holds no write
   transaction.
2. Validates the command as if it would succeed, including hedge-rejection, dedup checks, and rule
   evaluation.
3. Returns a success envelope (or the appropriate validation error envelope) as if the write had
   occurred.
4. Writes nothing to the event log.

**Why not a separate binary?** A flag keeps the C2 condition behaviourally identical to C1 (same
binary, same prompt, same tool schema), isolating only the persistence dimension. A separate binary
introduces surface differences that could confound evaluation.

**Relationship to `--dry-run`.** `dont rules test` already uses the term "dry-run" for rule
evaluation without state mutation. `--no-persist` is scoped to write commands only; read commands
are unaffected. We avoid the term "dry-run" at the global flag level to prevent confusion with the
rules-test dry-run.

**Store consistency.** Because `--no-persist` does not open a write transaction, it does not hold a
write lock. This means a concurrent writer could commit between the read-validation and the simulated
write. The ephemeral mode is explicitly not serialisable with concurrent writes; this is acceptable
for eval harness use (single-agent, single-session).

## dont export --eval — eval-export format

**Relationship to existing commands.** `dont export` is a new top-level command family (not a flag
on `dont stats`). The `--eval` subformat is the first defined format; additional formats (e.g.,
`--graphml` for graph export) are deferred.

**Payload shape.** The `EvalExport` payload is a flat JSON document (not NDJSON) containing:
- `session_id` (if scoped to a session)
- `exported_at` (RFC 3339 timestamp)
- `scope` (the time window or session reference used)
- `claims_by_status` (map of status → count)
- `events_by_kind` (map of event kind → count)
- `trust_events` (array of `{event_id, target_claim_id, doubt, reason_excerpt, timestamp}`)
- `dedup_refusals` (array of `{attempted_text_hash, timestamp}`)
- `wall_clock_duration_seconds` (if session scope is used and the session has both start and end events)

**Why flat?** Eval harnesses (Python scripts, R notebooks) need simple columnar data, not graph
traversal. The flat shape trades expressiveness for parse simplicity.

**Scope flag reuse.** `dont export --eval` accepts the same `--session` / `--since` / `--until`
scope flags as `dont stats` for consistency.

## Capability placement

These three new capabilities do not modify any existing spec. They are additive:
- `dont-analytics` is a new read-only query command, parallel to `dont prime` and `dont doctor`.
- `dont-ephemeral-mode` adds a universal flag to the `dont-cli-surface` contract but is specified
  separately to avoid bloating that spec.
- `dont-eval-export` is a new top-level command in the same family as `dont import` (specified in
  `dont-import-surface`); like import, it is a top-level I/O command, not a filter on an existing
  subcommand.

All three can be implemented independently and in any order.

## Failure modes and mitigations

| Failure mode | Risk level | Mitigation |
|---|---|---|
| `dont stats` slow on large stores | MEDIUM | `caught_contradiction_count` requires a retrospective join over evidence relations; implementations should materialise this as an indexed query rather than a full table scan. Add a time-budget guard: if the join exceeds 5 s, return a partial result with `caught_contradiction_count: null` and a `warning` field. |
| `dont export --eval` slow on large stores | MEDIUM | `trust_events` and `dedup_refusals` arrays are unbounded; implementations should paginate or cap at 10 000 entries per field and include a `truncated: true` marker when the cap is hit. |
| `reason_excerpt` leaks task context | LOW | Callers sharing eval exports with third parties should redact `trust_events[*].reason_excerpt` if task content is sensitive. The 120-character truncation limits exposure but does not eliminate it. |
| `DONT_NO_PERSIST` misconfigured | LOW | If `DONT_NO_PERSIST` is set to any value other than `"1"`, the flag is silently treated as unset. Implementations SHOULD log a warning when an unrecognised value is detected to aid debugging. |
