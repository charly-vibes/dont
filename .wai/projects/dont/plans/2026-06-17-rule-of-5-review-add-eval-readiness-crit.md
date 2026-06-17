---
tags: [pipeline-run:tdd-ro5-2026-06-16-add-eval-readiness, pipeline-step:review]
---

## Rule of 5 Review — add-eval-readiness

### CRITICAL
1. DONT_NO_PERSIST env var unimplemented. Spec says SHALL honour DONT_NO_PERSIST=1. Fix: check env var in main.rs alongside cli.no_persist.
2. Default scope is epoch (1970-01-01), not today-midnight. Spec says default SHALL be the current calendar day (midnight-to-now UTC). Fix: compute today midnight UTC when since is None.

### HIGH
3. envelope.ephemeral field missing when --no-persist active. Spec says SHALL include ephemeral: true. Fix: add to Envelope struct.
4. apply_term_transition() not guarded by no_persist_mode(). trust/flag on terms writes despite --no-persist.
5. Command::Define, Ignore, Undoubt handlers do not check no_persist_mode().

### MEDIUM
6. claim_counts_by_status() calls heavy list_claims() — O(n×m) for n claims × m events.
7. all_events_in_scope() filters in Rust, not in CozoScript query.
8. TOCTOU race in dedup: check is outside write lock.
9. Old claims lack text_hash — dedup blind to pre-upgrade duplicates.
10. truncated and wall_clock_duration_seconds missing (task 5.7).

### LOW
11. NFC normalization missing from normalize_claim_text.
12. now_rfc3339_pub() leaky abstraction.
13. event_to_verb HashMap rebuilt per invocation.

### Verdict: NEEDS_REVISION — 2 CRITICAL and 3 HIGH gaps must be fixed.
