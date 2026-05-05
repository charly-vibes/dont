## 1. Project Scaffold
- [x] 1.1 Initialize Rust project (`cargo init`), add dependencies: clap (derive), cozo, serde, serde_json, ulid
- [x] 1.2 Add `just` recipes: build, test, lint, run

## 2. Storage Layer (Red→Green→Tidy)
- [ ] 2.1 Test + implement: CozoDB database open/create at `.dont/db.cozo` with true datom storage `(entity, attr, value, tx, assert_bit)` (per §4.2); timestamps as RFC 3339 UTC whole-second strings (per §10.3)
- [ ] 2.2 Test + implement: append event (write) and query claim by ID; writes use monotonically increasing transaction numbers, assertion/retraction datoms for status changes, and `claim:`/`event:` ULID prefixes (per §10.3)
- [ ] 2.3 Add store metadata with `schema_version: 1` to prevent future migration collisions
- [ ] 2.4 Tidy: extract `store` module with clean API boundary

## 3. JSON Envelope & Error Types (per §10.2, §10.5)
- [ ] 3.1 Test + implement: `Envelope<T>` serialization with `ok`, `envelope_version: "0.2"`, `cli_version`, `envelope_kind`, `data`, required structured `warnings[]` (`{rule_name, entity_id?, message, suggested_remediation?}`), required `meta {duration_ms, tx, request_id}`, and required success-only `hints[]` (empty for tracer when no next action applies; omitted on error envelopes)
- [ ] 3.2 Test + implement: `ErrorResult` with `code`, `message`, `rule_name?`, `spec_ref?`, `entity_id?`, `unmet_clauses[]`, and non-empty `remediation[{command, description}]` invariant (constructor refuses empty)
- [ ] 3.3 Tidy: extract `envelope` module

## 4. Status Lattice (per §5.1, §9.0)
- [ ] 4.1 Test + implement: `Status` enum (Unverified, Verified, Doubted) with 4 valid transitions: unverified→doubted (trust), unverified→verified (dismiss), verified→doubted (trust), doubted→verified (dismiss)
- [ ] 4.2 Test: invalid state-changing transitions (Doubted→Unverified, Verified→Unverified, repeat `trust` on Doubted) return typed refusal; already-verified `dismiss` is not a status transition and is covered as evidence append in 8.4
- [ ] 4.3 Tidy: extract `model` module with Status, Claim, Event types

## 5. CLI: Context & init (per §4.4, §14)
- [ ] 5.1 Test + implement: `dont init` creates `.dont/` with the canonical core entries needed for the tracer (`db.cozo`, `config.toml`, `AGENTS.md`, `seed/`, `vocab/`, `rules/`, `imports/`, `sessions/`, `schemas/`) and a valid minimal config (`[project]`, `[output]`, `[trust.hedges]`, `[storage]`)
- [ ] 5.2 Test: `dont init` on existing project returns error envelope with code `already-initialised`, exit 3 (substrate/config per §10.7.1), remediation pointing to existing `.dont/` directory
- [ ] 5.3 Implement recursive parent-directory search for `.dont/` (Project Root discovery)
- [ ] 5.4 Support `DONT_DIR` environment variable as a Project Root override for test isolation

## 6. CLI: conclude (per §9.1)
- [ ] 6.1 Test + implement: `dont conclude "claim text"` creates unverified claim, returns ClaimView envelope
- [ ] 6.2 Test: conclude outside initialized project returns error with remediation ("run dont init first"), exit 3

## 7. CLI: trust (per §9.3 — "trust doubts an entity", §5.1 transition table)
- [ ] 7.1 Test + implement: `dont trust <id> --reason "..."` transitions unverified→doubted, records `trusted` event
- [ ] 7.2 Test: trust without --reason returns refusal with remediation (reason-required), exit 1
- [ ] 7.3 Test + implement: `dont trust <id> --reason "..."` transitions verified→doubted (re-doubting a verified claim)
- [ ] 7.4 Test: trust on already-doubted claim returns refusal with remediation (invalid-transition), exit 1
- [ ] 7.5 Implement "Hedge MVP": refuse reasons containing default case-insensitive hedge substrings (`i think`, `maybe`, `not sure`, `probably`) with code `reason-not-hedge`; do not use regex evaluation

## 8. CLI: dismiss (per §9.4 — "dismiss verifies an entity", §5.1 transition table)
- [ ] 8.1 Test + implement: `dont dismiss <id> --evidence "..."` transitions unverified→verified, records `dismissed` event
- [ ] 8.2 Test: dismiss without --evidence returns refusal (no-evidence), exit 1
- [ ] 8.3 Test + implement: `dont dismiss <id> --evidence "..."` transitions doubted→verified (clearing doubt with evidence)
- [ ] 8.4 Test + implement: `dont dismiss <id> --evidence "..."` on an already verified claim appends evidence/history without creating a new identity or requiring a status change

## 9. CLI: show and list (per §10.4)
- [ ] 9.1 Test + implement: `dont show <id>` returns ClaimView with event history and hardcoded empty `applicable_rules: {}`
- [ ] 9.2 Test + implement: `dont list` returns all claims with current status and hardcoded empty `applicable_rules: {}`
- [ ] 9.3 Test: show with nonexistent ID returns error with remediation, exit 1

## 10. Integration Tests
See `test-strategy.md` for full acceptance criteria, envelope contract requirements, and required coverage table.
- [ ] 10.1 End-to-end: init → conclude → trust → show (doubted claim with 2 events: concluded + trusted)
- [ ] 10.2 End-to-end: init → conclude → dismiss → show (verified claim with 2 events: concluded + dismissed)
- [ ] 10.3 End-to-end: full transition cycle — conclude → dismiss (→verified) → trust (→doubted) → dismiss with evidence (→verified) — proves `unverified→verified`, `verified→doubted`, and `doubted→verified` paths work through the binary
- [ ] 10.4 End-to-end: refusal loop — conclude → dismiss (no evidence) → error with structured remediation[{command, description}] → dismiss (with evidence) → verified
- [ ] 10.5 Performance: `dont list --json` completes in <50ms on project with 100 claims (use `DONT_DIR` for seed isolation; seed via `conclude` invocations)
- [ ] 10.6 Persistence: conclude then separate process `show` returns the same claim
- [ ] 10.7 Discovery: run command from project subdirectory, confirm `.dont/` found via parent walk
- [ ] 10.8 Discovery: run command outside any project, confirm `config-missing` exit 3
- [ ] 10.9 Hedge: `trust --reason "maybe"` returns `reason-not-hedge`, exit 1, non-empty remediation
- [ ] 10.10 Envelope: assert `envelope_version: "0.2"`, required `warnings[]`, required `meta`, and success-only `hints[]` on every command's output (parameterised over all 6 commands)
- [ ] 10.11 Evidence append: dismiss an already verified claim with additional evidence and confirm status remains `verified` while event/history/evidence output records the new evidence
