# Design: WASM claims-query surface

## Context

The `dont` CLI is a Rust single binary backed by an embedded CozoDB instance
using the SQLite storage engine (`src/store.rs:309`,
`DbInstance::new("sqlite", ...)`). The store is coupled to the filesystem in
four places: the SQLite file, an `fs2` exclusive file lock (`with_file_lock`),
a `tx.seq` sidecar (`seq_path`), and `std::panic::catch_unwind` corrupt-file
detection around `DbInstance::new`.

We want the same inference engine available in the browser, inside the mdBook
docs site, with identical query semantics. The browser target cannot use
SQLite, file locks, or the seq sidecar.

### Validated facts (grounding)

1. **Cozo mem engine compiles to WASM.** `cargo check --target
   wasm32-unknown-unknown` with `cozo = { default-features=false,
   features=["wasm"] }` and `getrandom = { version="0.2", features=["js"] }`
   builds cleanly in 17s. `cozo/src/lib.rs:138` shows the `"mem"` arm is
   unconditional (no feature gate); only `sqlite`/`rocksdb`/`sled`/`tikv` are
   gated. The `wasm` feature enables `uuid/js` + `js-sys`.
2. **`getrandom` is the only WASM blocker** in core cozo. It enters via `rand`
   (used by cozo core); the standard `js` feature resolves it. No storage
   backend is needed because `mem` is always compiled.
3. **Shipped rules are engine-agnostic.** They use only `list_claims`,
   `list_terms`, `term_by_id`, `resolve_curie_reference` (`src/rules/*.rs`).
   File-based `.dl` rules use `run_rule_query` (`src/rules/mod.rs:210`), which
   calls `db.run_script_str` — also engine-agnostic.
4. **`ClaimRecord` / `TermRecord` / `EventRecord` / `CurieResolution` /
   `ImportedTermRecord` do NOT derive `Serialize`** (`src/store.rs:99,107` and
   following; only `AtomRecord`/`HypothesisRecord`/`HypothesisAssessment` do at
   `:115,123,129`). The CLI hand-builds JSON via `build_claim_view`
   (`src/main.rs:2106`), which also takes a live `&Store`.
5. **Cozo `backup_db` / `restore_backup` are unusable in WASM.** They require
   `storage-sqlite` and write to an `impl AsRef<Path>` file
   (`cozo/src/runtime/db.rs:609,627`). Rejected as the snapshot format.

## Goals / Non-Goals

- Goals:
  - Read-only browser query surface with CLI-identical rule and Datalog
    semantics.
  - A portable snapshot format produced by the CLI and consumed by WASM.
  - Minimal, non-invasive library change (no `Store` trait abstraction).
- Non-Goals:
  - Browser-side writes.
  - Full CLI `ClaimView` JSON parity (derived assessments, projected evidence).
  - Live project sync.
  - A trait abstraction over `Store` (rejected; see D1).

## Decisions

### D1: Reuse `Store` via a mem-engine constructor, not a trait

Add `Store::open_mem()` that constructs `DbInstance::new("mem", "", "")` and
runs `ensure_schema()`, skipping `with_file_lock`, `tx.seq`, permission
tightening, and `catch_unwind`. The existing query methods are unchanged
because they only call `db.run_script_str`.

- Rationale: a `QueryStore` trait would ripple through `main.rs` (7000+ lines)
  and the rule signatures (`fn check(store: &Store)`). A second constructor
  keeps the diff small and the WASM path reuses the exact same code the CLI
  exercises.
- Trade-off: `Store` retains `path`/`lock_path`/`seq_path` fields in the WASM
  build. Make them `Option<PathBuf>` (cheap) or leave as unused `PathBuf`
  (dead code, acceptable). Prefer `Option` so `open_mem` constructs without
  inventing paths.
- Alternative considered: a `QueryStore` trait implemented by `Store` and a
  `MemStore`. Rejected — duplicates the query method surface and forces rule
  generics for no behavioral gain, since the mem engine satisfies the same
  `run_script_str` contract.

### D2: Snapshot format = datom-level JSON, rehydrated via `import_datoms`

The export command queries `*datoms[entity, attribute, value, tx, assert_bit]`
and serializes the rows to a JSON array. The WASM module creates a fresh
`Store::open_mem()` and bulk-inserts the datoms via a new public
`Store::import_datoms(&[Datom])` entry point that wraps the existing internal
`put_datoms` path (plus the schema-version check and reset-on-reload).

- Rationale: a literal datom dump preserves the entire DB state (claims, terms,
  events, atoms, hypotheses, tx ordering, **and retractions**) without
  reconstructing it through high-level `append_*` methods. It needs no new
  `Serialize` derives on record types (resolves the CORR-001 gap by
  sidestepping it). `put_datoms` (via `import_datoms`) already exists and is
  used by every write path.
- Trade-off: the snapshot is larger than a trimmed record JSON and is coupled
  to the `datoms` schema shape. Acceptable for a docs-page payload (single
  project, committed at build time, gzip-served).
- Alternatives considered:
  - Cozo `backup_db`/`restore_backup`: rejected (D5; requires sqlite + fs).
  - Add `Serialize` to record types + re-insert via `append_claim`/`append_term`:
    rejected for the snapshot path (re-implements the datom model, loses event
    tx fidelity). May still be added later for a trimmed record-view API.
- Schema-stability note: the snapshot is versioned by `schema_version`
  metadata; the WASM module MUST refuse a snapshot whose `schema_version`
  differs from its compiled expectation.

### D3: FFI surface via `wasm-bindgen`, JSON-string returns

