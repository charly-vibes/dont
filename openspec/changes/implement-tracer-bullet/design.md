## Context

This is the first implementation of `dont`. No code exists yet. The monolith spec (v0.3.2) and 7 OpenSpec capability decompositions define the target behaviour. This design covers only the tracer-bullet slice — enough to prove the architecture end-to-end.

Stakeholders: LLM harness agents (primary user), human auditors (secondary). The tracer bullet targets the LLM path only.

## Goals / Non-Goals

Goals:
- Prove Rust + CozoDB datoms as the storage substrate
- Validate the status lattice transitions
- Validate the JSON envelope + refusal protocol contract
- Achieve sub-50ms cold start on small projects
- Produce a working binary that accepts `init`, `conclude`, `trust`, `dismiss`, `show`, `list`

Non-Goals:
- Full CLI surface (completions, help tutorial, colour, signals)
- Spawn protocol or harness integration
- Datalog rule engine or shipped rules
- Import adapters or external ontology support
- Multi-platform release builds or packaging
- Human-readable output formatting (JSON-only for tracer)

## Decisions

### Crate structure: single crate, flat modules

A single `dont` crate with modules (`cli`, `store`, `model`, `envelope`). No workspace, no library crate split yet.

**Why:** The tracer bullet is small enough that premature crate boundaries would add friction. Split when integration tests need to import library types without going through the CLI binary, or when build time justifies it.

**Alternatives:** Workspace with `dont-core` lib + `dont-cli` bin. Rejected: adds Cargo complexity for ~2k lines of code.

### CLI framework: clap derive

Use `clap` with derive macros for command parsing. Subcommand enum maps 1:1 to verbs.

**Why:** Clap is the Rust standard. Derive API keeps command definitions close to handler code. Supports `--json` flag natively. Monolith §9 assumes clap-style conventions.

### Storage: CozoDB embedded with RocksDB backend

Use the `cozo` crate (≥0.7) with RocksDB for on-disk persistence at `.dont/db.cozo`.

**Why:** Settled in monolith §4.2. Datoms are the natural shape for event-sourced entities. Datalog queries are native. Time-travel is built in (deferred but free to add later).

**Schema (tracer-bullet subset):**

```
# Claims
:create claims { id: String => text: String, status: String, created_at: String, updated_at: String }

# Events (append-only log)
:create events { id: String => entity_id: String, kind: String, payload: String, author: String, timestamp: String }
```

The tracer store is throwaway — no production data will exist to migrate. Typed relations are simpler than the full datom model (entity/attribute/value/tx/op); the goal is proving CozoDB works, not building the final schema. Migration to full datoms happens when we implement time-travel and the rule engine.

**Trade-off:** We'll need a schema replacement when moving to full datoms. Acceptable because the tracer store is disposable by design.

### Entity IDs: ULIDs with type prefix

Claims get `claim:` prefix + ULID (e.g., `claim:01HY...`). Events get `event:` prefix + ULID. Matches the monolith §10.3 identity conventions.

**Why:** ULIDs sort chronologically, are CLI-friendly (no dashes), and the colon-separated prefix makes IDs self-describing in output and logs. The monolith uses `claim:`, `term:`, `event:`, `evidence:`, `spawn:` — colon is the canonical separator.

### Status lattice: enum with transition table

```rust
enum Status { Unverified, Verified, Doubted }

fn transition(from: Status, to: Status) -> Result<(), Refusal>
```

Per §9.0: `dont trust` *doubts* an entity ("submit for trust review — I do not trust this enough"); `dont dismiss` *verifies* an entity ("dismiss the doubt"). The tracer implements 4 valid transitions for the 3-state subset:
- `Unverified → Doubted` (via `trust --reason`) — per §5.1
- `Unverified → Verified` (via `dismiss --evidence`) — per §5.1
- `Verified → Doubted` (via `trust --reason`) — per §5.1
- `Doubted → Verified` (via `dismiss --evidence`) — per §5.1

All other transitions (e.g., `Doubted → Unverified`, `Verified → Unverified`) are refused. The full lattice (stale, locked, ignored, cascading) layers on by extending the enum and transition table.

**Why:** An enum + match is exhaustive — the compiler catches missing cases when we add states. A transition function centralises the lattice logic.

### JSON envelope: struct with serde

```rust
struct Envelope<T: Serialize> {
    ok: bool,
    envelope_version: String,  // "0.2" per §10.2
    cli_version: String,       // "0.0.1-tracer" for tracer
    envelope_kind: String,     // "claim", "claims", "events", "error"
    data: T,                   // ErrorResult when ok=false
    hints: Option<Vec<Hint>>,  // deferred: always None for tracer
    warnings: Vec<Warning>,
    meta: Option<Meta>,        // deferred: always None for tracer
}

struct Hint {
    command: String,
    description: String,
}

struct Warning {
    rule_name: Option<String>,
    entity_id: Option<String>,
    message: String,
    suggested_remediation: Option<String>,
}

struct ErrorResult {
    code: String,
    message: String,
    rule_name: Option<String>,
    spec_ref: Option<String>,
    entity_id: Option<String>,
    unmet_clauses: Vec<UnmetClause>,
    remediation: Vec<Remediation>,  // invariant: non-empty
}

struct UnmetClause {
    clause: String,
    fix: String,
}

struct Remediation {
    command: String,
    description: String,
}
```

**Why:** Maps directly to the envelope spec (§10.2, §10.5). Generic over `T` so each command provides its own payload type; when `ok: false`, `T` is `ErrorResult`. The `remediation` non-empty invariant from §3.2.5 is enforced at construction time via a builder that panics on empty remediation. `hints` and `meta` are `Option` — present in the type for forward compatibility but always `None` in the tracer.

### Refusal protocol: hardcoded checks, no rule engine

The tracer hardcodes three refusal checks:
1. **reason-required**: `trust` and `dismiss` require `--reason`
2. **no-evidence**: `dismiss` requires `--evidence`
3. **invalid-transition**: status lattice rejects impossible transitions

**Why:** The full rule engine (§13) evaluates Datalog rules. For the tracer, hardcoded checks prove the refusal→remediation→retry loop works. The rule engine is a separate work stream.

### Exit codes: per §10.7.1

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | Refusal (claim/lattice violation, verb-level validators) |
| 2 | Usage error (bad arguments, unknown flag) |
| 3 | Substrate or configuration error (db-locked, already-initialised) |
| 4 | Internal error (bugs) |

Deferred: 130/143 (signal handling).

## Risks / Trade-offs

- **CozoDB maturity risk** → Mitigated by the tracer itself: if CozoDB is problematic, we discover it before building the full system. RocksDB backend is the most tested Cozo path.
- **Simplified schema diverges from full datom model** → Accepted: tracer store is throwaway. No production data to migrate.
- **No rule engine means refusals are incomplete** → Accepted: hardcoded checks prove the protocol shape. Rule engine is deferred but the `ErrorResult` envelope is identical regardless of source.
- **JSON-only output** → Acceptable for tracer. Human-readable rendering is explicitly non-goal per monolith §3.1.4.

## Migration Plan

Not applicable — this is greenfield. When the full datom model is implemented, the tracer's typed-relation schema will be replaced. No backward compatibility needed.

## Open Questions

- **cozo crate version**: The monolith specifies ≥0.7. Need to verify current crate availability on crates.io and any API changes since the spec was written.
