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

- **`hints` presence**: `hints` is strictly required as an array on success envelopes (empty `[]` if unused). Omission is non-conformant for producers, though parsers MUST tolerate missing `hints` for legacy v0.1/v0.2 compatibility.
- **`meta.tx` constraints**: `meta.tx` is strictly `null` (not omitted) for read-only commands and an integer in the safe JSON range `[1, 2^53-1]` for mutating commands.
- **Import idempotence**: Keyed by `canonical_source_id` with importer-specific normalization rules (SHA-256 content hash for local files to handle aliases/symlinks, and deterministic whitespace/comment normalization for SPARQL queries).
- **Network safety policy**: HTTP imports strictly refuse non-HTTP(S) schemes and blocked destination classes (loopback/link-local/multicast/private) by default. Mixed DNS answer sets are denied immediately if *any* resolved address is blocked.
- **Import safety refusals**: Use deterministic codes: blocked destination/scheme → `unresolvable-uri`; policy-evaluation config failure → `config-missing`.
- **Spawn race determinism**:
  - Late callbacks arriving after a timeout MUST be accepted (to preserve epistemic work) but surface a `spawn-expired` warning.
  - Late timeout sweeps encountering an already-resolved spawn MUST ignore the spawn (transaction commit order wins).
  - Duplicate terminal callbacks MUST be explicitly refused with `spawn-already-resolved`.

## Risks / Trade-offs

- Tightened contracts may reveal existing non-conformant implementations during adoption.
- Network safety defaults can block uncommon on-prem use cases; future explicit allowlist controls can address that.

## Migration Plan

- Treat missing `hints` in existing parser implementations as backward-compatibility tolerance, but keep producer conformance strict.
- Add conformance tests for `meta.tx`, producer/parser `hints` obligations, import identity normalization, URL policy checks (including mixed DNS answers), and spawn-race resolution.
