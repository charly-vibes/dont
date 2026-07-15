## ADDED Requirements

### Requirement: Event log export
The system SHALL provide `dont log export [path]`, defaulting `path` to
`.dont/events.jsonl`, which writes every event currently in the local store
to that file as one JSON object per line ("JSONL"), ordered by transaction
number (`tx`). Each line SHALL contain the event's `id`, `entity_id`,
`event_kind`, `at`, `author`, `tx`, and any attribute assertions/retractions
associated with that transaction, matching the `event`/`attribute` relations
defined in `dont-data-model`. Export SHALL be a read-only operation: it MUST
NOT create new events, mutate `tx`, or lock the store for writing beyond a
snapshot read. Export SHALL take a consistent snapshot of the store at the
start of the operation so that concurrent writes during export are not
reflected in the output.

#### Scenario: export writes one line per event
- **WHEN** the local store contains 12 events and the caller runs `dont log export`
- **THEN** `.dont/events.jsonl` is created (or overwritten) with exactly 12 lines, each a single JSON object

#### Scenario: export uses the default path when none is given
- **WHEN** the caller runs `dont log export` with no `path` argument
- **THEN** the output is written to `.dont/events.jsonl`

#### Scenario: export accepts an explicit path
- **WHEN** the caller runs `dont log export /tmp/backup.jsonl`
- **THEN** the output is written to `/tmp/backup.jsonl` instead of the default location

#### Scenario: export preserves transaction order
- **WHEN** the store contains events from transactions 1 through 12
- **THEN** the exported lines appear in ascending `tx` order

#### Scenario: export does not mutate the store
- **WHEN** `dont log export` runs against a store with an existing history
- **THEN** the store's event count and latest `tx` are unchanged after export completes

#### Scenario: export of an empty store produces zero lines
- **WHEN** `dont log export` runs against a store with no events
- **THEN** `.dont/events.jsonl` is created (or overwritten) as an empty file

#### Scenario: export snapshots the store at start
- **WHEN** a concurrent `dont conclude` commits an event during `dont log export`
- **THEN** the exported file does not contain the concurrently written event

### Requirement: Event line JSON schema for interop
The EventLine JSON object SHALL use the following schema, defined here as the canonical reference since the data model spec defines the domain concepts but not the interchange serialisation. Every field SHALL be present on every line (no sparse/optional top-level fields except assertions which is an array and may be empty).

| Field | Type | Description |
|-------|------|-------------|
| `id` | string (ULID) | The event's unique identifier, prefixed `event:` |
| `entity_id` | string (ULID, prefixed) | The entity this event belongs to (e.g. `claim:01HX...`) |
| `event_kind` | string | One of the 12 canonical event kinds per `dont-data-model` |
| `at` | string (RFC 3339) | Timestamp of the event |
| `author` | string | Author identity in `<actor-kind>:<id>` format |
| `tx` | integer | Monotonically increasing transaction number from the source store |
| `assertions` | array of objects | Attribute assertions in this transaction; each object has `attr` (string), `value` (any valid JSON), and `assert` (boolean, true=assertion, false=retraction) |

The `assertions` array MUST contain at least one entry for every event type except those that carry no attribute changes (e.g., some meta-events by convention). Implementations MUST NOT reject an event with an empty `assertions` array if the source store produced it, but SHOULD NOT generate such events.

#### Scenario: EventLine round-trips through serde
- **WHEN** a valid EventLine JSON object is serialised from a store event and then deserialised
- **THEN** all fields match the original event: `id`, `entity_id`, `event_kind`, `at`, `author`, `tx`, and every entry in `assertions`

#### Scenario: assertions array carries attr, value, assert
- **WHEN** an event includes attribute assertions
- **THEN** each assertion object in the `assertions` array has string `attr`, a JSON value `value`, and boolean `assert`

#### Scenario: EventLine with empty assertions is parseable
- **WHEN** an EventLine has `"assertions": []`
- **THEN** the line is valid and deserialises without error

### Requirement: Event log import
The system SHALL provide `dont log import <path>`, which reads a file in
the format produced by `dont log export` and, for each line, applies the
event to the local store only if no event with that `id` already exists
locally. Each newly-applied event's original `tx` value from the source
file SHALL be preserved rather than renumbered into the local store's
transaction sequence. Import SHALL validate the entire file for
well-formedness before applying any events: a single malformed line SHALL
cause the entire import to be rejected with no partial writes. The command
SHALL print a summary containing the count of events applied and the count
skipped as already-present.

#### Scenario: import applies new events
- **WHEN** the caller runs `dont log import teammate.jsonl` and none of its event `id`s exist locally
- **THEN** every event in `teammate.jsonl` is applied to the local store and the summary reports that count as applied

#### Scenario: import is idempotent
- **WHEN** the caller runs `dont log import teammate.jsonl` a second time with no local changes in between
- **THEN** no new events are applied and the summary reports all events as skipped

#### Scenario: import applies only new events from a mixed file
- **WHEN** `teammate.jsonl` contains some event `id`s already present locally and some that are not
- **THEN** only the not-yet-present events are applied, and the summary reports both an applied count and a skipped count

#### Scenario: import preserves source transaction numbers
- **WHEN** an imported event's source `tx` is `7`
- **THEN** the event is stored locally with `tx = 7`, not renumbered to the local store's next transaction number

#### Scenario: malformed line is rejected without partial writes
- **WHEN** a line in the import file is not valid JSON or is missing a required field
- **THEN** the command exits non-zero with a non-empty `remediation[]` naming the offending line number, and no events from that file are applied

#### Scenario: import of an empty file reports zero applied and zero skipped
- **WHEN** the caller runs `dont log import empty.jsonl` and the file contains no lines
- **THEN** the summary reports 0 applied and 0 skipped

### Requirement: Event log import supports --dry-run
The system MAY accept `--dry-run` on `dont log import`. When set, the import SHALL read and validate the file for well-formedness, compute which events would be applied and which would be skipped, print the same summary as a real import, but SHALL NOT write any events to the store. Dry-run is a convergence optimisation for the review-before-apply workflow and MUST NOT change the idempotence or validation semantics.

#### Scenario: dry-run prints summary without writing
- **WHEN** the caller runs `dont log import --dry-run teammate.jsonl`
- **THEN** the summary is printed with counts of new and skipped events
- **AND** no events are written to the store

#### Scenario: dry-run validates well-formedness
- **WHEN** `dont log import --dry-run malformed.jsonl` runs against a file with a broken line
- **THEN** the command exits non-zero with remediation naming the offending line
- **AND** no events are written to the store

### Requirement: Doctor warns about git-tracked store file
The system SHALL provide a `dont doctor` check that warns (not fails) if
`.dont/db.cozo` or any file matching `.dont/db.cozo*` is tracked by git.
The warning SHALL include the remediation "add `.dont/db.cozo*` to
`.gitignore` and run `git rm --cached .dont/db.cozo*`".

#### Scenario: doctor warns on tracked db.cozo
- **WHEN** `dont doctor` runs in a git repository where `.dont/db.cozo` is tracked
- **THEN** the output includes a warning about the tracked store file

#### Scenario: doctor passes on gitignored db.cozo
- **WHEN** `dont doctor` runs in a git repository where `.dont/db.cozo*` is already gitignored
- **THEN** no warning is emitted about the store file

#### Scenario: doctor skip check outside git repo
- **WHEN** `dont doctor` runs in a directory that is not a git repository
- **THEN** the git-tracked store check is skipped
