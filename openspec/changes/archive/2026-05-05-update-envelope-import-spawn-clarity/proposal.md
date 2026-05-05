# Change: tighten envelope, import, and spawn edge-case contracts

## Why

Quality diagnostics identified several high-impact ambiguities that would produce parser drift, inconsistent importer behaviour, and non-deterministic spawn handling under race conditions.

The highest-risk gaps are:
- contradictory `hints` semantics in success envelopes
- ambiguous `meta.tx` numeric constraints
- underspecified import idempotence identity
- missing import network-destination safety policy (SSRF/private-network fetch)
- unspecified timeout-vs-callback race handling for spawn requests

## What Changes

- Modify `dont-envelope` to make `hints` strictly required as an array on success envelopes, and `meta.tx` explicitly `null` (not omitted) for read-only commands and bounded `[1, 2^53-1]` for mutating commands.
- Modify `dont-import-surface` to define importer-specific canonical source identity for idempotence (SHA-256 for local files, deterministic whitespace/comment normalization for HTTP queries).
- Add a strict network destination safety requirement for HTTP-backed imports (default refusal of loopback/private/multicast destinations and mixed DNS answer sets).
- Modify and extend `dont-spawn-protocol` to define deterministic resolution of timeout/callback races (accepting late callbacks with warnings, ignoring resolved spawns on timeout sweeps, and explicitly refusing duplicate callbacks with `spawn-already-resolved`).

## Impact

- Affected specs: `dont-envelope`, `dont-import-surface`, `dont-spawn-protocol`
- Affected code (future): envelope builders/parsers, import adapters, spawn sweeper/callback handling
- Breaking change: no (tightens normative behaviour and clarifies edge semantics within v0.2 envelope family)
