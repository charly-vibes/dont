## ADDED Requirements

### Requirement: Session-scoped usage statistics command

The system SHALL provide `dont stats` as a read-only command that aggregates event-log data over a
configurable scope and returns a `StatsView` payload. The default scope when no scope flag is given
SHALL be the current calendar day (midnight-to-now UTC). The command SHALL accept `--session <id>`
to scope to a specific session, and `--since <timestamp>` / `--until <timestamp>` for explicit time
windows. `dont stats` MUST NOT open a write transaction or mutate any stored state.

#### Scenario: bare stats returns today's aggregates

- **WHEN** the caller runs `dont stats --json` with no scope flags
- **THEN** the command returns `envelope_kind: "stats"`
- **AND** the payload aggregates over events recorded since midnight UTC of the current calendar day
- **AND** the command performs no writes

#### Scenario: session-scoped stats

- **WHEN** the caller runs `dont stats --session <session-id> --json`
- **THEN** the payload aggregates only over events recorded within that session's boundary events
- **AND** the `scope` field in the payload reflects the resolved session identifier

#### Scenario: time-window scoped stats

- **WHEN** the caller runs `dont stats --since <t1> --until <t2> --json`
- **THEN** the payload aggregates over events whose timestamps fall within `[t1, t2)`
- **AND** `--since` and `--until` accept RFC 3339 timestamps

#### Scenario: inverted time window returns error

- **WHEN** the caller runs `dont stats --since <t2> --until <t1> --json` where `<t2>` is later than `<t1>`
- **THEN** the command returns an error envelope with a message indicating that `--since` must not be after `--until`
- **AND** `envelope.ok` is `false`

#### Scenario: unknown session ID returns error

- **WHEN** the caller runs `dont stats --session <nonexistent-id> --json`
- **AND** no session matching the provided identifier exists in the store
- **THEN** the command returns an error envelope identifying the unknown session ID
- **AND** `envelope.ok` is `false`

#### Scenario: stats on empty scope returns zero counts

- **WHEN** no events exist in the specified scope
- **THEN** the command returns a `StatsView` with all counts set to zero and `idle_skill: true`
- **AND** `envelope.ok` is `true`

### Requirement: StatsView payload fields

The `StatsView` payload SHALL include: `verb_counts` (a map from **write-command** event-kind
string to non-negative integer count; write commands are all subcommands that create or modify
claim-graph state, the canonical write-command names are `conclude`, `define`, `trust`, `flag`, `spawn`, `link`, `lock`, and `ignore`; command aliases are resolved to their canonical name before keying — events from the deprecated `dismiss` alias are counted under `flag`;
read-only commands such as `list`, `show`, `why`, `prime`, `doctor`, `stats`, `export`, `vocab`,
`trace`, and `schema` SHALL NOT appear in `verb_counts`), `dedup_refusal_count` (count of
duplicate-refused error events in the scope), `claim_verification_rate` (ratio of claims whose
status is `verified` as of the `--until` timestamp — or now if no `--until` is set — to the total
number of claims in the store at that same timestamp, regardless of when each claim was created;
expressed as a float in `[0.0, 1.0]`; the field SHALL always be present in the payload — its value is `null` when no claims exist and a float otherwise), `idle_skill` (a boolean that is `true` when the agent performed no write-capable commands during the scope — equivalently, when `verb_counts` has no entries; a `true` value signals inactivity on writes, not absence from the session), and
`caught_contradiction_count` (see the caught-contradiction requirement below). All count fields
SHALL be non-negative integers.

> **Design note:** `claim_verification_rate` is intentionally store-wide at scope end, not bounded
> to the scope window. It measures cumulative epistemic health at a point in time; `verb_counts` and
> `dedup_refusal_count` measure activity within the window. This asymmetry is by design.

#### Scenario: verb_counts includes only write-command event kinds seen in scope

- **WHEN** the caller runs `dont stats --json` over a scope containing conclude, trust, dismiss, and list events
- **THEN** `verb_counts` maps `"conclude"` and `"trust"` to their respective event counts, and `"dismiss"` events are counted under `"flag"`
- **AND** `"list"` and other read-only commands do not appear in `verb_counts`
- **AND** event kinds not present in the scope are omitted from the map rather than appearing as zero

#### Scenario: idle_skill is true when no events exist

- **WHEN** no events are recorded in the scope
- **THEN** `idle_skill` is `true`

#### Scenario: idle_skill is false when any write-command events exist

- **WHEN** at least one write-command event (e.g., conclude, trust, dismiss) is recorded in the scope
- **THEN** `idle_skill` is `false`

#### Scenario: idle_skill is true when only read-only commands were run

- **WHEN** the scope contains only read-only command events (e.g., list, show, prime)
- **THEN** `idle_skill` is `true` because no write-command events were recorded

#### Scenario: claim_verification_rate reflects end-of-scope state

- **WHEN** 10 claims exist in the store and 4 are in `verified` status as of the `--until` timestamp (or now if no `--until` is set)
- **THEN** `claim_verification_rate` is `0.4`
- **AND** claims created before or after the scope window are all counted as long as they exist at the `--until` timestamp

#### Scenario: claim_verification_rate is null when no claims exist

- **WHEN** no claims exist at scope end
- **THEN** `claim_verification_rate` is `null` rather than zero or NaN

### Requirement: Caught-contradiction count

The system SHALL compute `caught_contradiction_count` as the number of `trust` events in the scope
where (a) the event carries `doubt: true` and (b) the targeted claim appears as an evidence
reference for at least one other claim that was created before the doubt event. This metric requires
no new write-path events; it is a retrospective join over the existing event log and evidence
relations. Each doubt event in scope is counted independently regardless of how many prior doubt events targeted the same claim.

#### Scenario: contradiction counted when doubted claim was evidence for another

- **WHEN** claim X is created at T₀ and cited as evidence for claim Y at T₁
- **AND** a `trust --doubt` event targeting claim X is recorded at T₂ > T₁
- **AND** T₂ falls within the stats scope
- **THEN** `caught_contradiction_count` includes this event

#### Scenario: doubt event not counted when targeted claim was never used as evidence

- **WHEN** a `trust --doubt` event targets claim X
- **AND** claim X has never been cited as evidence for any other claim
- **THEN** this event does not increment `caught_contradiction_count`

#### Scenario: multiple doubt events on the same claim count independently

- **WHEN** two distinct `trust --doubt` events both target claim X
- **AND** claim X was used as evidence before both events
- **AND** both events fall within the scope
- **AND** both doubt events are valid recordings that the store permitted
- **THEN** `caught_contradiction_count` is incremented by 2
