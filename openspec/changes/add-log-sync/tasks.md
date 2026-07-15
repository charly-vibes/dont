## 1. Store layer

- [ ] 1.1 Add `Store::export_events` returning all events ordered by `tx`, reusing the existing event/datom read path. Evaluate whether streaming (iterator) is needed for stores >10K events before finalising the return type.
- [ ] 1.2 Add `Store::import_events` that, per event, checks whether `id` already exists and, if not, appends the corresponding datoms within a single transaction per event, preserving the source `tx` value. The implementation SHALL pre-scan the file for well-formedness before applying any events (all-or-nothing rejection on malformed lines).
- [ ] 1.3 Define the `EventLine` JSON shape (serde) matching the field-level schema table in `dont-log-sync/spec.md`. The serde definition MUST produce JSON output that round-trips through the spec's EventLine schema table.

## 2. CLI surface

- [ ] 2.1 Add `dont log export [path]` (default `.dont/events.jsonl`), writing one `EventLine` JSON object per line. Export SHALL snapshot the store at start so concurrent writes are not reflected.
- [ ] 2.2 Add `dont log import <path>`, printing a summary envelope with counts of applied vs. skipped events. Import SHALL validate well-formedness of the entire file before applying any events.
- [ ] 2.2a Add `--dry-run` flag to `dont log import`: validate the file, print the summary, but do not write any events.
- [ ] 2.3 Add both verbs to `dont completions` and `dont help` per `dont-cli-surface` conventions (universal flags, `--json` support).
- [ ] 2.4 (spec gap) Add a `dont doctor` check that warns (not fails) if `.dont/db.cozo` is tracked by git.

## 3. Project scaffolding

- [ ] 3.1 Update `dont init` to write `.dont/db.cozo*` into `.gitignore` and `.dont/events.jsonl merge=union` into `.gitattributes` when a git repo is detected. Scaffolding SHALL be additive: append if absent, never overwrite existing entries.
- [ ] 3.2 Add `dont doctor --fix` support for git scaffolding (implied by the `init === doctor --fix` invariant): if `.dont/db.cozo*` is missing from `.gitignore` or the `merge=union` attribute is missing from `.gitattributes`, `dont doctor --fix` SHALL add them.

## 4. Tests

- [ ] 4.1 Export produces one line per stored event, in `tx` order.
- [ ] 4.2 Import of a freshly exported file into an empty store reproduces the same `list-claims`/`list-terms` output as the source store.
- [ ] 4.3 Re-running `dont log import` on the same file twice is a no-op the second time (idempotence).
- [ ] 4.4 Importing a file containing a mix of already-applied and new events applies only the new ones.
- [ ] 4.5 Concurrent-access test analogous to existing `tests/concurrent_access.rs`, covering `dont log import` racing with a normal write verb.
- [ ] 4.6 Edge-case tests: empty store export (0 events → 0 lines), empty file import (0 lines → 0 applied), malformed-line import rejection (bad JSON on line 3 → total rejection with error on line 3).
- [ ] 4.7 EventLine JSON shape round-trip test: construct an event in code, serialise to JSONL, deserialise, and confirm all fields match.
- [ ] 4.8 (if --dry-run implemented) Dry-run import produces same summary as real import but does not mutate the store.

## 5. Docs

- [ ] 5.1 Document the export/import workflow and the "commit `events.jsonl`, gitignore `db.cozo`" pattern in `.dont/AGENTS.md` and `docs/`.
- [ ] 5.2 Note the `tx`-preservation / time-travel caveat from `design.md` in user-facing docs.
- [ ] 5.3 Add the `dont doctor` check for git-tracked `db.cozo` to any existing `doctor` doc listing.
