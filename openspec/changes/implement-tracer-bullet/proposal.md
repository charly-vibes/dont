# Change: Implement tracer-bullet end-to-end slice

## Why

Seven OpenSpec changes decompose the monolith into capability specs, but none are implemented yet. Before investing in full-surface implementation, we need to prove the architecture works end-to-end: Rust + CozoDB datoms, the status lattice, the JSON envelope contract, and the refusal protocol. A tracer bullet through the thinnest viable cross-section will surface integration risks early and give us a working binary to iterate against.

## What Changes

- Add a Rust project scaffold (`Cargo.toml`, `src/`, `just` recipes)
- Implement CozoDB-backed event-sourced storage (datom writes, snapshot queries)
- Implement 6 CLI commands: `init`, `conclude`, `trust`, `dismiss`, `show`, `list`
- Implement the status lattice for the 3-state subset: `trust` → `:doubted`, `dismiss` → `:verified` (4 valid transitions across the 3 states)
- Implement the versioned JSON envelope (§10.2) with `ok`, `envelope_version: "0.2"`, `cli_version`, `envelope_kind`, `data`, structured `warnings[]`
- Implement the refusal protocol: `ErrorResult` in `data` with structured `remediation[{command, description}]`, hardcoded `reason-required` and `no-evidence` checks
- Add `dont-build` capability spec for build/distribution requirements (§4.1, §4.2)

## What This Proves

- Rust + CozoDB datom model works for the event-sourced design
- The four-verb lattice is implementable as specified
- The refusal-with-remediation pattern works end-to-end
- The JSON envelope contract is coherent
- Sub-50ms cold start is achievable

## What This Defers

- `define` verb (terms) — same lattice, layers on after
- `stale`, `locked`, `ignored` statuses and cascading
- Spawn protocol — requires harness integration
- Datalog rule engine — own work stream
- Import adapters — out-of-process, separate concern
- Time-travel queries, sessions, atom-level verification
- Shell completions, `--help` tutorial, colour handling, signal handling
- Seed vocabulary and mode system

## Spec Relationship

This change implements a thin cross-cutting slice of requirements already specified in:
- `add-core-dont-specs` — status lattice transitions (§5.1), four-verb semantics (§9)
- `add-dont-data-model-specs` — entity schema, event log, ULID identity (§4.2, §4.3)
- `add-dont-envelope-specs` — envelope contract (§10.2), error envelope (§10.5), exit codes (§10.7.1)
- `add-dont-project-layout-specs` — `.dont/` directory, config.toml, db.cozo (§14)

It adds one new capability (`dont-build`) for build/distribution requirements sourced from monolith §4.1 and §4.2, which are not covered by the behavioural spec changes.

## Impact

- Affected specs: `dont-build` (new)
- Affected code: new Rust crate at project root
- Affected workflow: transitions project from design phase to implement phase
