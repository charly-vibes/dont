---
tags: [pipeline-run:tdd-ro5-2026-06-16-add-eval-readiness, pipeline-step:plan]
---

## add-eval-readiness: Implementation Plan

### What's being built
Three new capabilities:
1. **dont stats** — read-only command returning StatsView (verb_counts, idle_skill, claim_verification_rate, caught_contradiction_count, dedup_refusal_count) with --session/--since/--until scope flags
2. **--no-persist** — universal flag making any write command run validation-only (no store writes, no lock held)
3. **dont export --eval** — structured EvalExport JSON for eval harnesses (claims_by_status, events_by_kind, trust_events, dedup_refusals)

Plus required infrastructure:
- Claim text dedup check (NFC + lowercase + whitespace-collapse + SHA-256) needed for the --no-persist dedup scenario
- New store query methods: events in time range, all events, claim listing at scope end
- EnvelopeKind::Stats and EnvelopeKind::EvalExport

### Test strategy (TDD — write tests first)
Test files: tests/stats.rs, tests/no_persist.rs, tests/eval_export.rs

**stats tests:**
- bare stats on empty store → envelope_kind=stats, all zeros, idle_skill=true, claim_verification_rate=null
- stats after conclude → verb_counts contains 'conclude': 1, idle_skill=false
- alias dismiss maps to 'flag' in verb_counts
- inverted --since/--until → error envelope (ok=false)
- unknown --session → error envelope (ok=false)
- 10 claims 4 verified → claim_verification_rate=0.4
- doubt event on claim used as evidence → caught_contradiction_count increments
- doubt event on claim NOT used as evidence → no increment

**no-persist tests:**
- conclude --no-persist on valid claim → ok=true, no record in store (list still empty)
- conclude --no-persist on duplicate text → duplicate-refused error, no record
- trust --no-persist with hedge reason → hedge-rejection error, no record
- list --no-persist → identical to list (flag ignored)
- import --no-persist → ok=true, envelope.ephemeral=true, no terms in store

**export tests:**
- bare export --eval → envelope_kind=eval_export, empty arrays/maps
- export after trust events → trust_events populated with correct fields
- dedup_refusals format has attempted_text_hash (SHA-256 hex)
- --since/--until scoping filters events correctly

### Implementation order
1. Add Stats, EvalExport to EnvelopeKind (envelope.rs)
2. Add store query: events_since(since, until) → Result<Vec<EventRecord>> using CozoScript time filter
3. Add claim dedup: normalize_claim_text() → String, add claim_text_hash to store schema, check in append_claim, return StoreError::DuplicateClaim{hash, existing_id}
4. Handle DuplicateClaim in conclude handler → emit duplicate-refused error code
5. Implement dont stats command handler (Command::Stats variant in main.rs)
6. Add --no-persist as global arg (Cli struct), thread through write handlers
7. Implement dont export --eval command handler (Command::Export variant)
8. Integration tests for all scenarios
9. Update .dont/AGENTS.md managed block

### Key constraints
- dont stats and dont export MUST NOT acquire write transactions
- --no-persist MUST NOT hold write lock; validation reads current store state without locking
- caught_contradiction_count: retrospective CozoScript join over trust events (doubt=true) × evidence relations to find doubted claims that were prior evidence for other claims
- claim_verification_rate: store-wide at scope end, not bounded to scope window
- EvalExport.dedup_refusals.attempted_text_hash: SHA-256 of NFC-normalized, lowercased, whitespace-collapsed text
- EnvelopeKind serialization: 'stats' and 'eval_export' (lowercase with underscore)

### Scope boundary
- dedup_refusal_count counts StoreError::DuplicateClaim rejections that are logged as a new DedupRefused StoreEventKind; currently 0 until a claim with same normalized text is concluded twice
- Session scoping (--session flag) requires sessions directory to have boundary events; if not implemented, return an error identifying unknown session ID
- No real-time streaming, no per-verb ablation mode, no external benchmark integration
