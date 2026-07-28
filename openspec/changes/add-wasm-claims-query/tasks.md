# Tasks: add WASM claims-query surface

Tasks follow red → green → refactor per project TDD discipline. Each task is
independently verifiable. Do not start implementation until the proposal is
approved.

## 1. Library: mem-engine store constructor (TDD)
- [ ] 1.1 RED — write a test that opens a `Store` via `Store::open_mem()`,
  asserts `schema_version()` returns the expected version, and asserts no
  files are created on disk (no `db.cozo`, no `tx.seq`, no lock file).
- [ ] 1.2 GREEN — implement `Store::open_mem()` using
  `DbInstance::new("mem", "", "")`, skipping `with_file_lock`, `seq_path`, and
  `catch_unwind`. Make `path`/`lock_path`/`seq_path` `Option<PathBuf>` so the
  mem constructor need not invent paths. Add an in-memory tx counter for the
  mem path.
- [ ] 1.3 REFACTOR — extract the shared `ensure_schema()` call so `open_mem`
  and `open_dont_dir` share one initialization path.
- [ ] 1.4 Verify native `cargo test` still passes (no CLI behavior change).

## 2. Library: snapshot export + rehydration (TDD)
- [ ] 2.1 RED — write a round-trip test: populate a `Store` (mem) with claims,
  terms, events (including a status transition that writes a retraction);
  export snapshot via a new `Store::export_snapshot()` returning
  `Vec<Vec<Value>>` datom rows; rehydrate into a fresh `Store::open_mem()`
  via the new public `import_datoms`; assert `list_claims()` and `list_terms()`
  equal the source as sets after sorting by a stable key.
- [ ] 2.2 GREEN — implement `Store::export_snapshot()` as a `*datoms` query
  (reuse `query_rows`). Implement the public `Store::import_datoms(&[Datom])`
  wrapping the internal `put_datoms` plus the schema-version check; confirm
  `import_datoms` accepts the rehydrated rows unchanged. If `Datom` needs
  `Serialize`/`Deserialize` for the FFI layer, add it here.
- [ ] 2.3 RED — add a test that rehydration refuses a snapshot whose
  `schema_version` differs from the compiled expectation.
- [ ] 2.4 GREEN — enforce the schema-version check in `import_datoms`.
- [ ] 2.5 RED — add a test that calling `import_datoms` on an already-populated
  mem store resets the store before rehydrating (no hybrid state).
- [ ] 2.6 GREEN — implement reset-on-reload in `import_datoms`.

## 3. Library: FFI record serialization (TDD)
- [ ] 3.1 RED — write a test serializing `ClaimRecord`, `TermRecord`,
  `EventRecord`, `CurieResolution`, `ImportedTermRecord`, and `StoreEventKind`
  to JSON and back; assert round-trip equality.
- [ ] 3.2 GREEN — add `Serialize`/`Deserialize` derives to those five types
  (and `Status`, which is already derived). No behavior change to the CLI.
- [ ] 3.3 REFACTOR — confirm the CLI's hand-built `build_claim_view` still
  compiles and is unchanged (FFI serialization is additive).

## 4. CLI: `dont export snapshot` subcommand + envelope kind
- [ ] 4.1 RED — add an integration test (`tests/export_snapshot.rs`) that runs
  `dont export snapshot --json` on a fixture project and asserts the envelope
  contains a `rows` array of datom tuples and `envelope_kind: "snapshot"`.
- [ ] 4.2 GREEN — wire a `Snapshot` variant into the existing `Export` command
  (`src/main.rs`), distinct from `--eval`. Add the `Snapshot` variant to
  `EnvelopeKind` (`src/envelope.rs`, serialized as `"snapshot"`). Emit the
  datom rows via the envelope.
- [ ] 4.3 Add a `--out <path>` option to write the snapshot to a file (used by
  the docs build). Default emits to stdout as JSON.

## 5. WASM cdylib: FFI surface (TDD)
- [ ] 5.1 Add the `wasm32-unknown-unknown` target and a `wasm` feature-gated
  in-crate module (`src/wasm.rs`) with `wasm-bindgen`, target-gated `cozo`
  `wasm` feature, and `getrandom` `js` feature. Make the Cozo storage-backend
  features independently selectable in `Cargo.toml` so the same source
  compiles to both targets. Confirm `cargo check --target
  wasm32-unknown-unknown` succeeds.
- [ ] 5.2 RED — write a host-side test (rust test on native target using the
  shared lib) for `load_snapshot(json)` → `list_claims()` returning the
  rehydrated claims; and `run_shipped_rule("ungrounded")` returning matches.
- [ ] 5.3 GREEN — implement the FFI in `src/wasm.rs`: `load_snapshot`,
  `list_claims`, `list_terms`, `run_shipped_rule`, `run_datalog`. All return
  JSON envelope strings. `load_snapshot` uses `Store::import_datoms`
  (including reset-on-reload). `run_datalog` MUST pass `immutable=true`.
- [ ] 5.4 RED — add a test that `run_datalog` rejects a mutable script (e.g.
  `:put`, `:rm`, `:delete`, `:create`, or `:replace`) with an error envelope.
- [ ] 5.5 GREEN — confirm Cozo's immutable mode rejects the mutable script
  (enforced by the engine via `immutable=true`).
- [ ] 5.6 Verify `wasm-pack build --target web` (or `wasm-bindgen` fallback)
  produces a `.wasm` artifact; record gzipped size. If >5MB gzipped, fail CI
  or file a documented trim follow-up (per `dont-build` size budget).

## 6. CI: build and stage the WASM artifact
- [ ] 6.1 Add a CI job (or extend the docs workflow) that builds the WASM
  artifact, runs `wasm-opt -Oz`, and stages it under the docs static directory.
- [ ] 6.2 Fail CI if the WASM build fails or the artifact is missing.
- [ ] 6.3 Confirm the native single-binary build and `cargo test` are
  unaffected by the target-gated dependencies.

## 7. Docs: interactive claims-query page
- [ ] 7.1 Add an mdBook page (`docs/claims-explorer.md` or similar) with a JS
  loader that fetches the WASM artifact + committed snapshot.
- [ ] 7.2 Implement IndexedDB snapshot caching keyed by content hash (D4);
  rehydrate on load, re-fetch only on hash change.
- [ ] 7.3 Provide a minimal UI: list claims/terms, run a shipped rule, run a
  Datalog query box. Read-only.
- [ ] 7.4 Commit a curated example snapshot (not the project's own `.dont/`)
  under `docs/` for the page to consume.
- [ ] 7.5 Add the page to `docs/SUMMARY.md`.

## 8. Validate
- [ ] 8.1 Run `openspec validate add-wasm-claims-query --strict`.
- [ ] 8.2 Run `cargo test` (native) and the WASM host tests.
- [ ] 8.3 Run `just ci` / the docs build and confirm the page renders and the
  WASM query works in a local build.