The FFI lives in a new `src/wasm.rs` module **inside the `dont` crate**,
feature-gated behind a `wasm` feature so it compiles only for the
`wasm32-unknown-unknown` target and is absent from the native binary. Keeping
it in-crate lets the FFI call the private `put_datoms` path indirectly, via a
new public `Store::import_datoms(&[Datom])` entry point that wraps
`put_datoms` + the schema-version check + reset-on-reload. (We add a public
entry point rather than exposing `put_datoms`, so the public surface is
purpose-built for rehydration.)

Expose a small `cdylib` with `wasm-bindgen`:
- `load_snapshot(json: &str) -> String` — parse datom rows, reset the
  module-global mem store if populated, `import_datoms` into a
  `Store::open_mem()`, return an envelope JSON (`ok`/error).
- `list_claims() -> String` / `list_terms() -> String` — return the existing
  record data as JSON. (Requires `Serialize` on `ClaimRecord`/`TermRecord` for
  this FFI layer only; add the derives here — they are cheap and orthogonal to
  D2's snapshot path.)
- `run_shipped_rule(name: &str) -> String` — return `Vec<RuleMatch>` as JSON.
- `run_datalog(script: &str) -> String` — return `run_script_str` result rows
  as JSON, read-only (reject mutability).

- Rationale: an in-crate feature-gated module avoids a separate wrapper crate
  and the private-fn visibility problem, and reuses the exact code paths the
  CLI exercises. JSON strings avoid `wasm-bindgen` value-marshalling
  complexity for nested serde types and match the CLI's `--json` envelope
  convention.
- Trade-off: a parse step on each call; negligible for a docs page.

### D4: Persistence across page reloads via IndexedDB snapshot cache

The mem engine is ephemeral — every page navigation would reload an empty DB.
The JS loader caches the snapshot blob in IndexedDB keyed by a content hash
emitted alongside the snapshot asset. On load, it rehydrates the WASM mem DB
from the cache; it re-fetches only when the hash changes.

- Rationale: avoids re-parsing the snapshot on every navigation while keeping
  the WASM engine itself stateless and simple.
- Trade-off: IndexedDB is a JS-side concern; the Rust WASM module stays
  storage-free.

### D5: No file locking, no `catch_unwind`, no `tx.seq` in the WASM path

- `with_file_lock` becomes a no-op passthrough in `open_mem` (single-threaded
  isolate; no cross-process contention).
- `catch_unwind` corrupt-file guard is dropped — the mem engine has no file to
  corrupt. (Also avoids the `panic=abort` vs `catch_unwind` conflict on
  `cdylib`/`wasm32`.)
- `next_tx` reads from an in-memory counter instead of `seq_path` when the
  store was opened via `open_mem`.

### D6: WASM build artifact, target-gated dependencies

Add a `wasm` build profile / example in `Cargo.toml`. Target-gate the
`cozo` `wasm` feature, `getrandom` `js` feature, and `wasm-bindgen` to
`wasm32-unknown-unknown` so the native binary is unaffected. CI builds the
artifact with `wasm-pack build --target web` (or `wasm-bindgen` directly),
runs `wasm-opt -Oz`, and stages it under the docs static directory.

- Open item: confirm `wasm-pack` availability in CI (it was not installed in
  the investigation environment). Fallback: `cargo build --target
  wasm32-unknown-unknown` + `wasm-bindgen` CLI.

## Risks / Trade-offs

- **Cozo WASM binary size.** Cozo core + dont rules compiled to WASM is likely
  1–3 MB gzipped. Mitigation: `wasm-opt -Oz`, LTO, and accept the size for a
  docs page served once and cached. Measure in the first implementation task;
  if >5MB gzipped, revisit trimming unused cozo features (graph-algo, etc.).
- **`chrono` `clock` feature on WASM** (EDGE-004). `now_rfc3339_seconds` uses
  `Utc::now`. On `wasm32-unknown-unknown` this may need the `js-sys` path
  already pulled by cozo's `wasm` feature. Mitigation: verify in the first
  build task; if it fails, inject timestamps from JS via the FFI.
- **Snapshot schema drift.** A docs snapshot built with one `dont` version may
  not match a WASM module built with another. Mitigation: D2 schema-version
  check + ship snapshot and WASM artifact from the same CI build.
- **Read-only enforcement for Datalog.** `run_datalog` MUST reject mutable
  scripts so a docs page visitor cannot corrupt the in-memory DB. Mitigation:
  pass `immutable=true` as the third argument to `run_script_str` (Cozo's
  signature is `run_script_str(payload, params, immutable: bool)` —
  `query_rows` already passes `true` for read-only; `run` passes `false`).
  Cozo itself rejects mutable operations in immutable mode, so this is
  enforced by the engine, not by token-scanning in our code.

## Migration Plan

1. Add `Store::open_mem()` + `Serialize` derives behind the existing lib; no
   CLI behavior changes. Native tests pass unchanged.
2. Add `dont export snapshot --json`; add a round-trip test (export →
   `open_mem` + `put_datoms` → `list_claims` equals source `list_claims`).
3. Add the `cdylib` WASM target + FFI; build in CI.
4. Add the mdBook page + JS loader + committed snapshot.
5. Rollback: the WASM artifact and docs page are additive; removing them
   restores the prior docs site. `Store::open_mem` and the export command are
   additive and unused by the native CLI path.

## Open Questions

- Should the docs page ship a curated example snapshot, or export the `dont`
  project's own `.dont/db.cozo` at docs-build time? (Lean: curated example, to
  keep the docs self-contained and avoid leaking internal claims.)
- Is `wasm-pack` acceptable as a CI dependency, or should the build use
  `cargo` + `wasm-bindgen` directly? Resolve in the first build task.
