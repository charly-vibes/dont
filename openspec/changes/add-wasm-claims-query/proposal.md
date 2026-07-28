# Change: add WASM claims-query surface for the docs site

## Why

The `dont` inference system (the shipped rule engine plus Cozo Datalog over the
claims DB) currently lives only behind the native CLI. Readers of the published
mdBook docs cannot inspect a real claims database, run a shipped rule, or try an
ad-hoc Datalog query without installing the binary. A browser-resident query
surface — compiled from the same Rust library the CLI uses — would let the docs
site host an interactive "claims explorer" with zero install and identical
semantics to the local tool.

The core enabling fact, validated during investigation: Cozo's in-memory engine
(`DbInstance::new("mem", ...)`) is **not** feature-gated and compiles cleanly to
`wasm32-unknown-unknown` with `cozo` `features=["wasm"]` plus the standard
`getrandom = { features=["js"] }` fix. No npm package, no IndexedDB storage
backend, no native SQLite in the browser is required.

## What Changes

- **New capability `dont-wasm-query`**: a WebAssembly module compiled from the
  `dont` library that embeds Cozo's in-memory engine, rehydrates a claims
  database from a portable snapshot, and exposes a small `wasm-bindgen` FFI for
  running shipped rules and ad-hoc Datalog queries.
- **New read query `dont export snapshot`**: a CLI command that dumps the live
  claims DB as a portable datom-level JSON snapshot consumable by the WASM
  module (and by any other tooling). Distinct from the existing `--eval`
  aggregate export.
- **New docs-site page**: an interactive claims-query page in the mdBook site
  that loads the WASM module plus a committed snapshot and lets the reader
  browse claims/terms, run shipped rules, and execute Datalog queries.
- **New build artifact**: a `wasm32-unknown-unknown` `cdylib` build target
  alongside the existing single binary, produced in CI and versioned as a
  static asset under the docs source.
- **Library refactor (minimal)**: add a `Store::open_mem()` constructor that
  builds a Cozo mem-engine `DbInstance` without filesystem access, file locks,
  the `tx.seq` sidecar, or panic-guarded corrupt-file detection. Existing query
  methods (`list_claims`, `list_terms`, `term_by_id`, `resolve_curie_reference`,
  `run_rule_query`) are engine-agnostic and are reused unchanged.

## Scope and non-goals

In scope:
- Read-only query surface in the browser (list, show-equivalent, shipped rules,
  ad-hoc Datalog).
- A snapshot export + rehydration path with identical query semantics to the CLI.
- CI build of the WASM artifact and mdBook page wiring.

Out of scope (explicitly deferred):
- Write operations (conclude/trust/flag/define) in the browser. The WASM surface
  is read-only; the snapshot is rebuilt from the CLI at docs-build time.
- Live sync between a local project and the docs page. The docs page ships a
  frozen snapshot committed to the repo.
- CLI JSON view parity (`build_claim_view` and its `&Store`-dependent
  projections). The WASM surface returns raw record data plus rule results, not
  the full CLI `ClaimView` with derived assessments. That parity is a separate,
  larger effort.
- A trait abstraction over `Store`. Reused via a second constructor, not a trait,
  to avoid rippling through `main.rs`.

## Impact

- Affected specs:
  - `dont-wasm-query` (NEW)
  - `dont-envelope` (ADDED requirement: `Snapshot` envelope kind)
  - `docs-site` (ADDED requirement: interactive claims-query page)
  - `dont-build` (ADDED requirement: WASM build artifact, orthogonal to the
    single-binary rule)
  - `dont-derived-queries` (ADDED requirement: portable snapshot export)
- Ordering dependency: the `docs-site` delta ADDEDs requirements to the
  `docs-site` capability, which is established by the completed but
  not-yet-archived `add-mdbook-docs-site` change. That change MUST be archived
  (creating `openspec/specs/docs-site/spec.md`) before this change is
  archived; until then this proposal's `docs-site` delta is conditional on it.
- Affected code:
  - `src/store.rs` — add `Store::open_mem()`; add a public `import_datoms`
    entry point that reuses the internal `put_datoms` path; add `Serialize`
    to `Datom` if needed for the snapshot path; keep existing methods
    unchanged.
  - `src/envelope.rs` — add the `Snapshot` variant to `EnvelopeKind`.
  - `src/main.rs` — add `dont export snapshot` subcommand mode.
  - `Cargo.toml` — make Cozo storage-backend features independently
    selectable (target-gate `storage-sqlite`/`storage-sqlite-src` for native;
    `cozo` `wasm` feature + `getrandom` `js` feature for `wasm32`); add
    `wasm-bindgen`; add a `wasm` feature-gated in-crate FFI module.
  - `src/wasm.rs` (NEW) — the `wasm-bindgen` FFI module, feature-gated so it
    compiles only for the `wasm32` target and does not affect the native
    binary.
  - `docs/` — new mdBook page + JS loader + committed snapshot asset.
  - `.github/workflows/` — WASM build + asset staging in the docs workflow.
- Affected build: a second artifact (`dont_query.wasm`) is produced in CI; the
  native single-binary distribution is unchanged.

## Traceability

- Enabling fact (Cozo mem engine on WASM): validated by `cargo check
  --target wasm32-unknown-unknown` with `cozo 0.7.6, features=["wasm"]` +
  `getrandom 0.2 js` (17s clean build). `cozo/src/lib.rs:138` shows `"mem"` is
  not feature-gated.
- Rule read-surface (4 methods): `src/rules/*.rs` use only `list_claims`,
  `list_terms`, `term_by_id`, `resolve_curie_reference`; file-based `.dl` rules
  additionally use `run_rule_query` (`src/rules/mod.rs:210`).
- Snapshot strategy (datom-level, not Cozo backup): Cozo `backup_db` /
  `restore_backup` require `storage-sqlite` + a filesystem
  (`cozo/src/runtime/db.rs:609,627`), so they are incompatible with the WASM mem
  build. The snapshot is a `*datoms` row dump rehydrated via the existing
  `put_datoms` path.
