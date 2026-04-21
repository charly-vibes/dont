## 1. Project Scaffold
- [ ] 1.1 Initialize Rust project (`cargo init`), add dependencies: clap (derive), cozo, serde, serde_json, ulid
- [ ] 1.2 Add `just` recipes: build, test, lint, run

## 2. Storage Layer (Red→Green→Tidy)
- [ ] 2.1 Test + implement: CozoDB database open/create at `.dont/db.cozo` with claim and event schemas (per §4.2); timestamps as RFC 3339 UTC strings (per §10.3)
- [ ] 2.2 Test + implement: append event (write) and query claim by ID; entity IDs use `claim:` and `event:` prefixes with ULID (per §10.3)
- [ ] 2.3 Tidy: extract `store` module with clean API boundary

## 3. JSON Envelope & Error Types (per §10.2, §10.5)
- [ ] 3.1 Test + implement: `Envelope<T>` serialization with `ok`, `envelope_version: "0.2"`, `cli_version`, `envelope_kind`, `data`, structured `warnings[]` (`{rule_name?, entity_id?, message, suggested_remediation?}`), `hints` and `meta` as `Option` (always `None` for tracer)
- [ ] 3.2 Test + implement: `ErrorResult` with `code`, `message`, `rule_name?`, `spec_ref?`, `entity_id?`, `unmet_clauses[]`, and non-empty `remediation[{command, description}]` invariant (constructor refuses empty)
- [ ] 3.3 Tidy: extract `envelope` module

## 4. Status Lattice (per §5.1, §9.0)
- [ ] 4.1 Test + implement: `Status` enum (Unverified, Verified, Doubted) with 4 valid transitions: unverified→doubted (trust), unverified→verified (dismiss), verified→doubted (trust), doubted→verified (dismiss)
- [ ] 4.2 Test: invalid transitions (Doubted→Unverified, Verified→Unverified, Doubted→Doubted, Verified→Verified) return typed refusal
- [ ] 4.3 Tidy: extract `model` module with Status, Claim, Event types

## 5. CLI: init (per §4.4, §14)
- [ ] 5.1 Test + implement: `dont init` creates `.dont/` directory with database and minimal `config.toml`
- [ ] 5.2 Test: `dont init` on existing project returns error envelope with code `already-initialised`, exit 3 (substrate/config per §10.7.1), remediation pointing to existing `.dont/` directory

## 6. CLI: conclude (per §9.1)
- [ ] 6.1 Test + implement: `dont conclude "claim text"` creates unverified claim, returns ClaimView envelope
- [ ] 6.2 Test: conclude outside initialized project returns error with remediation ("run dont init first"), exit 3

## 7. CLI: trust (per §9.3 — "trust doubts an entity", §5.1 transition table)
- [ ] 7.1 Test + implement: `dont trust <id> --reason "..."` transitions unverified→doubted, records `trusted` event
- [ ] 7.2 Test: trust without --reason returns refusal with remediation (reason-required), exit 1
- [ ] 7.3 Test + implement: `dont trust <id> --reason "..."` transitions verified→doubted (re-doubting a verified claim)
- [ ] 7.4 Test: trust on already-doubted claim returns refusal with remediation (invalid-transition), exit 1

## 8. CLI: dismiss (per §9.4 — "dismiss verifies an entity", §5.1 transition table)
- [ ] 8.1 Test + implement: `dont dismiss <id> --reason "..." --evidence "..."` transitions unverified→verified, records `dismissed` event
- [ ] 8.2 Test: dismiss without --reason returns refusal (reason-required), exit 1
- [ ] 8.3 Test: dismiss without --evidence returns refusal (no-evidence), exit 1
- [ ] 8.4 Test + implement: `dont dismiss <id> --reason "..." --evidence "..."` transitions doubted→verified (clearing doubt with evidence)

## 9. CLI: show and list (per §10.4)
- [ ] 9.1 Test + implement: `dont show <id>` returns ClaimView with event history
- [ ] 9.2 Test + implement: `dont list` returns all claims with current status
- [ ] 9.3 Test: show with nonexistent ID returns error with remediation, exit 1

## 10. Integration Tests
- [ ] 10.1 End-to-end: init → conclude → trust → show (doubted claim with 2 events: concluded + trusted)
- [ ] 10.2 End-to-end: init → conclude → dismiss → show (verified claim with 2 events: concluded + dismissed)
- [ ] 10.3 End-to-end: full cycle — conclude → trust (→doubted) → dismiss with evidence (→verified) — proves both re-transition paths work
- [ ] 10.4 End-to-end: refusal loop — conclude → dismiss (no evidence) → error with structured remediation[{command, description}] → dismiss (with evidence) → verified
- [ ] 10.5 Performance: `dont list` completes in <50ms on project with 100 claims
