# Change: add event-log export/import for git-based multi-user sharing

## Why

`dont`'s primary store (`.dont/db.cozo`) is a binary SQLite file. Per
`dont-project-layout`, `.dont/` is meant to stay "self-contained and portable
with the repository," but a binary DB file cannot be diffed or merged by git.
Today, two contributors who each run `dont conclude`/`dont trust`/`dont flag`
in their own clone and then sync via git have no supported way to combine
their claims — the result is either a silent last-write-wins overwrite or
manual reconciliation outside the tool entirely.

Per `dont-data-model`, the store is already an append-only, event-sourced log
of datoms. That log can be serialised to a git-friendly, line-oriented
interchange format without changing the storage engine, giving teams a way
to share epistemic state the same way they already share code.

## What Changes

- New capability `dont-log-sync` introducing a `dont log` subcommand group
  (parallel to how `dont import` is a command family) containing:
  - `dont log export [path]` (default `.dont/events.jsonl`) that writes every
    event currently in the local store to a file, one JSON object per line,
    ordered by transaction number.
  - `dont log import <path>` that reads a JSONL file in that format and
    replays any event whose `id` is not already present in the local store.
    Already-applied events are skipped, so import is idempotent and safe to
    re-run.
- `dont init` scaffolding gains a recommended `.gitattributes` entry marking
  `.dont/events.jsonl` as `merge=union`, and documents that `.dont/db.cozo*`
  should be gitignored as a locally-rebuildable cache rather than committed.
- `dont-project-layout` gains `events.jsonl` as a new layout entry and
  distinguishes git-tracked vs gitignored artifacts.
- **BREAKING**: none. Both verbs are additive; existing single-user, no-git
  workflows are unaffected, and no existing command's behaviour changes.

## Impact

- Affected specs: `dont-log-sync` (new capability), `dont-project-layout`
  (modified — add `events.jsonl` as layout entry, git tracking guidance),
  `dont-init-modes` (modified — git scaffolding on init)
- Affected code: `src/store.rs` (new `export_events`/`import_events`
  methods), `src/main.rs` (new `log export`/`log import` subcommands),
  `src/project.rs` or `src/config.rs` (git scaffolding in `dont init`)

## Out of Scope (deferred to a follow-up change)

- Re-sequencing `tx` numbers across clones for full time-travel correctness.
  `dont-data-model` defines `tx` as a locally monotonically increasing
  counter; this proposal preserves each event's original `tx` on import
  rather than renumbering it, which is sufficient for replay but not for a
  single globally-ordered transaction log. Ordering guarantees for
  time-travel queries across merged histories are left for a later change.
- Real-time or networked sync (e.g. a shared server instance). This proposal
  only covers git-cadence sharing via file export/import.