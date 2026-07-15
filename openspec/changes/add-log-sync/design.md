## Context

`dont` stores every action as an immutable datom `(entity, attr, value, tx,
assert_bit)` in a local Cozo/SQLite file (`.dont/db.cozo`), guarded by a
file lock for single-writer safety (see `dont-data-model`). This works well
on one machine but has no answer for teams: the store was designed assuming
exactly one writer, and nothing in the current CLI surface lets a second
contributor's local history be combined with the first's.

Beads (`gastownhall/beads`), a comparable agent-memory tool, solves this by
using Dolt (a version-controlled SQL engine) as its storage backend, with
`bd dolt push`/`bd dolt pull` syncing against a dedicated git ref. Adopting
Dolt in `dont` would be a large architectural change (new storage engine,
new dependency, schema migration). Because `dont`'s store is already
event-sourced, a lighter-weight approach is available: export the event log
as a git-mergeable text format and replay it into any clone's local store.

## Goals / Non-Goals

- Goals:
  - Let two or more clones of a `.dont`-tracked project combine their event
    history through a normal `git pull` plus one `dont` command.
  - Keep the change additive: no existing verb, schema, or output envelope
    changes behaviour.
  - Make the interchange file boring — plain JSONL, one event per line, no
    new dependency.
- Non-Goals:
  - Real-time / networked multi-writer sync.
  - Global, cross-clone transaction ordering. `tx` remains locally
    monotonic; imported events keep their source `tx` value rather than
    being renumbered into the local sequence.
  - Automatic invocation of export/import from other verbs. This proposal
    only adds the two explicit commands; wiring them into `dont init` /
    a `dont sync` convenience wrapper is a possible follow-up, not required
    here.

## Decisions

- **Decision:** The exported JSONL line shape mirrors the existing internal
  event/datom relation from `dont-data-model` (`id`, `entity_id`,
  `event_kind`, `at`, `author`, plus the underlying datom fields needed to
  reconstruct attribute state) rather than inventing a new schema.
  - Alternatives considered: a bespoke "diff" format. Rejected — reusing the
    existing shape means export is a straight read of already-modelled
    data, and import is a straight replay, with no translation layer to
    keep in sync as the data model evolves.
- **Decision:** `.dont/db.cozo*` is documented as a derived, locally
  rebuildable cache (gitignored); `.dont/events.jsonl` is the artifact
  intended for git. This mirrors beads' treatment of `issues.jsonl` as an
  interchange export sitting alongside (not replacing) its real storage
  engine.
- **Decision:** Import idempotence is keyed on event `id` (ULID), which
  `dont-data-model` already guarantees is unique per event. No new ID
  scheme is needed, unlike beads, which had to introduce hash-based issue
  IDs specifically to avoid collisions across clones.
- **Decision:** A git `merge=union` attribute is the recommended (not
  enforced) merge strategy for `.dont/events.jsonl`, since each line is a
  self-contained, order-independent record and a plain line-union merge
  will not corrupt data — at worst it produces harmless duplicate lines,
  which `dont log import`'s id-based skip already handles.

## Risks / Trade-offs

- **Risk:** Preserving source `tx` values instead of renumbering means two
  clones can have events with the same `tx` number referring to different
  facts. `Time-travel query support` (`dont-data-model`) is defined in
  terms of `tx`, so time-travel queries against an imported history are not
  guaranteed to reflect a single coherent transaction order.
  → Mitigation: documented explicitly as out of scope; `dont log import`
  does not claim to fix time-travel semantics. Teams that need this should
  wait for the ordering follow-up before relying on `dont trace`/time-travel
  queries across merged histories.
- **Risk:** A contributor could commit `.dont/db.cozo` by mistake if
  `.gitignore` scaffolding isn't applied to an already-initialised project.
  → Mitigation: `dont doctor` (existing diagnostic verb) gains a check that
  warns if `.dont/db.cozo` is tracked by git; see tasks.md.
- **Risk:** Two branches that both modify the same claim produce a
  `merge=union` file containing both events with different event IDs.
  Import applies both (different IDs, no dedup), and the last-imported
  event's attribute state effectively wins — which is file-order, i.e.
  branch A's lines then branch B's lines (or vice versa depending on which
  side git union places first). This is a last-writer-wins merge strategy
  and inherits that strategy's well-known limitations.
  → Mitigation: documented as an implicit design choice. The alternative
  (rejecting merges that touch the same entity) is operationally infeasible
  for git-based sharing. Teams that require CRDT-level merge semantics
  should wait for the ordering follow-up.
- **Risk:** Two concurrent `dont log export` invocations both write to
  `events.jsonl`. The last writer silently overwrites the first writer's
  output.
  → Mitigation: export is intended for git-cadence use, not concurrent
  writers. Concurrent export is undefined behaviour and not a supported
  use case; the snapshot isolation guarantee ensures both invocations see
  the same event set if they start within the same transaction window.

## Migration Plan

- New projects: `dont init` scaffolds the `.gitignore`/`.gitattributes`
  entries automatically.
- Existing projects: run `dont log export` once to create
  `.dont/events.jsonl`, add the two scaffolding lines by hand or via
  `dont doctor --fix`, and `git rm --cached .dont/db.cozo*` if it was
  previously committed.
- No rollback concerns: both verbs are additive and read/write only the new
  JSONL file plus already-existing store APIs.

## Open Questions (resolved)

- ~~Should `dont log import` support a `--dry-run` that reports how many new
  events it would apply without writing them?~~ **Resolved: yes.** Dry-run is
  a convergence optimisation for the review-before-apply workflow. It does
  not change idempotence or validation semantics. The spec delta includes
  a non-normative `MAY` requirement; implementation is at implementor's
  discretion for MVP and can be added later.