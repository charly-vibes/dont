## Context

This change is a spec-clarification pass across three capabilities. It does not add new user-facing verbs; it removes ambiguity and closes security/concurrency edge cases discovered during diagnostic evaluation.

## Goals / Non-Goals

- Goals:
  - Make envelope producer/parser obligations deterministic
  - Make import idempotence testable and implementation-independent
  - Add an explicit SSRF/private-network safety boundary for URL imports
  - Make spawn timeout/callback races deterministic under concurrent invocations
- Non-Goals:
  - Redesign envelope versioning
  - Introduce new import adapters
  - Change spawn command intent semantics

## Decisions

- `hints` is required on success envelopes; omission is non-conformant.
- `meta.tx` is `null` for read-only commands and an integer in `[1, 2^53-1]` for mutating commands.
- Import idempotence is keyed by `canonical_source_id` with importer-specific normalization rules (including deterministic SPARQL normalization and file-content hash authority for path aliases).
- HTTP imports refuse non-HTTP(S) schemes and blocked destination classes (loopback/link-local/multicast/private); mixed DNS answer sets are denied if any resolved address is blocked.
- Import safety refusals use deterministic codes: blocked destination/scheme → `unresolvable-uri`; policy-evaluation config failure → `config-missing`.
- Spawn timeout/callback collisions are resolved by transaction-commit order; loser path is surfaced as warning-only and must not apply a second terminal transition.

## Risks / Trade-offs

- Tightened contracts may reveal existing non-conformant implementations during adoption.
- Network safety defaults can block uncommon on-prem use cases; future explicit allowlist controls can address that.

## Migration Plan

- Treat missing `hints` in existing parser implementations as backward-compatibility tolerance, but keep producer conformance strict.
- Add conformance tests for `meta.tx`, producer/parser `hints` obligations, import identity normalization, URL policy checks (including mixed DNS answers), and spawn-race resolution.
